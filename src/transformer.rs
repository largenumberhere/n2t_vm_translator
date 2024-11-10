use std::io::{Read, Write};
use std::path::Path;
use crate::parser::Parser;
use crate::{parser, writer};
use crate::writer::CodeWriter;

pub type TransformResult<T> = Result<T, TransformError> ;

#[derive(Clone, Debug)]
pub enum TransformError {
    SyntaxError(String, usize),
    IoError(String)
}

impl std::fmt::Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TransformError::SyntaxError(msg, line) => write!(f, "Syntax Error: {} on line {}", msg, line),
            TransformError::IoError(msg) => write!(f, "IO Error: {}", msg)
        }
    }
}

pub fn transform_file(in_file_path: &Path, errored: &mut bool) {
    let out_path = crate::assume_output_path(&in_file_path);

    let file_out = std::fs::File::create(&out_path)
        .expect("Failed to open output file");

    let file_in = std::fs::File::open(&in_file_path)
        .expect("Failed to open input file");

    println!("Transforming file '{:60}'   ==>   '{}'", in_file_path.display(), out_path.display());

    let result = transform(file_in, file_out);
    match result {
        Ok(_) => {},
        Err(error) => {
            eprintln!("                  ^----- '{:?}'", error.to_string());
            *errored = true;
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