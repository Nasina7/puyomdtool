use puyomdtool::{
    PMDTError, compress::Compress, convert::Convert, decompress::Decompress,
    fix_checksum::FixChecksum,
};
use std::{env, error::Error};

fn print_help() {
    println!("=== puyomdtool by Nasina");
    println!();
    println!("Each command can be prefixed with \"ifnewer\" to have the tool skip operation if");
    println!("the output file is newer than the input file (if it even exists).  This does not");
    println!("apply to the fix command.  This is only meant to be used in build systems to speed");
    println!("up build times.");
    println!();
    println!("Usage 1: puyomdtool fix src_file.bin dst_file.bin");
    println!("  - This will fix the checksum of any Megadrive rom passed to it.");
    println!();
    println!("Usage 2: puyomdtool [compress|decompress(nobuf)] src_file.bin dst_file.bin");
    println!("  - This will compress / decompress src_file.bin and save it as dst_file.bin");
    println!("  - decompressnobuf disables an intended(?) part of the decompression that can lead");
    println!("    to data being discarded.");
    println!();
    println!("Usage 3: puyomdtool convert common_word src_file.ext dst_file.ext");
    println!("  - Converts between bgmap types.  Type will be inferred using the file extension.");
    println!("  - If you are using the bgpal type, specify the bgpalm file.");
    println!("    bgpalp will be created or obtained automatically.");
    println!("  - When converting from a smaller data format to a larger format ");
    println!("    (bgbyte -> bgpal/bgword or bgbyte/bgpal -> bgword), common_word is used as an ");
    println!("    OR value (Ex: byte | common_word -> word).  When converting to a smaller format");
    println!("    this value should be set to zero.");
    println!();
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().collect();

    // This flag will make it so that the operation will only happen if
    // the input file is newer than the output file (or if the output
    // file doesn't exist.)  This is mainly meant for improving disassembly
    // build times.
    let check_newer = if args.len() >= 2 && args[1].as_str() == "ifnewer" {
        args.remove(1);
        true
    } else {
        false
    };

    let result: Result<(), Box<dyn Error>> = if args.len() < 2 {
        print_help();
        Ok(())
    } else {
        match args[1].as_str() {
            "compress" if args.len() == 3 => Compress::run(&args[2], &args[2], false),
            "compress" if args.len() == 4 => Compress::run(&args[2], &args[3], check_newer),
            "compress" => Err(Box::new(PMDTError::InvalidNumOfArguments)),
            "convert" if args.len() == 5 => Convert::run(
                &args[3],
                &args[4],
                check_newer,
                u16::from_str_radix(&args[2], 16)?,
            ),
            "convert" => Err(Box::new(PMDTError::InvalidNumOfArguments)),
            "decompress" if args.len() == 3 => Decompress::run(&args[2], &args[2], false, false),
            "decompress" if args.len() == 4 => {
                Decompress::run(&args[2], &args[3], check_newer, false)
            }
            "decompress" => Err(Box::new(PMDTError::InvalidNumOfArguments)),
            "decompressnobuf" if args.len() == 3 => {
                Decompress::run(&args[2], &args[2], false, true)
            }
            "decompressnobuf" if args.len() == 4 => {
                Decompress::run(&args[2], &args[3], check_newer, true)
            }
            "decompressnobuf" => Err(Box::new(PMDTError::InvalidNumOfArguments)),
            "fix" if args.len() == 3 => FixChecksum::run(&args[2], &args[2]),
            "fix" if args.len() == 4 => FixChecksum::run(&args[2], &args[3]),
            "fix" => Err(Box::new(PMDTError::InvalidNumOfArguments)),
            _ => Ok(()),
        }
    };

    if let Err(ref e) = result {
        println!("Hit an Error: {e}\n");
    }

    result
}
