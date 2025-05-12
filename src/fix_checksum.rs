use super::PMDTError;
use std::error::Error;

pub struct FixChecksum {}

impl FixChecksum {
    pub fn run(input_filename: &str, output_filename: &str) -> Result<(), Box<dyn Error>> {
        let mut rom = std::fs::read(input_filename)?;
        if rom.len() <= 0x201 {
            return Err(Box::new(PMDTError::InvalidRomSize));
        }

        let word_length = (rom.len() - 0x200) / 2;

        let checksum = ((rom[0x18E] as u16) << 8) | rom[0x18F] as u16;
        let mut calc_checksum: u16 = 0;

        for index in 0..word_length {
            calc_checksum += ((rom[0x200 + index * 2] as u16) << 8) | rom[0x201 + index * 2] as u16;
        }

        if checksum == calc_checksum {
            println!("[MSG] Checksum is correct, nothing to do.");
        } else {
            println!(
                "[MSG] Checksum is {:04X}, changing to {:04X}",
                checksum, calc_checksum
            );
            rom[0x18E] = (calc_checksum >> 8) as u8;
            rom[0x18F] = calc_checksum as u8;
            std::fs::write(output_filename, rom)?;
        }

        Ok(())
    }
}
