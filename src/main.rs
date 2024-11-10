use std::fs::{DirEntry, File, FileType};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use rusty_parser::str;

mod parser;
mod writer;
mod emit_asm;

use parser::Parser;
use crate::writer::CodeWriter;

fn visit_dir_entry(dir: DirEntry) {
    // recursively visit all subdirectories
    if dir.file_type().unwrap().is_dir() {
        let path = dir.path();
        for d in path.read_dir().unwrap() {
            visit_dir_entry(d.unwrap());
        }
    } else {
        let path = dir.path();
        if path.extension() == Some("vm".as_ref()) {
            transform_file(path.as_path());
        }
    }
}

fn traverse_directories(path: &Path) {
    for entry in std::fs::read_dir(path).unwrap()
    {
        visit_dir_entry(entry.unwrap());
    }
}

fn main() {
    let mut args = std::env::args();
    let arg1 = args.skip(1).next()
        .expect("Please provide a file or folder to translate");

    let path = Path::new(&arg1);
    if path.is_dir() {
        traverse_directories(path);
    } else {
        transform_file(path);
    }
}

fn transform_file(in_file_path: &Path) {
    println!("tranforming {}", in_file_path.display());
    let out_path = assume_output_path(&in_file_path);

    let file_out = std::fs::File::create(&out_path)
        .expect("Failed to open output file");

    let file_in = std::fs::File::open(&in_file_path)
        .expect("Failed to open input file");
    println!("{:?} -> {:?}", in_file_path, out_path);

    let result = transform(file_in, file_out);
    match result {
        Ok(_) => {},
        Err(error) => {
            eprintln!("Failed to transform file '{:?}' because '{}'", in_file_path, error.to_string());
        }
    }
}

// replace the file extension in a path with .asm
fn assume_output_path(input_path: &Path) -> PathBuf {
    // let path = Path::fro(input_path);
    let path = input_path;
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

    return PathBuf::from(new_str);
}

type TransformResult<T> = Result<T, TransformError> ;
#[derive(Clone, Debug)]
enum TransformError {
    SyntaxError(String),
    IoError(String)
}
impl std::fmt::Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TransformError::SyntaxError(msg) => write!(f, "IO Error:{}", msg),
            TransformError::IoError(msg) => write!(f, "IO Error: {}", msg)
        }
    }
}

fn transform<R: Read, W: Write>(in_stream: R, out_stream: W) -> Result<(), TransformError> {

    let mut reader: Parser = parser::Parser::new(in_stream);
    let mut writer: CodeWriter<W> = writer::CodeWriter::new(out_stream);

    while let Some(val) = reader.next_command() {
         let (command, line) = val?;

        writer.write_command(&command, &line);
    }

    return Ok(());
}