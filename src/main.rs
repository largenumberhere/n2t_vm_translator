use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use rusty_parser::str;

mod parser;
mod writer;
mod emit_asm;

use parser::Parser;
use crate::writer::CodeWriter;

fn main() {
    let mut args = std::env::args();
    let _ = args.next();
    let arg1 = args.next().unwrap();
    // println!("{:?}", arg1);

    let path = Path::new(&arg1);
    let ext = path.extension().unwrap_or_default().to_str().unwrap();
    let rest = path.to_str().unwrap();

    let mut new_str = String::new();
    for c in rest.chars() {
        new_str.push(c);
    }

    // discard any file extension
    if ext.len() > 0 {
        for _ in 0.. ext.chars().count()+1 {
            _ = new_str.pop()
        }
    }

    // add .asm extenions
    new_str.push_str(".asm");
    let out_path = new_str;

    let file_out = std::fs::File::create(&out_path).expect("Failed to open output file");
    println!("{}", out_path);
    let file_in = std::fs::File::open(&arg1).expect("Failed to open input file");
    let mut reader = parser::Parser::new(file_in);
    let mut writer: CodeWriter<File> = writer::CodeWriter::new(file_out);

    while let Some((command, line)) = reader.next_command() {

        //println!("command: {:?} was {}", command, line);
        writer.write_command(&command, &line);
    }

}