//! Makes every Markdown guide available to rustdoc's doctest runner.

use std::{env, fs, io, path::Path};

fn collect_pages(directory: &Path, pages: &mut Vec<std::path::PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_pages(&path, pages)?;
        } else if path.extension().is_some_and(|extension| extension == "md") {
            pages.push(path);
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=docs");
    let mut pages = Vec::new();
    collect_pages(Path::new("docs"), &mut pages)?;
    pages.sort();

    let mut tests = String::new();
    for (index, path) in pages.iter().enumerate() {
        // A Rust string literal also escapes Windows paths and quotes in paths.
        let path = fs::canonicalize(path)?;
        tests.push_str(&format!(
            "#[doc = include_str!({:?})]\nmod page_{index} {{}}\n",
            path.to_str().ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "Documentation paths must be UTF-8"
            ))?
        ));
    }
    fs::write(
        Path::new(&env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR")).join("documentation.rs"),
        tests,
    )
}
