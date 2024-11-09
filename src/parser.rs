use std::{fs::File, io::{BufRead, BufReader, Lines, Read}, iter::Peekable};
use std::fmt::{Display, Formatter};

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

    pub fn peek_n(&self, len: usize) -> Option<&char> {
        self.characters.get(self.cursor+len)
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

    pub fn peek_rest(&self) -> String {
        let mut rest = String::new();
        for i in self.cursor.. self.characters.len() {
            rest.push(self.characters[i]);
        }

        return rest;
    }

}


pub struct Parser {
    scanner: Scanner
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Segment {
    Local,
    Constant,  // Aka const
    Argument,
    Temp,
    Static,
    That,
    This,
    Pointer
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArithmeticType {
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not
}



#[derive(Debug)]
pub enum  CommandDetails {
    Arithmetic(ArithmeticType),
    Push(Segment, i16),
    Pop(Segment, i16),
    Label,
    Goto,
    If,
    Function,
    Return,
    Call
}




impl Parser {
    // constructor
    pub fn new(mut input_stream: File) -> Parser {

        let mut reader= String::new();
        input_stream.read_to_string(&mut reader).unwrap();
        
        let parser = Parser {
            scanner: Scanner::new(reader.chars().collect())
        };

        parser
    }

    fn peek_line(&self) -> String {
        let mut str = String::new();
        for i in 0.. {
            let c = self.scanner.peek_n(i);
            match c {
                Some(v) => {
                    if v == &'\n' || v == &'\r' {
                        break;
                    } else {
                        str.push(*v);
                    }
                }
                None => {break;}
            }
        }

        return str;
    }

    fn consume_line(&mut self) -> String {
        let line = self.peek_line();
        for i in 0.. {
            let c = self.scanner.pop(); 
            match c {
                Some('\n') => {
                    break;
                }
                _ => {}
            }
        }

        line
    }

    fn consume_non_whitespace(&mut self) {
        while let Some(v) = self.scanner.peek() {
            if v.is_whitespace() {
               break; 
            }
            self.scanner.pop();
        }
    }

    fn consume_whitespace(&mut self) {
        loop {
            match self.scanner.peek() {
                Some(v) => {
                    if !v.is_whitespace() {
                        break;
                    } else {
                        let _ =  self.scanner.pop();
                    }
                }
                None => {break;} 
            }
        }
    }

    fn parse_integer(&mut self) -> i16 {
        self.consume_whitespace();
        let rest = self.peek_line();
        
        let mut s= String::new();
        for c in rest.chars() {
            if c.is_whitespace() {
                break;
            } else {
                s.push(c);
            }
        }

        // println!("s='{}'", s);
        let v = str::parse(s.as_str())
            .unwrap();
        
        v
    }

    fn parse_segment(&mut self, source_line: &str) -> Segment {
        self.consume_non_whitespace();
        self.consume_whitespace();
        let rest = self.peek_line();
        let segment;
        if rest.starts_with("constant") {
            segment = Segment::Constant;
        } else if rest.starts_with("local") {
            segment = Segment::Local;
        } else if rest.starts_with("argument") {
            segment = Segment::Argument;
        } else if rest.starts_with("this") {
            segment = Segment::This;
        } else if rest.starts_with("that") {
            segment = Segment::That;
        } else if rest.starts_with("temp") {
            segment = Segment::Temp;
        } else {
            println!("rest='{}'. Line='{}'", rest,source_line);
            unimplemented!();
        }

        self.consume_non_whitespace();

        return segment;
    }

    // reuturns none if end of parsing
    pub fn next_command(&mut self) -> Option<(CommandDetails, String)> {
        self.consume_line();

        self.consume_whitespace();

        let rest = self.peek_line();
        if rest.len() == 0 {
            return None;
        }
        
        if rest.starts_with("pop") {
            let segment = self.parse_segment(&rest);
            let value = self.parse_integer();
            return Some((CommandDetails::Pop(segment, value), rest.clone()));
        } else if rest.starts_with("push") {
            let segment = self.parse_segment(&rest);
            let value = self.parse_integer();
            return Some((CommandDetails::Push(segment, value), rest.clone()))
        } else if rest.starts_with("//") {
            return self.next_command();
        } else if rest.starts_with("add") {
            return Some((CommandDetails::Arithmetic(ArithmeticType::Add), rest.clone()));
        } else if rest.starts_with("sub") {
            return Some((CommandDetails::Arithmetic(ArithmeticType::Sub), rest.clone()));
        } else if rest.starts_with("eq") {
            return Some((CommandDetails::Arithmetic(ArithmeticType::Eq), rest.clone()));
        } else if rest.starts_with("lt") {
            return Some((CommandDetails::Arithmetic(ArithmeticType::Lt), rest.clone()));
        } else if rest.starts_with("gt") {
            return Some((CommandDetails::Arithmetic(ArithmeticType::Gt), rest.clone()));
        } else if rest.starts_with("neg") {
            return Some((CommandDetails::Arithmetic(ArithmeticType::Neg), rest.clone()));
        } else if rest.starts_with("and") {
            return Some((CommandDetails::Arithmetic(ArithmeticType::And), rest.clone()));
        } else if rest.starts_with("or") {
            return Some((CommandDetails::Arithmetic(ArithmeticType::Or), rest.clone()));
        } else if rest.starts_with("not") {
            return Some((CommandDetails::Arithmetic(ArithmeticType::Not), rest.clone()));
        } else {
            println!("rest='{}'", rest);
            unimplemented!();
        }

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