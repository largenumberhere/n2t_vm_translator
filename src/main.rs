use std::io::Write;
mod parser;
mod writer;
use parser::Parser;
fn main() {
    let mut args = std::env::args();
    let _ = args.next();
    let file_path = args.next().expect("Please specify a file path");
    let file = std::fs::read_to_string(file_path).expect("Failed to open file");
    
    let mut out = std::fs::File::create("out.txt").unwrap();

    let lines =  file.lines();
    let mut line_counter = 0;
    for line in lines {
        line_counter +=1;
        
        let rest = skip_whitespace(&line);
        let (rest, command) = read_contiguous(rest);
        match command.as_str() {
            "" => {
                // ignore empty lines
                println!("igorning blank line");
                out.write_fmt(format_args!("//{}:\t {}\n", line_counter, line)).unwrap();
                continue;
            }
            "//" => {
                // ignore comments
                println!("igornign commments");
                out.write_fmt(format_args!("//{}:\t {}\n", line_counter, line)).unwrap();
                continue;
            }
            "push" => {
                let rest = skip_whitespace(rest);
                let (rest, type_) = read_contiguous(rest);
                match type_.as_str() {
                    "constant" => {
                        let rest = skip_whitespace(rest);
                        let (rest, val) = read_u16(rest);
                        println!("pushing constant {}\n", val);
                        out.write_fmt(format_args!("//{}:\t {}\n", line_counter, line)).unwrap();
                        // out.write_fmt()
                    }
                    _=> {
                        println!("line={} rest='{}'", line, rest);
                        unimplemented!()}
                }
            },
            "add" => {
                println!("adding");
            }
            _=> {unimplemented!();}
        }
    }
    


}

fn read_contiguous(line: &str) -> (&str, String) {
    let mut cursor = line.chars();
    let mut ahead_cursor = line.chars();

    let mut string = String::new();
    loop {
        let next = ahead_cursor.next();
        match next {
            Some(v) => {
                if v.is_whitespace() {
                    break;
                }

                let more = cursor.next().unwrap();
                string.push(more);
            }
            None => {
                break;
            }
        }

    }

    return (cursor.as_str(), string);
}

fn read_u16(line: &str) -> (&str, u16) {
    let (rest,  block) = read_contiguous(line);
    if block.is_empty() {
        panic!("no number found");
    }

    let num = block.parse::<u16>().expect("Failed to parse u16");
    
    (rest, num)
    
}

fn skip_whitespace(line: &str) -> &str{
    let mut cursor = line.chars();
    let mut ahead_cursor = cursor.clone();
    loop {
        let next = ahead_cursor.next();
        if let Some(v) = next {
            if v.is_whitespace() {
                let _ = cursor.next();
                continue;
            }
        }

        break;
    }

    return cursor.as_str();
}