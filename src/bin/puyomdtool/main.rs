use puyomdtool::compress::Compress;
use std::{env, error::Error};

fn print_help() {
    println!("--puyomdtool by Nasina--");
    println!();
    println!("Usage 1: puyomdtool build");
    println!("  - This will build puyodisasm");
    println!();
    println!("Usage 2: puyomdtool fix src_file.bin dst_file.bin");
    println!("  - This will fix the header of any Megadrive rom passed to it.");
    println!();
    println!("Usage 3: puyomdtool [compress|decompress] src_file.bin dst_file.bin");
    println!("  - This will compress / decompress src_file.bin and save it as dst_file.bin");
    println!();
    println!("For more advanced usages, pass --help_advanced.");
}

fn print_help_advanced() {
    println!("--puyomdtool by Nasina--");
    println!();
    println!("Usage 4: puyomdtool convert src_file.ext dst_file.ext");
    println!("  - Converts between bgmap types.  Type will be inferred using the file extension.");
    println!("  - If you are using the bgpal type, specify the bgpalm file.");
    println!("    bgpalp will be created or obtained automatically.");
    println!();
    println!(
        "Usage 5: puyomdtool combine [horizontal|vertical] sizec size1 size2 src_file_1.bgword src_file_2.bgword dst_file.bgword"
    );
    println!("  - Combines two background mappings to make a larger background mapping.");
    println!(
        "  - sizec = Common Size.  If combining horizontally, this is the mapping height and vice versa"
    );
    println!(
        "  - size1 / size2 = Different Size.  If combining horizontally, this is src_file_1's width and src_file_2's width."
    );
    println!();
    println!(
        "Usage 6: puyomdtool split [horizontal|vertical] sizec size1 size2 src_file.bgword dst_file_1.bgword dst_file_2.bgword"
    );
    println!("  - Splits a background mapping into two background mappings.");
    println!(
        "  - sizec = Common Size.  If splitting horizontally, this is the mapping height and vice versa"
    );
    println!(
        "  - size1 / size2 = Different Size.  If splitting horizontally, this is dst_file_1's width and dst_file_2's width."
    );
    println!();
    println!("Usage 7: puyomdtool parse base.asm parsed.asm");
    println!(
        "  - This will parse an assembly file (or any text file), run commands, and output the result."
    );
    println!("  - For more information on what this means, check the wiki on GitHub.");
    println!();
}

#[derive(Debug)]
enum PMDTError {
    InvalidNumOfArguments,
}

impl std::fmt::Display for PMDTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for PMDTError {}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    #[allow(clippy::unit_arg)]
    let result: Result<(), Box<dyn Error>> = if args.len() < 2 {
        Ok(print_help())
    } else {
        match args[1].as_str() {
            "--help_advanced" => Ok(print_help_advanced()),
            "compress" if args.len() == 3 => Compress::run(&args[2], &args[2]),
            "compress" if args.len() == 4 => Compress::run(&args[2], &args[3]),
            "compress" => Err(Box::new(PMDTError::InvalidNumOfArguments)),
            _ => Ok(()),
        }
    };

    if let Err(ref e) = result {
        println!("Hit an Error: {e}\n");
    }

    result
}
