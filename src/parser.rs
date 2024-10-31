pub struct Parser {}


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
    pub fn new() -> Parser {
        Parser {}
    }

    pub fn has_more_lines(&self) -> bool {
        todo!()
    } 

    // advance to next line. Must be called after construction
    pub fn advance(&self){
        todo!()
    }

    pub fn command_type(&self) -> CommandType {
        todo!();
    }

    pub fn arg1(&self) -> String {
        todo!();
    }

    pub fn arg2(&self) -> i16 {
        todo!()
    }

}