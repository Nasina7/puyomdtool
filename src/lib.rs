pub mod compress;
pub mod decompress;
pub mod fix_checksum;

#[derive(Debug)]
pub enum PMDTError {
    InvalidNumOfArguments,
    InvalidRomSize,
}

impl std::fmt::Display for PMDTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for PMDTError {}

// Todo: Remove this
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
