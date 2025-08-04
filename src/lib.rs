use std::error::Error;

pub mod compress;
pub mod convert;
pub mod decompress;
pub mod fix_checksum;

// Checks if output_filename is newer than input_filename.
fn check_output_newer(
    input_filename: &str,
    output_filename: &str,
    check_newer: bool,
) -> Result<bool, Box<dyn Error>> {
    if check_newer && std::fs::exists(output_filename)? {
        // Get Metadata
        let input_file_meta = std::fs::metadata(input_filename)?;
        let output_file_meta = std::fs::metadata(output_filename)?;

        // Check if the output file is newer.
        if input_file_meta.modified().is_ok() {
            let input_file_time = input_file_meta.modified()?;
            let output_file_time = output_file_meta.modified()?;
            if output_file_time > input_file_time {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

#[derive(Debug)]
pub enum PMDTError {
    InvalidNumOfArguments,
    InvalidRomSize,
    UnknownMappingType,
    WrongMappingSize,
}

impl std::fmt::Display for PMDTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for PMDTError {}
