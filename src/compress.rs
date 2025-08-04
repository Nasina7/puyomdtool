use std::error::Error;
use std::option::Option;

enum CompressCommand {
    Run,
    Cache,
}

pub struct Compress {
    init_data: Vec<u8>,
    init_ind: usize,

    compress_buf: [Option<u8>; 0x100],
    compress_ind: usize,

    compress_buf_bak: [Option<u8>; 0x100],
    compress_ind_bak: usize,

    output_buffer: Vec<u8>,
}

impl Compress {
    pub fn run(
        input_filename: &str,
        output_filename: &str,
        check_newer: bool,
    ) -> Result<(), Box<dyn Error>> {
        if crate::check_output_newer(input_filename, output_filename, check_newer)? {
            return Ok(());
        }

        // Run Compression
        let mut compress_instance = Compress::new(input_filename)?;
        compress_instance.compress();

        // Create output directory path if it doesn't exist, and write the file.
        let path = std::path::Path::new(output_filename);
        let prefix = path
            .parent()
            .ok_or("Getting directory path of file failed!")?;
        std::fs::create_dir_all(prefix)?;
        std::fs::write(output_filename, compress_instance.output_buffer)?;
        Ok(())
    }

    fn new(input_filename: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            init_data: std::fs::read(input_filename)?,
            init_ind: 0,

            compress_buf: [Option::None; 0x100],
            compress_ind: 0,

            compress_buf_bak: [Option::None; 0x100],
            compress_ind_bak: 0,

            output_buffer: Vec::new(),
        })
    }

    // Function to read data from the decompressed file
    fn read_init(&mut self) -> u8 {
        let a = self.init_data[self.init_ind];
        self.init_ind += 1;
        a
    }

    // Writes data to the output file
    fn write_out(&mut self, val: u8) {
        self.output_buffer.push(val);
    }

    // Takes in an array and checks to see if it can be found inside the compression buffer.
    // Returns Some(index) if it is found, and None if it isn't.
    fn find_in_buf(&mut self, a: &[u8]) -> Option<i32> {
        // Do a backup of the buffer before we touch anything.  We'll be modifying it a lot on the
        // fly here, and we need to be able to revert the changes later.
        self.backup_buf();

        // Start at the current index into the compression buffer.
        let mut i: u16 = self.compress_ind_bak as u16;

        // Loop until we've looped all the way around to the original value again
        'main_loop: while i < (0x100 | self.compress_ind_bak as u16) {
            // Checks for an intended race condition where the data in the beginning of a can be
            // detected in compress_buf.
            let race_condition = i + a.len() as u16 >= (0x100 | self.compress_ind_bak as u16);

            // Reload the original buffer only if we're hitting the race condition (optimization)
            if race_condition {
                self.restore_buf();
            }

            // Loop for the length of the inputted array.
            for (pos, cur_byte) in a.iter().enumerate() {
                match self.read_buf(i as usize + pos) {
                    // If the index contains an initialized value, check to see if it's the
                    // next value we expect.  If so, update the compression buffer, and if not,
                    // continue to the next index.
                    Some(val) if val == *cur_byte => {
                        // Only do this if we're hitting the race condition (optimization)
                        if race_condition {
                            self.write_buf(val);
                        }
                    }
                    _ => {
                        // The value grabbed from the decompression buffer wasn't initialized, or
                        // wasn't the next value we expect, so go to the next index.
                        i += 1;
                        continue 'main_loop;
                    }
                }
            }

            // Restore changes to the buffer and return the index at which the inputted array was
            // first found.
            self.restore_buf();
            return Some((i & 0xFF) as i32);
        }

        // Inputted array was not found, so restore the changes to the buffer and return None.
        self.restore_buf();
        None
    }

    // Read from the compression buffer
    fn read_buf(&self, ind: usize) -> Option<u8> {
        self.compress_buf[ind & 0xFF]
    }

    // Write to the compression buffer
    fn write_buf(&mut self, val: u8) {
        self.compress_buf[self.compress_ind] = Option::Some(val);
        self.compress_ind += 1;
        self.compress_ind &= 0xFF;
    }

    // Backup the compression buffer
    fn backup_buf(&mut self) {
        self.compress_buf_bak.copy_from_slice(&self.compress_buf);
        self.compress_ind_bak = self.compress_ind;
    }

    // Restore the compression buffer
    fn restore_buf(&mut self) {
        self.compress_buf.copy_from_slice(&self.compress_buf_bak);
        self.compress_ind = self.compress_ind_bak;
    }

    // Determine what the next command in the compressed file will be.  v2 will always contain one
    // byte when the function starts.
    fn determine_next_command(&mut self, v2: &[u8]) -> CompressCommand {
        // Check to see if the data in v2 can be found in the buffer.
        let mut v3 = v2.to_vec();
        if self.find_in_buf(&v3).is_none() {
            // If the data cannot be found, do a run
            CompressCommand::Run
        } else {
            // The byte in v2 can be found, so now check to see if there is enough uncompressed
            // data left for a chche command.  (Cache commands have a minimum length of 3 bytes.)
            if self.init_ind + 2 >= self.init_data.len() {
                // Ran out of data, do a run for the rest.
                return CompressCommand::Run;
            }

            // If the data could be found, and the bounds check passed, then do an initial check to
            // see if the next two bytes along with this one are found in the table.
            v3.push(self.init_data[self.init_ind + 1]);
            v3.push(self.init_data[self.init_ind + 2]);
            if self.find_in_buf(&v3).is_some() {
                // If they can be found, end the run and initiate a normal cache command.
                CompressCommand::Cache
            } else {
                // If they cannot be found, do a run
                CompressCommand::Run
            }
        }
    }

    // Main loop to compress the data.
    pub fn compress(&mut self) {
        // Start of compression is always a run command.  Cache command is impossible to use here
        // since data in compression buffer will all be undefined.
        let mut next_command = CompressCommand::Run;

        // While there is still data left to compress
        while self.init_ind < self.init_data.len() {
            // Initialize two vectors for later usage
            let mut v: Vec<u8> = Vec::new();
            let mut v2: Vec<u8> = Vec::new();

            // Run code depending on the next command
            match next_command {
                CompressCommand::Run => {
                    loop {
                        // Push a byte into V and the buffer (at this point, this byte is confirmed
                        // not to be part of a cache command)
                        v.push(self.read_init());
                        self.write_buf(v[v.len() - 1]);

                        // Bounds check on file length and run command length
                        // (Run commands have a maximum length of 0x7F)
                        if self.init_ind >= self.init_data.len() || v.len() == 0x7F {
                            // Time to end the run.
                            break;
                        }

                        // Grab the next byte
                        let next_byte = self.init_data[self.init_ind];

                        // Check to see if the byte can match something in the table.
                        v2 = Vec::new();
                        v2.push(next_byte);

                        // If we're still doing a run command, loop.  Otherwise, break out.
                        next_command = self.determine_next_command(&v2);
                        if let CompressCommand::Run = next_command {
                            continue;
                        } else {
                            break;
                        }
                    }

                    // Write the run command to the file.
                    self.write_out(v.len() as u8);
                    for i in v.iter() {
                        self.write_out(*i);
                    }
                }
                CompressCommand::Cache => {
                    // Entering this assumes we found at least one byte in the table.

                    // First, pull one byte.
                    v.push(self.read_init());

                    // Define a bool that lets us break out of this loop without removing an extra
                    // value from the buffer.
                    let mut remove_value = true;

                    // Next, we loop until we can no longer match something in the table.
                    loop {
                        // Bounds Check
                        if self.init_ind >= self.init_data.len() {
                            // Time to end the cache.  Since we end unexpectedly, we don't want to
                            // remove a value from the array of bytes that the cache uses.
                            remove_value = false;
                            break;
                        }

                        // Load the next byte onto the cache run
                        v.push(self.read_init());

                        // If this array of values doesn't exist in the cache, break out of the
                        // loop.
                        if self.find_in_buf(&v).is_none() {
                            break;
                        }

                        // Bounds Check
                        if self.init_ind >= self.init_data.len() {
                            // Time to end the cache.
                            remove_value = false;
                            break;
                        }

                        // Cache commands have a maximum length of 0x82
                        if v.len() >= 0x82 {
                            // Time to end the cache.
                            remove_value = false;
                            break;
                        }
                    }

                    // Here, V contains one too many elements currently (if we didn't exit
                    // unexpectedly), since it still contains the incorrect byte at the end so we
                    // remove it.
                    if remove_value {
                        v.remove(v.len() - 1);
                        self.init_ind -= 1;
                    }

                    // Grab the index of the found array
                    let Some(ind) = self.find_in_buf(&v) else {
                        // Original code didn't account for this?  I don't think this can happen,
                        // need to double check.
                        panic!("Unreachable?  Open an issue on GitHub if you see this.");
                    };

                    // Next, we need to construct the command.
                    self.write_out(0x80 | (v.len() as u8 - 3)); // Command | (Length - 3)

                    // The index byte is determined by taking the original cache index and
                    // subtracting it by the found index and then also subtracting by 1.
                    self.write_out((self.compress_ind as u8 - ind as u8) - 1);

                    // Write the cache loaded bytes to the compression buffer.
                    for h in v.iter() {
                        self.write_buf(*h);
                    }

                    // Bounds check
                    if self.init_ind >= self.init_data.len() {
                        // Quit to the main loop.
                        break;
                    }

                    // The cache buffer is already updated, so all that's left to do is find the
                    // next command.
                    v2.push(self.init_data[self.init_ind]);
                    next_command = self.determine_next_command(&v2);
                }
            }
        }

        // End the file with an end command
        self.write_out(0);
    }
}
