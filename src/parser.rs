use std::{fs::File, io::{BufReader, Read}};
use std::fmt::Display;
use crate::transformer::TransformError;
use crate::transformer::TransformResult;

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

    pub fn line(&self) -> usize {
        // this should only be used under error conditions so it doesn't need to be fast
        let mut i = 0;
        for c in self.characters.iter() {
            if *c == '\n' {
                i+=1;
            }
        }

        return i;

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
    Label(String),
    IfGoto(String),
    Function {n_vars: i16, symbol: String},
    Return,
    Call{n_args: i16, symbol: String},
    Goto(String)
}




impl Parser {
    // constructor
    pub fn new<R: Read>(mut input_stream: R) -> Parser {

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
                None => {break;}
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
        } else if rest.starts_with("static") {
            segment = Segment::Static;
        } else if rest.starts_with("pointer") {
            segment = Segment::Pointer;
        } else {
            println!("rest='{}'. Line='{}'", rest,source_line);
            unimplemented!();
        }

        self.consume_non_whitespace();

        return segment;
    }

    fn parse_label_symbol(&mut self) -> String {
        self.consume_non_whitespace();
        self.consume_whitespace();
        let mut string = String::new();
        loop {
            if let Some(v) = self.scanner.peek() {
                // check for a non-alphanumeric character
                if ! (v.is_alphabetic() || *v=='.') {
                    break;
                }
                string.push(*self.scanner.pop().unwrap());
            } else {break;}
        }

        self.consume_non_whitespace();

        string
    }

    // reuturns none if end of parsing
    pub fn next_command(&mut self) -> Option<TransformResult<(CommandDetails, String)>> {
        self.consume_line();
        self.consume_whitespace();

        let rest = self.peek_line();
        if rest.len() == 0 {
            return None;
        }
        
        if rest.starts_with("pop") {
            let segment = self.parse_segment(&rest);
            let value = self.parse_integer();
            return Some(Ok((CommandDetails::Pop(segment, value), rest.clone())));
        } else if rest.starts_with("push") {
            let segment = self.parse_segment(&rest);
            let value = self.parse_integer();
            return Some(Ok((CommandDetails::Push(segment, value), rest.clone())))
        } else if rest.starts_with("//") {
            return self.next_command();
        } else if rest.starts_with("add") {
            return Some(Ok((CommandDetails::Arithmetic(ArithmeticType::Add), rest.clone())));
        } else if rest.starts_with("sub") {
            return Some(Ok((CommandDetails::Arithmetic(ArithmeticType::Sub), rest.clone())));
        } else if rest.starts_with("eq") {
            return Some(Ok((CommandDetails::Arithmetic(ArithmeticType::Eq), rest.clone())));
        } else if rest.starts_with("lt") {
            return Some(Ok((CommandDetails::Arithmetic(ArithmeticType::Lt), rest.clone())));
        } else if rest.starts_with("gt") {
            return Some(Ok((CommandDetails::Arithmetic(ArithmeticType::Gt), rest.clone())));
        } else if rest.starts_with("neg") {
            return Some(Ok((CommandDetails::Arithmetic(ArithmeticType::Neg), rest.clone())));
        } else if rest.starts_with("and") {
            return Some(Ok((CommandDetails::Arithmetic(ArithmeticType::And), rest.clone())));
        } else if rest.starts_with("or") {
            return Some(Ok((CommandDetails::Arithmetic(ArithmeticType::Or), rest.clone())));
        } else if rest.starts_with("not") {
            return Some(Ok((CommandDetails::Arithmetic(ArithmeticType::Not), rest.clone())));
        } else if rest.starts_with("label") {
            let symbol = self.parse_label_symbol();
            return Some(Ok((CommandDetails::Label(symbol), rest.clone())));
        } else if rest.starts_with("if-goto") {
            let symbol = self.parse_label_symbol();
            return Some(Ok((CommandDetails::IfGoto(symbol), rest.clone())));
        } else if rest.starts_with("goto") {
            let symbol = self.parse_label_symbol();
            return Some(Ok(((CommandDetails::Goto(symbol), rest.clone()))));
        } else if rest.starts_with("function") {
            let symbol = self.parse_label_symbol();
            let n_vars = self.parse_integer();
            return Some(Ok((CommandDetails::Function { symbol, n_vars }, rest.clone())));
        } else if rest.starts_with("call") {
            let symbol = self.parse_label_symbol();
            let n_args = self.parse_integer();
            return Some(Ok((CommandDetails::Call { symbol, n_args }, rest.clone())));
        } else if rest.starts_with("return") {
            return Some(Ok((CommandDetails::Return, rest.clone())));
        } else {
            let err = format!("Unimplemented command.'{}'", rest);
            return Some(Err(TransformError::SyntaxError(err, self.scanner.line())));
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