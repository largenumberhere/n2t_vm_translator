#![allow(unused)]

use crate::transform::transform_directory;
use crate::transformer::writer::WriterContext;
use crate::transform::transform_file;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;

mod transformer;
use transformer::{transform};

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

    let inject_init = match args.next().as_deref() {
        Some("--init") => true,
        _ => false,
    };


    let context = WriterContext::default();
    let path = Path::new(&arg1);
    let out_path = assume_output_path(path);
    let out_steam = Arc::new(std::fs::File::create(&out_path).unwrap());
    if path.is_dir() {
        transform_directory(path, &mut translate_error, out_steam, inject_init, context, out_path.clone());
    } else {
        transform_file(context, path, out_steam, &mut translate_error, inject_init, out_path.clone());
    }

    if translate_error {
        std::process::exit(1);
    }
}

// replace/append file extension with .asm in a path
fn assume_output_path(input_path: &Path) -> PathBuf {
    let mut path = PathBuf::from(input_path);

    if path.is_dir() {
        // replace folder/file.vm with folder/folder.asm
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        path.pop(); //
        path.push(file_name.as_str()); // /folder/
        path.push(file_name); // /folder/
        path.set_extension("asm"); // /folder/folder.asm
    } else {
        // replace folder/file.vm with folder/file.asm
        path.set_extension("asm");
    }

    return path;
}
