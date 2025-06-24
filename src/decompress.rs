use std::error::Error;

pub struct Decompress {
    init_data: Vec<u8>,
    init_ind: usize,

    output_pre_buf: Vec<u8>,
    output_pre_ind: usize,
    output_pre_disable: bool,
    output_buffer: Vec<u8>,

    decompress_buf: Vec<u8>,
    decompress_ind: u8,

    cmd: u32,
}

impl Decompress {
    pub fn run(
        input_filename: &str,
        output_filename: &str,
        nobuf: bool,
    ) -> Result<(), Box<dyn Error>> {
        let mut decompress_instance = Decompress::new(input_filename, nobuf)?;
        decompress_instance.decompress();
        std::fs::write(output_filename, decompress_instance.output_buffer)?;
        Ok(())
    }

    fn new(input_filename: &str, nobuf: bool) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            init_data: std::fs::read(input_filename)?,
            init_ind: 0,

            output_pre_buf: vec![0; 0x4],
            output_pre_ind: 0,
            output_pre_disable: nobuf,
            output_buffer: Vec::new(),

            decompress_buf: vec![0; 0x100],
            decompress_ind: 0,

            cmd: 0,
        })
    }

    // Function to read data from the compressed file
    fn read_init(&mut self) -> u8 {
        let a = self.init_data[self.init_ind];
        self.init_ind += 1;
        a
    }

    // Writes to the decompression buffer
    fn write_buf(&mut self, val: u8) {
        self.decompress_buf[self.decompress_ind as usize] = val;
        self.decompress_ind += 1;
    }

    // Handles the 4-byte buffer and writes data to the output file.
    fn write_out(&mut self, val: u8) {
        if !self.output_pre_disable {
            // Write to a temporary 4-byte buffer and increment it's index
            self.output_pre_buf[self.output_pre_ind] = val;
            self.output_pre_ind += 1;

            // If the buffer is full, output it.
            if self.output_pre_ind == 4 {
                self.output_pre_ind = 0;
                for byte in self.output_pre_buf.iter() {
                    self.output_buffer.push(*byte);
                }
            }
        } else {
            self.output_buffer.push(val);
        }
    }

    // This command will load a series of bytes following the command byte.
    fn cmd_run(&mut self) {
        // Get the length of the run command from the command byte.
        self.cmd &= 0x007F;
        self.cmd -= 1;

        // Loop until we reach the end of the command.
        loop {
            // Grab a byte of data from the compressed data
            let data = self.read_init();

            // Write it out
            self.write_out(data);

            // Write the grabbed byte to the decompression buffer
            self.write_buf(data);

            // Decrease the length, and check if the command is done.
            self.cmd -= 1;
            if self.cmd == 0xFFFFFFFF {
                break;
            }
        }
    }

    // This command will load data from the decompression buffer, rather than the compressed file.
    fn cmd_cache(&mut self) {
        // Grab the number of bytes to load from the cache.
        self.cmd &= 0x007F;
        self.cmd += 2;

        // The second byte of the command determines where the data will begin being loaded from the buffer.
        // This requires a bit of calculation..
        let mut decompress_calcind = self.decompress_ind;
        decompress_calcind -= self.read_init();
        decompress_calcind -= 1;

        // Load data from the buffer until the length of the command runs out
        loop {
            // Load from the decompression buffer
            let data = self.decompress_buf[decompress_calcind as usize];

            // Write it out
            self.write_out(data);

            // Update the decompression buffer with the newly loaded data
            self.write_buf(data);

            // Increase the index we load from, decrease the length of the command, and if we're done, break out of the command.
            decompress_calcind += 1;
            self.cmd -= 1;
            if self.cmd == 0xFFFFFFFF {
                break;
            }
        }
    }

    pub fn decompress(&mut self) {
        loop {
            // Get the current command from the compressed data
            self.cmd = self.read_init() as u32;

            // If the command's highest bit is set, then it is a cache command, otherwise, it's a run command.
            // If the command is 0x00, then we've reached the end of the file, so we break out of the loop.
            if (self.cmd & 0x80) != 0 {
                self.cmd_cache();
            } else if (self.cmd & 0xFF) != 0 {
                self.cmd_run();
            } else {
                if self.output_pre_ind != 0 {
                    println!(
                        "[WARN] {} byte(s) discarded when decompressed!",
                        self.output_pre_ind
                    );
                }
                break;
            }
        }
    }
}
