pub struct CodeWriter {}
use super::parser::CommandType;

impl CodeWriter {
    pub fn new() -> CodeWriter {
        CodeWriter {}
    }

    pub fn write_arithmetic(&self, command: String) {
        todo!();
    }

    pub fn write_push_pop(&self, command: CommandType) {
        todo!();
    }

    pub fn close(self) {
        todo!()
    }
    
}