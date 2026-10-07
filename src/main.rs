use std::{env, ffi::OsStr, fs, io, net::SocketAddr, path::Path};

use leptos::prelude::{LeptosOptions, get_configuration};
use mcb_smart_boy::{export::export_site, server::build_app};

fn read_address(default: SocketAddr) -> Result<SocketAddr, io::Error> {
    match env::var("BLOG_ADDR") {
        Ok(value) => value.parse().map_err(|source| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid BLOG_ADDR {value}: {source}"),
            )
        }),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(source) => Err(io::Error::new(io::ErrorKind::InvalidInput, source)),
    }
}

fn validate_site_root(options: &LeptosOptions) -> Result<(), io::Error> {
    let expected = Path::new(env!("CARGO_MANIFEST_DIR")).join("run/site");
    let actual = fs::canonicalize(options.site_root.as_ref()).map_err(|source| {
        io::Error::new(io::ErrorKind::InvalidInput,
            format!("failed to resolve configured site root {}: {source}; build with ./dev.sh leptos build", options.site_root))
    })?;
    let expected = fs::canonicalize(&expected).map_err(|source| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "failed to resolve expected site root {}: {source}; build first",
                expected.display()
            ),
        )
    })?;
    if actual != expected || options.site_pkg_dir.as_ref() != "pkg" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "unexpected static site root {} or package directory {}; expected {} / pkg",
                actual.display(),
                options.site_pkg_dir,
                expected.display()
            ),
        ));
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = env::args_os().skip(1);
    let command = arguments.next();
    if arguments.next().is_some()
        || command
            .as_deref()
            .is_some_and(|value| value != OsStr::new("export"))
    {
        return Err(
            io::Error::new(io::ErrorKind::InvalidInput, "usage: mcb-smart-boy [export]").into(),
        );
    }
    let config = get_configuration(Some("Cargo.toml"))?;
    let mut options = config.leptos_options;
    if command.is_some() {
        let output = export_site(options).await?;
        println!("Exported static pages to {}", output.display());
        return Ok(());
    }
    validate_site_root(&options)?;
    let address = read_address(options.site_addr)?;
    options.site_addr = address;
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|source| {
            io::Error::new(
                source.kind(),
                format!("failed to bind blog at {address}: {source}"),
            )
        })?;
    println!("Blog demo listening on http://{address}");
    axum::serve(listener, build_app(options))
        .await
        .map_err(|source| {
            io::Error::new(
                source.kind(),
                format!("blog server failed at {address}: {source}"),
            )
        })?;
    Ok(())
}
