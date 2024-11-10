use std::fs::DirEntry;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
mod parser;
mod writer;
mod emit_asm;
mod transformer;

fn visit_dir_entry(dir: DirEntry, translate_error: &mut bool) {
    // recursively visit all subdirectories
    if dir.file_type().unwrap().is_dir() {
        let path = dir.path();
        for d in path.read_dir().unwrap() {
            visit_dir_entry(d.unwrap(), translate_error);
        }
    } else {
        let path = dir.path();
        if path.extension() == Some("vm".as_ref()) {
            transformer::transform_file(path.as_path(), translate_error);
        }
    }
}

fn traverse_directories(path: &Path, translate_error: &mut bool) {
    for entry in std::fs::read_dir(path).unwrap()
    {
        visit_dir_entry(entry.unwrap(), translate_error);
    }
}

fn main() {
    let mut translate_error = false;
    let mut args = std::env::args();

    let arg1 = match args.skip(1).next() {
        Some(arg) => arg,
        None => {
            eprintln!("A file or folder must be supplied as the first argument.");
            exit(1);
        }
    };

    let path = Path::new(&arg1);
    if path.is_dir() {
        traverse_directories(path, &mut translate_error);
    } else {
        transformer::transform_file(path, &mut translate_error);
    }

    if translate_error {
        std::process::exit(1);
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




