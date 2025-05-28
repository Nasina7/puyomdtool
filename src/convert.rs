use std::error::Error;
use std::option::Option;

enum MappingType {
    Byte,
    Pal,
    Word,
}

struct Mapping {
    filename: String,

    map_data: Vec<u8>,
    pal_data: Vec<u8>, // Only used by Pal MappingType.

    pal_queue: u8,
    pal_queue_ind: usize,

    map_type: MappingType,
    common_word: u16,

    read_index: usize,
}

impl Mapping {
    pub fn new(filename: &str, common_word: u16, is_output: bool) -> Result<Self, Box<dyn Error>> {
        let map_type = if filename.ends_with("bgbyte") {
            MappingType::Byte
        } else if filename.ends_with("bgpalm") {
            MappingType::Pal
        } else if filename.ends_with("bgword") {
            MappingType::Word
        } else {
            return Err(Box::new(super::PMDTError::UnknownMappingType));
        };

        let mapping = if !is_output {
            let mut palp_string = String::from(filename);
            palp_string.pop();
            palp_string.push('p');
            Mapping {
                filename: filename.to_string(),
                map_data: std::fs::read(filename)?,
                pal_data: if let MappingType::Pal = map_type {
                    std::fs::read(palp_string)?
                } else {
                    Vec::new()
                },
                pal_queue: 0,
                pal_queue_ind: 0,
                map_type,
                common_word,
                read_index: 0,
            }
        } else {
            Mapping {
                filename: filename.to_string(),
                map_data: Vec::new(),
                pal_data: Vec::new(),
                pal_queue: 0,
                pal_queue_ind: 0,
                map_type,
                common_word,
                read_index: 0,
            }
        };

        // Do some validity checks.  If the mapping is an output, these checks will still pass.
        if let MappingType::Pal = mapping.map_type {
            if mapping.map_data.len() != mapping.pal_data.len() * 4 {
                return Err(Box::new(super::PMDTError::WrongMappingSize));
            }
        }
        if let MappingType::Word = mapping.map_type {
            if mapping.map_data.len() & 0x1 != 0 {
                return Err(Box::new(super::PMDTError::WrongMappingSize));
            }
        }

        Ok(mapping)
    }

    // Read a tile from the mapping file.  If there are none left, return None.
    pub fn read(&mut self) -> Option<u16> {
        if self.read_index >= self.map_data.len() {
            None
        } else {
            Some(match self.map_type {
                MappingType::Byte => {
                    self.read_index += 1;
                    self.common_word | self.map_data[self.read_index - 1] as u16
                }
                MappingType::Word => {
                    self.read_index += 2;
                    (self.map_data[self.read_index - 2] as u16) << 8
                        | (self.map_data[self.read_index - 1] as u16)
                }
                MappingType::Pal => {
                    self.read_index += 1;
                    let pal = (self.pal_data[(self.read_index - 1) / 4]) as u16
                        >> (2 * (self.read_index & 0x3));

                    (pal << 13) | self.common_word | (self.map_data[self.read_index - 1] as u16)
                }
            })
        }
    }

    // Write a tile to the output mapping file.
    pub fn write(&mut self, val: u16) {
        match self.map_type {
            MappingType::Byte => {
                self.map_data.push(val as u8);
            }
            MappingType::Word => {
                self.map_data.push((val >> 8) as u8);
                self.map_data.push(val as u8);
            }
            MappingType::Pal => {
                self.map_data.push(val as u8);
                // palp tile writes are 2 bits long, need to account for this.
                self.pal_queue |= (((val & 0x6000) >> 13) as u8) << ((3 - self.pal_queue_ind) * 2);
                self.pal_queue_ind += 1;
                if self.pal_queue_ind == 4 {
                    self.pal_queue_ind = 0;
                    self.pal_data.push(self.pal_queue);
                    self.pal_queue = 0;
                }
            }
        }
    }

    // Save the background mapping file.
    pub fn save(&mut self) -> Result<(), Box<dyn Error>> {
        std::fs::write(&self.filename, &self.map_data)?;
        if let MappingType::Pal = self.map_type {
            let mut palp_string = String::from(&self.filename);
            palp_string.pop();
            palp_string.push('p');

            // Make sure we write any remaining data in the pal queue
            if self.pal_queue_ind != 0 {
                self.pal_data.push(self.pal_queue);
            }

            std::fs::write(palp_string, &self.pal_data)?;
        }

        Ok(())
    }
}

pub struct Convert {
    input_mapping: Mapping,
    output_mapping: Mapping,
}

impl Convert {
    pub fn run(
        input_filename: &str,
        output_filename: &str,
        common_word: u16,
    ) -> Result<(), Box<dyn Error>> {
        let mut convert_instance = Convert::new(input_filename, output_filename, common_word)?;
        convert_instance.convert();
        convert_instance.output_mapping.save()?;
        Ok(())
    }

    fn new(
        input_filename: &str,
        output_filename: &str,
        common_word: u16,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            input_mapping: Mapping::new(input_filename, common_word, false)?,
            output_mapping: Mapping::new(output_filename, 0, true)?,
        })
    }

    fn convert(&mut self) {
        while let Some(val) = self.input_mapping.read() {
            self.output_mapping.write(val);
        }
    }
}
