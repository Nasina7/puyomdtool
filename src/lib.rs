pub mod compress;
pub mod convert;
pub mod decompress;
pub mod fix_checksum;

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
