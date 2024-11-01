use std::{fs::File, io::{BufRead, BufReader, Lines, Read}, iter::Peekable};

// inspired by https://depth-first.com/articles/2021/12/16/a-beginners-guide-to-parsing-in-rust/
struct Scanner {
    cursor: usize,
    characters: Vec<char>
}

impl Scanner {
    fn new(characters: Vec<char>) -> Scanner {
        Scanner {
            characters,
            cursor: 0
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn peek(&self) -> Option<&char> {
        self.characters.get(self.cursor)
    }

    pub fn peek_n(&self, len: usize) -> Option<String> {
        let mut string = String::new();
        for i in 0..len {
            let c = self.characters.get(self.cursor+i);
            match c {
                Some(v) => {
                    string.push(*v);
                } 
                _ => {
                    return None;
                }
            }
        }

        Some(string)
    }

    pub fn is_done(&self) -> bool {
        self.cursor == self.characters.len()
    }

    pub fn pop(&mut self) -> Option<&char> {
        let current = self.characters.get(self.cursor);
        if let Some(v) =current {
            self.cursor+=1;
            Some(v)
        } else {
            return None;
        }
    }

    pub fn discard_n(&mut self, n: usize) {
        self.cursor+=n;
    }
    
    pub fn take(&mut self, target: &char) -> bool {
        let current = self.characters.get(self.cursor);
        if Some(target) == current {
            self.cursor+=1;
            return true;
        } else {
            return false;
        }
    }

    // pub fn transform<T>(&mut self, callback : impl FnOnce(&char)->Option<T> ) -> Option<T>  {
    //     let current = self.characters.get(self.cursor)?;
    //     let ret = callback(current);
    //     if let Some(t) = ret {
    //         self.cursor+=1;
    //         Some(t)
    //     } else {
    //         None
    //     }
    // }
}


pub struct Parser {
    reader: String,
    string_pos: usize,
    line_tally: usize,
    // line_pos: usize
}


#[derive(Debug)]
pub enum  CommandType {
    CArithmeticPush,
    CPush,
    CPop,
    CLabel,
    CGoto,
    CIf,
    CFunction,
    CReturn,
    CCall
}

impl Parser {
    // constructor
    pub fn new(mut input_stream: File) -> Parser {

        let mut reader= String::new();
        input_stream.read_to_string(&mut reader).unwrap();
        
        let parser = Parser {
            reader,
            line_tally: 0,
            // line_pos: 0,
            string_pos: 0
        };

        parser
    }

    fn peek_line(&self) -> String {
        let mut start = self.reader.chars().skip(self.string_pos);
        let mut string = String::new();
        loop {
            let c = start.next().unwrap();
            if c == '\n' || c=='\r' {
                break;
            }

            string.push(c);
        }

        string
    }

    fn consume_line(&mut self) -> String {
        if self.string_pos == 0 {
            return String::default();
        }

        let mut start = self.reader.chars().skip(self.string_pos).peekable();
        let mut string = String::new();
        loop {
            let c = start.next().unwrap();

            if c == '\r' && start.peek() == Some(&'\n') {
                self.string_pos+=2;
                break;
            }
            else if c == '\n' {
                self.string_pos+=1;
                break;
            } else {
                self.string_pos+=1;
                println!("{:?}", self.reader.chars().skip(self.string_pos).collect::<Vec<char>>());
                string.push(c);
            }
        }

        println!("string pos: {}\\{}", self.string_pos, self.reader.len());

        self.line_tally +=1;

        string
    }

    pub fn has_more_lines(&mut self) -> bool {
        println!("peek ={}", self.peek_line());
        self.peek_line().is_empty()
    } 

    // advance to next line. Must be called after construction
    pub fn advance(&mut self){
        println!("advacing");
        let consumed = self.consume_line();
        println!("consumed={}", consumed);
    }

    pub fn command_type(&self) -> CommandType {
        

        todo!("Command type");
    }

    pub fn arg1(&self) -> String {
        todo!();
    }

    pub fn arg2(&self) -> i16 {
        todo!()
    }

    pub fn line_number(&self) -> usize {
        self.line_tally
    }


}


fn ignore_whitespace(reader: &mut BufReader<File>) {
    let mut byte = [0u8];
    loop {

        reader.read_exact(&mut byte).unwrap();
        let c = byte[0] as char;
        if !c.is_whitespace() {
            continue;
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