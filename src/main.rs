use std::fs::DirEntry;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;

mod parser;
mod writer;
mod emit_asm;
mod transformer;



use crate::transformer::traverse_directories;
use crate::writer::WriterContext;

fn main() {

    let mut translate_error = false;
    let mut args = std::env::args().skip(1);

    let arg1 = match args.next() {
        Some(arg) => arg,
        None => {
            eprintln!("A file or folder must be supplied as the first argument.");
            exit(1);
        }
    };

    let inject_init = match  args.next().as_deref() {
        Some("--init") => true,
        _ => false
    };


    let context = WriterContext::default();
    let path = Path::new(&arg1);
    let out_path = assume_output_path(path);
    let out_steam = Arc::new(std::fs::File::create(&out_path).unwrap());
    if path.is_dir() {
        traverse_directories(path, &mut translate_error, out_steam, inject_init, context);
    } else {
        transformer::transform_file(context, path, out_steam , &mut translate_error, inject_init);
    }

    if translate_error {
        std::process::exit(1);
    }
}

// replace the file extension in a path with .asm
fn assume_output_path(input_path: &Path) -> PathBuf {
    // let path = Path::fro(input_path);
    let mut path = PathBuf::from(input_path);
    // let ext = path.extension().unwrap_or_default().to_str().unwrap();
    // let rest = path.to_str().unwrap();

    if path.is_dir() {
        // replace folder/file.vm with folder/folder.asm
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        path.pop();     //
        path.push(file_name.as_str());   // /folder/
        path.push(file_name);   // /folder/
        path.set_extension("asm");               // /folder/folder.asm

    }
    else {
        // replace folder/file.vm with folder/file.asm
        path.set_extension("asm");
    }

    return path;
}




