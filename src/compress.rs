use std::collections::VecDeque;
use std::error::Error;

pub struct Compress {
    init_data: Vec<u8>,
    init_ind: usize,

    compression_buf: [Option<u8>; 0x100],
    compression_ind: usize,

    compression_buf_bak: [Option<u8>; 0x100],
    compression_ind_bak: usize,

    output_buffer: Vec<u8>,
    output_name: String,
}

impl Compress {
    pub fn run(input_filename: &str, output_filename: &str) -> Result<(), Box<dyn Error>> {
        let compress_instance = Compress::new(input_filename, output_filename)?;

        Ok(())
    }

    pub fn new(input_filename: &str, output_filename: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            init_data: std::fs::read(input_filename)?,
            init_ind: 0,

            compression_buf: [Option::None; 0x100],
            compression_ind: 0,

            compression_buf_bak: [Option::None; 0x100],
            compression_ind_bak: 0,

            output_buffer: Vec::new(),
            output_name: output_filename.to_string(),
        })
    }
}
