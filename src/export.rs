//! Builds a complete static site without binding an HTTP listener.
mod assets;
mod output;

use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

use axum::{
    body::{Body, Bytes, to_bytes},
    http::{Request, StatusCode},
};
use leptos::prelude::LeptosOptions;
use tower::ServiceExt;

use crate::{content::page_paths, server::build_app};

const MAX_PAGE_BYTES: usize = 16 * 1024 * 1024;
const OWNERSHIP_FILE: &str = ".mcb-static-export";

#[derive(Debug)]
pub enum ExportError {
    /// Invalid or untrusted input, including a non-owned previous output.
    Invalid { path: PathBuf, reason: &'static str },
    /// A public URL or asset would overwrite an existing output.
    Collision { path: PathBuf },
    /// A filesystem operation failed on the named path.
    Io {
        operation: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    /// Installing a completed export failed; rollback failure leaves the previous export retained.
    Replacement {
        target: PathBuf,
        retained_backup: Option<PathBuf>,
        install: io::Error,
        rollback: Option<io::Error>,
    },
    /// In-memory SSR failed for the named route.
    Render {
        route: String,
        source: Box<dyn Error + Send + Sync>,
    },
    /// The rendered route did not have its required HTTP status.
    Status {
        route: String,
        expected: StatusCode,
        actual: StatusCode,
    },
    /// SSR failed to produce a complete bounded response body.
    Body { route: String, source: axum::Error },
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid { path, reason } => write!(
                f,
                "unsafe or incomplete export input {}: {reason}",
                path.display()
            ),
            Self::Collision { path } => {
                write!(f, "static export output collision at {}", path.display())
            }
            Self::Io {
                operation,
                path,
                source,
            } => write!(f, "failed to {operation} {}: {source}", path.display()),
            Self::Replacement {
                target,
                retained_backup,
                install,
                rollback,
            } => {
                write!(
                    f,
                    "failed to install exported pages at {}: {install}",
                    target.display()
                )?;
                if let Some(rollback) = rollback {
                    write!(f, "; rollback also failed: {rollback}")?;
                }
                if let Some(path) = retained_backup {
                    write!(f, "; previous export retained at {}", path.display())?;
                }
                Ok(())
            }
            Self::Render { route, source } => write!(f, "failed to render route {route}: {source}"),
            Self::Status {
                route,
                expected,
                actual,
            } => write!(f, "route {route} returned {actual}, expected {expected}"),
            Self::Body { route, source } => {
                write!(f, "failed to read rendered route {route}: {source}")
            }
        }
    }
}

impl Error for ExportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Replacement { install, .. } => Some(install),
            Self::Render { source, .. } => Some(source.as_ref()),
            Self::Body { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn io_error(operation: &'static str, path: &Path, source: io::Error) -> ExportError {
    ExportError::Io {
        operation,
        path: path.to_owned(),
        source,
    }
}

fn invalid(path: &Path, reason: &'static str) -> ExportError {
    ExportError::Invalid {
        path: path.to_owned(),
        reason,
    }
}

fn validate_regular_file(path: &Path) -> Result<(), ExportError> {
    let metadata = fs::symlink_metadata(path).map_err(|e| io_error("inspect", path, e))?;
    if !metadata.file_type().is_file() || has_multiple_links(&metadata) {
        return Err(invalid(path, "expected an ordinary, singly linked file"));
    }
    Ok(())
}

#[cfg(unix)]
fn has_multiple_links(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    metadata.nlink() != 1
}

#[cfg(not(unix))]
fn has_multiple_links(_metadata: &fs::Metadata) -> bool {
    false
}

fn validate_directory(path: &Path) -> Result<(), ExportError> {
    if !fs::symlink_metadata(path)
        .map_err(|e| io_error("inspect", path, e))?
        .file_type()
        .is_dir()
    {
        return Err(invalid(path, "expected a real directory, not a symlink"));
    }
    Ok(())
}

fn validate_source(options: &LeptosOptions) -> Result<PathBuf, ExportError> {
    let run = Path::new(env!("CARGO_MANIFEST_DIR")).join("run");
    validate_directory(&run)?;
    let site = run.join("site-release");
    validate_directory(&site)?;
    if fs::canonicalize(options.site_root.as_ref()).map_err(|e| {
        io_error(
            "resolve site root",
            Path::new(options.site_root.as_ref()),
            e,
        )
    })? != fs::canonicalize(&site).map_err(|e| io_error("resolve release root", &site, e))?
        || options.site_pkg_dir.as_ref() != "pkg"
        || !options.hash_files
    {
        return Err(invalid(
            &site,
            "export requires the hashed run/site-release/pkg configuration",
        ));
    }
    validate_directory(&site.join("pkg"))?;
    Ok(run)
}

fn route_output(route: &str) -> Result<PathBuf, ExportError> {
    if route == "/" {
        return Ok(PathBuf::from("index.html"));
    }
    if route == "/404.html" {
        return Ok(PathBuf::from("404.html"));
    }
    if !route.starts_with('/') || !route.ends_with('/') {
        return Err(invalid(
            Path::new(route),
            "page URL must begin and end in /",
        ));
    }
    let mut result = PathBuf::new();
    for part in route[1..route.len() - 1].split('/') {
        if part.is_empty()
            || part == "."
            || part == ".."
            || part.chars().any(|character| {
                character.is_control() || matches!(character, '\\' | ':' | '%' | '?' | '#')
            })
        {
            return Err(invalid(Path::new(route), "unsafe page URL component"));
        }
        result.push(part);
    }
    Ok(result.join("index.html"))
}

async fn render_page(
    router: axum::Router,
    route: &str,
    expected: StatusCode,
) -> Result<Bytes, ExportError> {
    let request = Request::builder()
        .uri(route)
        .body(Body::empty())
        .map_err(|e| ExportError::Render {
            route: route.into(),
            source: Box::new(e),
        })?;
    let response = router
        .oneshot(request)
        .await
        .map_err(|e| ExportError::Render {
            route: route.into(),
            source: Box::new(e),
        })?;
    if response.status() != expected {
        return Err(ExportError::Status {
            route: route.into(),
            expected,
            actual: response.status(),
        });
    }
    let bytes = to_bytes(response.into_body(), MAX_PAGE_BYTES)
        .await
        .map_err(|source| ExportError::Body {
            route: route.into(),
            source,
        })?;
    if bytes.is_empty() || !bytes.starts_with(b"<!DOCTYPE html") {
        return Err(invalid(
            Path::new(route),
            "SSR returned empty or non-document HTML",
        ));
    }
    Ok(bytes)
}

/// Exports the indexed site, its shared 404 and verified static resources into run/pages/.
/// A failed preparation does not modify the last successfully exported site.
pub async fn export_site(options: LeptosOptions) -> Result<PathBuf, ExportError> {
    let run = validate_source(&options)?;
    let public = Path::new(env!("CARGO_MANIFEST_DIR")).join("public");
    validate_directory(&public)?;
    let required = assets::read_hashes(&options)?;
    let stage = tempfile::Builder::new()
        .prefix(".pages-stage-")
        .tempdir_in(&run)
        .map_err(|e| io_error("create staging directory under run", &run, e))?;
    let mut output = output::PreparedOutput::new(stage.path());
    assets::copy_assets(
        &run.join("site-release/pkg"),
        &public,
        &required,
        &mut output,
    )?;
    let router = build_app(options);
    for route in page_paths() {
        let path = route_output(route)?;
        let html = render_page(router.clone(), route, StatusCode::OK).await?;
        output.write(&path, &html)?;
    }
    let html = render_page(router, "/404.html", StatusCode::NOT_FOUND).await?;
    output.write(&route_output("/404.html")?, &html)?;
    output.finish()?;
    output::replace_output(&run, stage.path())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn route_output_rejects_traversal_and_missing_slash() {
        for route in [
            "/writing/../",
            "/writing//",
            "/writing/%2e%2e/",
            "/writing",
            "/a\\b/",
        ] {
            assert!(route_output(route).is_err(), "accepted {route}");
        }
        assert_eq!(
            route_output("/writing/deep/post/").unwrap(),
            PathBuf::from("writing/deep/post/index.html")
        );
    }
}
