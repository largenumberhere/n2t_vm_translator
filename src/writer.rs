use crate::emit_asm::Emitter;



pub struct CodeWriter<W: std::io::Write> {
    emit: Emitter<W>,
    first_run: bool,
}
use std::io::{BufWriter, Write};
use indoc::indoc;
use crate::parser::{ArithmeticType, Segment};

use super::parser::CommandDetails;

impl<W: std::io::Write> CodeWriter<W> {
    // constructor
    pub fn new(output_stream: W) -> CodeWriter<W> {
        let writer = Emitter::new(output_stream);
        
        CodeWriter {
            emit: writer,
            first_run: true,
        }
    }
    pub fn write_command(&mut self, command: &CommandDetails, source: &String) {
        if self.first_run {
            self.emit.emit_init();
            self.first_run = false;
        }

        self.emit.comment(format_args!("//{}\n", source))
            .expect("Io error");

        match command {
            CommandDetails::Push(segment, arg1) => {
                match segment {
                    Segment::Constant =>    self.emit.push_const(*arg1),
                    Segment::Local =>       self.emit.push_local_n(*arg1),
                    Segment::Argument =>    self.emit.push_arg_n(*arg1),
                    Segment::Temp =>        self.emit.push_temp_n(*arg1),
                    Segment::Static =>      self.emit.push_static_n(*arg1),
                    Segment::That =>        self.emit.push_that_n(*arg1),
                    Segment::This =>        self.emit.push_this_n(*arg1),
                    Segment::Pointer =>     self.emit.push_ptr_n(*arg1),
                }
            }
            CommandDetails::Pop(segment, arg1) => {
                match segment {
                    Segment::Constant =>    unimplemented!("cannot pop a constant"),
                    Segment::Local =>       self.emit.pop_local_n(*arg1),
                    Segment::Argument =>    self.emit.pop_argument_n(*arg1),
                    Segment::Temp =>        self.emit.pop_temp_n(*arg1),
                    Segment::Static =>      self.emit.pop_static_n(*arg1),
                    Segment::That =>        self.emit.pop_that_n(*arg1),
                    Segment::This =>        self.emit.pop_this_n(*arg1),
                    Segment::Pointer =>     self.emit.pop_ptr_n(*arg1)
                }
            }
            CommandDetails::Arithmetic(ArithmeticType::Add) =>  self.emit.add(),
            CommandDetails::Arithmetic(ArithmeticType::Eq) =>   self.emit.eq(),
            CommandDetails::Arithmetic(ArithmeticType::Sub) =>  self.emit.sub(),
            CommandDetails::Arithmetic(ArithmeticType::Neg) =>  self.emit.neg(),
            CommandDetails::Arithmetic(ArithmeticType::Gt) =>   self.emit.gt(),
            CommandDetails::Arithmetic(ArithmeticType::Lt) =>   self.emit.lt(),
            CommandDetails::Arithmetic(ArithmeticType::And) =>  self.emit.and(),
            CommandDetails::Arithmetic(ArithmeticType::Or) =>   self.emit.or(),
            CommandDetails::Arithmetic(ArithmeticType::Not) =>  self.emit.not(),

            CommandDetails::Label(symbol) => self.emit.label(symbol.as_str()),
            CommandDetails::Goto(symbol) => self.emit.goto(symbol.as_str()),
            CommandDetails::IfGoto(symbol) => self.emit.ifgoto(symbol.as_str()),
            CommandDetails::Function =>     todo!("function"),
            CommandDetails::Return =>       todo!("return"),
            CommandDetails::Call =>         todo!("call"),

        }
        
    }

}