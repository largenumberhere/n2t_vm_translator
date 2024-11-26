use std::fs::{DirEntry, File};
use std::io::Read;

use std::path::Path;

use super::parser::Parser;
use super::writer::{CodeWriter, WriterContext};
use super::{parser, transform as transformer, writer};
use std::sync::Arc;
use crate::transformer::compact_emitter::{CompactEmitter, CEmitterContext};
use crate::transformer::emit::{EContext, EmitAsm};
use crate::transformer::simple_emitter::SimpleEmitter;

pub type TransformResult<T> = Result<T, TransformError>;

#[derive(Clone, Debug)]
pub enum TransformError {
    SyntaxError(String, usize),
    IoError(String),
}

impl std::fmt::Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TransformError::SyntaxError(msg, line) => {
                write!(f, "Syntax Error: {} on line {}", msg, line)
            }
            TransformError::IoError(msg) => write!(f, "IO Error: {}", msg),
        }
    }
}

/// A type the emits hack instructions
type Emitter = super::simple_emitter::SimpleEmitter;

/// A type that encapsulates emitter state
type EmitterContext = super::simple_emitter::SContext;

pub fn transform_file(
    writer_context: WriterContext<EmitterContext>,
    in_file_path: &Path,
    out_steam: Arc<File>,
    errored: &mut bool,
    emit_init: bool,
) -> WriterContext<EmitterContext>
{
    let out_path = crate::assume_output_path(&in_file_path);

    let file_in = std::fs::File::open(&in_file_path).expect("Failed to open input file");

    println!(
        "Transforming file '{:60}'   ==>   '{}'",
        in_file_path.display(),
        out_path.display()
    );

    let result = transform::<_,Emitter>(
        writer_context.clone(),
        file_in,
        out_steam.clone(),
        emit_init,
    );
    let new_state = match result {
        Ok(s) => s,
        Err(error) => {
            eprintln!("                  ^----- '{:?}'", error.to_string());
            *errored = true;

            writer_context
        }
    };

    return new_state;
}

pub fn visit_dir_entry(
    dir: DirEntry,
    out_stream: Arc<File>,
    writer_context: WriterContext<EmitterContext>,
    translate_error: &mut bool,
    inject_init: bool,
) -> WriterContext<EmitterContext>
{
    let mut context = writer_context;

    // recursively visit all subdirectories
    if dir.file_type().unwrap().is_dir() {
        let path = dir.path();
        for d in path.read_dir().unwrap() {
            context = visit_dir_entry(
                d.unwrap(),
                out_stream.clone(),
                context,
                translate_error,
                inject_init,
            );
        }
    } else {
        let path = dir.path();
        if path.extension() == Some("vm".as_ref()) {
            context = transformer::transform_file(
                context,
                path.as_path(),
                out_stream,
                translate_error,
                inject_init,
            );
        }
    }

    context
}

pub fn traverse_directories(
    path: &Path,
    translate_error: &mut bool,
    out_stream: Arc<File>,
    emit_init: bool,
    writer_context: WriterContext<EmitterContext>,
)
{
    let mut context = writer_context;
    // let out_path = assume_output_path(path);
    for entry in std::fs::read_dir(path).unwrap() {
        context = visit_dir_entry(
            entry.unwrap(),
            out_stream.clone(),
            context,
            translate_error,
            emit_init,
        );
    }
}
fn transform<R: Read, E>
(
    writer_context: WriterContext<EmitterContext>,
    in_stream: R,
    out_stream: Arc<File>,
    emit_init: bool,
) -> Result<WriterContext<EmitterContext>, TransformError>
        where E: EmitAsm<EmitterContext>
{
    let mut reader: Parser = parser::Parser::new(in_stream);
    let mut writer: CodeWriter<EmitterContext, Emitter> =
        writer::CodeWriter::with_context(writer_context, out_stream, emit_init);

    while let Some(val) = reader.next_command() {
        let (command, line) = val?;

        writer.write_command(&command, &line);
    }

    return Ok(writer.close());
}
