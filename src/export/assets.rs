use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::Read,
    path::{Path, PathBuf},
};

use leptos::prelude::LeptosOptions;

use super::{
    ExportError, invalid, io_error, output::PreparedOutput, validate_directory,
    validate_regular_file,
};

const REQUIRED_HASHES: [&str; 3] = ["css", "js", "wasm"];

pub(super) fn read_hashes(options: &LeptosOptions) -> Result<BTreeSet<PathBuf>, ExportError> {
    let name = Path::new(options.hash_file.as_ref());
    if name.file_name().is_none() || name.components().count() != 1 {
        return Err(invalid(
            name,
            "hash manifest must be a file beside the export binary",
        ));
    }
    let executable = env::current_exe().map_err(|e| io_error("locate export binary", name, e))?;
    let manifest = executable
        .parent()
        .ok_or_else(|| invalid(&executable, "export binary has no parent"))?
        .join(name);
    read_hashes_at(&manifest, options.output_name.as_ref())
}

fn read_hashes_at(manifest: &Path, output_name: &str) -> Result<BTreeSet<PathBuf>, ExportError> {
    if output_name.is_empty()
        || !output_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(invalid(
            Path::new(output_name),
            "unsafe package output name",
        ));
    }
    validate_regular_file(manifest)?;
    let text =
        fs::read_to_string(manifest).map_err(|e| io_error("read hash manifest", manifest, e))?;
    if !text.ends_with('\n') {
        return Err(invalid(manifest, "incomplete hash manifest"));
    }
    let mut hashes = BTreeMap::new();
    for entry in text.lines() {
        let (kind, value) = entry
            .split_once(':')
            .ok_or_else(|| invalid(manifest, "malformed hash entry"))?;
        let value = value.trim();
        if !["css", "js", "wasm", "manifest", "split"].contains(&kind)
            || value.len() != 22
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || hashes.insert(kind, value).is_some()
        {
            return Err(invalid(
                manifest,
                "unexpected, duplicated or invalid hash entry",
            ));
        }
    }
    let mut required = BTreeSet::new();
    for kind in REQUIRED_HASHES {
        let hash = hashes
            .get(kind)
            .ok_or_else(|| invalid(manifest, "missing required CSS, JS or WASM hash"))?;
        required.insert(PathBuf::from(format!("{output_name}.{hash}.{kind}")));
    }
    if hashes.contains_key("manifest") != hashes.contains_key("split") {
        return Err(invalid(manifest, "incomplete WASM split manifest"));
    }
    if let Some(hash) = hashes.get("manifest") {
        required.insert(PathBuf::from(format!("__wasm_split_manifest.{hash}.json")));
    }
    if let Some(hash) = hashes.get("split") {
        required.insert(PathBuf::from(format!("__wasm_split.{hash}.js")));
    }
    Ok(required)
}

fn collect_files(root: &Path, base: &Path, files: &mut Vec<PathBuf>) -> Result<(), ExportError> {
    validate_directory(root)?;
    for entry in fs::read_dir(root).map_err(|e| io_error("list static assets", root, e))? {
        let entry = entry.map_err(|e| io_error("read static asset entry", root, e))?;
        let path = entry.path();
        let metadata =
            fs::symlink_metadata(&path).map_err(|e| io_error("inspect static asset", &path, e))?;
        if metadata.file_type().is_dir() {
            collect_files(&path, base, files)?;
        } else {
            validate_regular_file(&path)?;
            files.push(
                path.strip_prefix(base)
                    .map_err(|_| invalid(&path, "asset escapes its root"))?
                    .to_owned(),
            );
        }
    }
    Ok(())
}

fn copy_file(
    source: &Path,
    relative: &Path,
    output: &mut PreparedOutput<'_>,
) -> Result<(), ExportError> {
    validate_regular_file(source)?;
    let metadata = fs::metadata(source).map_err(|e| io_error("inspect static asset", source, e))?;
    if metadata.len() == 0 {
        return Err(invalid(source, "static asset is empty"));
    }
    if source.extension().is_some_and(|ext| ext == "wasm") {
        let mut signature = [0; 4];
        fs::File::open(source)
            .and_then(|mut file| file.read_exact(&mut signature))
            .map_err(|e| io_error("read WASM signature", source, e))?;
        if signature != *b"\0asm" {
            return Err(invalid(source, "invalid WASM signature"));
        }
    }
    output.copy(relative, source)
}

pub(super) fn copy_assets(
    pkg: &Path,
    public: &Path,
    required: &BTreeSet<PathBuf>,
    output: &mut PreparedOutput<'_>,
) -> Result<(), ExportError> {
    let mut pkg_files = Vec::new();
    collect_files(pkg, pkg, &mut pkg_files)?;
    let available: BTreeSet<_> = pkg_files.iter().cloned().collect();
    for asset in required {
        if !available.contains(asset) {
            return Err(invalid(
                &pkg.join(asset),
                "hashed asset named by manifest is missing",
            ));
        }
    }
    for file in pkg_files {
        let extension = file.extension().and_then(|e| e.to_str());
        if extension == Some("ts") {
            continue;
        } // wasm-bindgen's type declarations are not browser assets.
        if !matches!(extension, Some("css" | "js" | "wasm" | "json")) {
            return Err(invalid(&pkg.join(file), "unexpected package artifact"));
        }
        copy_file(&pkg.join(&file), &Path::new("pkg").join(file), output)?;
    }
    let mut public_files = Vec::new();
    collect_files(public, public, &mut public_files)?;
    for file in public_files {
        let extension = file.extension().and_then(|e| e.to_str());
        if file.starts_with("pkg")
            || !matches!(
                extension,
                Some(
                    "svg"
                        | "png"
                        | "jpg"
                        | "jpeg"
                        | "gif"
                        | "webp"
                        | "ico"
                        | "woff"
                        | "woff2"
                        | "ttf"
                        | "otf"
                        | "txt"
                )
            )
        {
            return Err(invalid(
                &public.join(file),
                "unexpected public asset or reserved package path",
            ));
        }
        copy_file(&public.join(&file), &file, output)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_manifest_rejects_missing_and_malformed_entries() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hash.txt");
        fs::write(
            &path,
            "css: 0123456789012345678901\njs: 0123456789012345678901\n",
        )
        .unwrap();
        assert!(read_hashes_at(&path, "mcb-smart-boy").is_err());
        fs::write(
            &path,
            "css: ../../outside\njs: 0123456789012345678901\nwasm: 0123456789012345678901\n",
        )
        .unwrap();
        assert!(read_hashes_at(&path, "mcb-smart-boy").is_err());
    }

    #[test]
    fn copy_assets_rejects_missing_hashed_bundle_without_writing_output() {
        let dir = tempfile::tempdir().unwrap();
        let pkg = dir.path().join("pkg");
        let public = dir.path().join("public");
        let target = dir.path().join("stage");
        fs::create_dir(&pkg).unwrap();
        fs::create_dir(&public).unwrap();
        fs::create_dir(&target).unwrap();
        let required = BTreeSet::from([PathBuf::from("missing.0123456789012345678901.wasm")]);
        assert!(copy_assets(&pkg, &public, &required, &mut PreparedOutput::new(&target)).is_err());
        assert!(fs::read_dir(target).unwrap().next().is_none());
    }
    #[test]
    fn copy_assets_rejects_invalid_wasm_and_copies_complete_bundle() {
        let dir = tempfile::tempdir().unwrap();
        let pkg = dir.path().join("pkg");
        let public = dir.path().join("public");
        let target = dir.path().join("stage");
        fs::create_dir(&pkg).unwrap();
        fs::create_dir(&public).unwrap();
        fs::create_dir(&target).unwrap();
        let css = "mcb.0123456789012345678901.css";
        let js = "mcb.0123456789012345678901.js";
        let wasm = "mcb.0123456789012345678901.wasm";
        fs::write(pkg.join(css), "body { color: black }").unwrap();
        fs::write(pkg.join(js), "export function hydrate() {}").unwrap();
        fs::write(pkg.join(wasm), "invalid").unwrap();
        fs::write(public.join("favicon.svg"), "<svg/>").unwrap();
        let required = BTreeSet::from([css.into(), js.into(), wasm.into()]);
        assert!(copy_assets(&pkg, &public, &required, &mut PreparedOutput::new(&target)).is_err());
        fs::write(pkg.join(wasm), b"\0asm\x01\0\0\0").unwrap();
        copy_assets(&pkg, &public, &required, &mut PreparedOutput::new(&target)).unwrap();
        assert_eq!(
            fs::read(target.join("pkg").join(wasm)).unwrap(),
            b"\0asm\x01\0\0\0"
        );
        assert_eq!(
            fs::read_to_string(target.join("favicon.svg")).unwrap(),
            "<svg/>"
        );
    }
}
