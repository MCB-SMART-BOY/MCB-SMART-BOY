#[path = "build_support/mod.rs"]
mod build_support;

use std::{env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("content/posts");
    println!("cargo:rerun-if-changed={}", root.display());
    let generated = build_support::generate_content(&root)?;
    let output = PathBuf::from(env::var("OUT_DIR")?).join("content_index.rs");
    fs::write(&output, generated).map_err(|source| {
        build_support::io_error(&output, "write generated content index", source)
    })?;
    Ok(())
}
