use std::fs::File;
use std::io::Write;
mod parser;
mod writer;
mod emit_asm;

use parser::Parser;
use crate::writer::CodeWriter;

fn main() {
    let mut args = std::env::args();
    let _ = args.next();
    let file_in = std::fs::File::open(args.next().expect("Please specify an input file path")).expect("Failed to open input file");
    let file_out = std::fs::File::create(args.next().expect("Please specify output file")).expect("Failed to open output file");
    
    let mut reader = parser::Parser::new(file_in);
    let mut writer: CodeWriter<File> = writer::CodeWriter::new(file_out);

    while let Some((command, line)) = reader.next_command() {

        //println!("command: {:?} was {}", command, line);
        writer.write_command(&command, &line);
    }

}