use std::io::{Read, Seek};
use std::path::Path;
use std::process::{Command, Stdio};
use thiserror::Error;

use crate::reader::XipReader;

pub mod reader;

#[derive(Error, Debug)]
pub enum UnxipError {
    #[error("IO Error: {0}")]
    IoError(std::io::Error),
    #[error("XAR Error: {0}")]
    XarError(apple_xar::Error),
    #[error("Miscellaneous Error: {0}")]
    Misc(String),
}

pub fn unxip<R: Read + Seek + Sized + std::fmt::Debug>(
    reader: &mut R,
    output_path: &Path,
) -> Result<(), UnxipError> {
    if Command::new("cpio")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_err()
    {
        return Err(UnxipError::Misc(
            "cpio command not found. Please install it.".to_string(),
        ));
    }

    let mut xip_reader = XipReader::new(reader)?;

    std::fs::create_dir_all(output_path).map_err(UnxipError::IoError)?;

    let mut child = Command::new("cpio")
        .arg("-idm")
        .current_dir(output_path)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| UnxipError::Misc(format!("Failed to spawn cpio: {}", e)))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| UnxipError::Misc("Failed to open cpio stdin".to_string()))?;
        std::io::copy(&mut xip_reader, stdin).map_err(UnxipError::IoError)?;
    }

    let status = child.wait().map_err(UnxipError::IoError)?;
    if !status.success() {
        return Err(UnxipError::Misc(format!(
            "cpio failed with status: {}",
            status
        )));
    }
    Ok(())
}
