use std::ffi::c_long;
use std::fs::{DirEntry, File};
use std::io::{Read, Write};
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use crate::parser::Parser;
use crate::{assume_output_path, parser, transformer, writer};
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

pub fn transform_file(in_file_path: &Path, out_steam: Arc<File>, errored: &mut bool, emit_init: bool) {
    let out_path = crate::assume_output_path(&in_file_path);

    let file_in = std::fs::File::open(&in_file_path)
        .expect("Failed to open input file");

    println!("Transforming file '{:60}'   ==>   '{}'", in_file_path.display(), out_path.display());

    let result = transform(file_in, out_steam.clone(), emit_init);
    match result {
        Ok(_) => {},
        Err(error) => {
            eprintln!("                  ^----- '{:?}'", error.to_string());
            *errored = true;
        }
    }
}

pub fn visit_dir_entry(dir: DirEntry, out_stream: Arc<File>, translate_error: &mut bool, inject_init: bool) {
    // recursively visit all subdirectories
    if dir.file_type().unwrap().is_dir() {
        let path = dir.path();
        for d in path.read_dir().unwrap() {
            visit_dir_entry(d.unwrap(), out_stream.clone(), translate_error, inject_init);
        }
    } else {
        let path = dir.path();
        if path.extension() == Some("vm".as_ref()) {
            transformer::transform_file(path.as_path(), out_stream, translate_error, inject_init);
        }
    }
}




pub fn traverse_directories(path: &Path, translate_error: &mut bool, out_stream: Arc<File>, emit_init: bool) {
    let out_path = assume_output_path(path);
    for entry in std::fs::read_dir(path).unwrap()
    {
        visit_dir_entry(entry.unwrap(), out_stream.clone(), translate_error, emit_init);
    }
}
fn transform<R: Read>(in_stream: R, out_stream: Arc<File>, emit_init: bool) -> Result<(), TransformError> {

    let mut reader: Parser = parser::Parser::new(in_stream);
    let mut writer: CodeWriter= writer::CodeWriter::new(out_stream, emit_init);

    while let Some(val) = reader.next_command() {
         let (command, line) = val?;

        writer.write_command(&command, &line);
    }

    return Ok(());
}