use std::env;
use std::path::PathBuf;
use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use tar::Archive;

const ARCHIVE_URL: &str = "https://github.com/moonbitlang/moonbit-compiler/releases/download/v0.6.24%2B012953835/moonbit-wasm.tar.gz";
const TARGET_DIR_IN_ARCHIVE: &str = "./moonc.assets";
const TARGET_EXTENSION: &str = "wasm";
const FINAL_FILE_NAME: &str = "moonc.wasm";

fn main() -> Result<()> {
    // Tell Cargo to rerun this script if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");

    //  Get the OUT_DIR environment variable set by Cargo
    //  This is the recommended location for our downloaded file
    let out_dir = env::var("OUT_DIR").context("Failed to get OUT_DIR environment variable")?;
    let dest_path = PathBuf::from(out_dir).join(FINAL_FILE_NAME);

    //  Download the file using reqwest (blocking)
    //  Using a blocking request is simpler in a build script
    let response = reqwest::blocking::get(ARCHIVE_URL)
        .with_context(|| format!("Failed to download file from URL: {}", ARCHIVE_URL))?;

    // Check the HTTP response status
    if !response.status().is_success() {
        anyhow::bail!("Received an error status code while downloading the file: {}", response.status());
    }

    let content = response.bytes()
        .context("Failed to read response content")?;

    // Decompress and extract the specific file from the archive in memory
    let tar = GzDecoder::new(content.as_ref());
    let mut archive = Archive::new(tar);

    let mut entry_found = false;
    for entry_result in archive.entries()? {
        let mut entry = entry_result.context("Failed to read archive entry")?;
        let path = entry.path()?.to_path_buf();
        // Check if the entry is in the target directory and has the correct extension
        if path.starts_with(TARGET_DIR_IN_ARCHIVE) && path.extension().and_then(|s| s.to_str()) == Some(TARGET_EXTENSION) {
            println!("cargo:info=Found target file in archive: {:?}", path);
            
            entry.unpack(&dest_path)
                .with_context(|| format!("Failed to unpack file {:?} to {:?}", path, dest_path))?;
            
            entry_found = true;
            break; // File found and extracted, no need to continue
        }
    }

    if !entry_found {
        anyhow::bail!(
            "Could not find a file with extension '.{}' in the directory '{}' within the downloaded archive.",
            TARGET_EXTENSION,
            TARGET_DIR_IN_ARCHIVE
        );
    }

    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=advapi32");
        println!("cargo:rustc-link-lib=tdh");
    }

    Ok(())
}