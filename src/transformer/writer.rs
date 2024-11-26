use super::simple_emitter::{SimpleEmitter, SContext};
use super::parser::{ArithmeticType, Segment};
use std::fs::File;
use std::marker::PhantomData;
use std::sync::Arc;

use super::parser::CommandDetails;
use super::emit::{EContext, EmitAsm};

pub struct CodeWriter<C, E>
    where C: EContext,
    E: EmitAsm<C>
{
    emit: E,
    first_run: bool,
    emit_init: bool,
    _phantom: PhantomData<C>
}

#[derive(Clone)]
pub struct WriterContext<C>
    where
        C: EContext,
{
    emitter_sate: C,
}

impl<C> Default for WriterContext<C>
    where
        C: EContext,
{
    fn default() -> Self {
        Self {
            emitter_sate: C::default(),
        }
    }
}
impl<C, E> CodeWriter<C, E>
    where C: EContext,
    E: EmitAsm<C>
{

    pub fn with_context(
        writer_context: WriterContext<C>,
        output_stream: Arc<File>,
        emit_init: bool,
    ) -> Self {
        let writer  = E::with_context(writer_context.emitter_sate, output_stream);

        CodeWriter {
            emit: writer,
            first_run: true,
            emit_init,
            _phantom: PhantomData::default()
        }
    }

    // constructor

    pub fn new(output_stream: Arc<File>, emit_init: bool) -> CodeWriter<C, E> {
        let writer = E::new(output_stream);

        CodeWriter {
            emit: writer,
            first_run: true,
            emit_init,
            _phantom: PhantomData::default()
        }
    }

    pub fn close(self) -> WriterContext<C> {
        WriterContext {
            emitter_sate: self.emit.close(),
        }
    }

    pub fn write_command(&mut self, command: &CommandDetails, source: &String) {
        if self.first_run {

            if self.emit_init {
                self.emit.emit_init();
            }

            self.emit.prelude();

            self.first_run = false;
        }

        // if self.first_run && self.emit_init {
        //     self.emit.emit_init();
        //     self.first_run = false;
        // }

        self.emit
            .comment(format_args!("//{}\n", source))
            .expect("Io error");

        match command {
            CommandDetails::Push(segment, arg1) => match segment {
                Segment::Constant => self.emit.push_const(*arg1),
                Segment::Local => self.emit.push_local_n(*arg1),
                Segment::Argument => self.emit.push_arg_n(*arg1),
                Segment::Temp => self.emit.push_temp_n(*arg1),
                Segment::Static => self.emit.push_static_n(*arg1),
                Segment::That => self.emit.push_that_n(*arg1),
                Segment::This => self.emit.push_this_n(*arg1),
                Segment::Pointer => self.emit.push_ptr_n(*arg1),
            },
            CommandDetails::Pop(segment, arg1) => match segment {
                Segment::Constant => unimplemented!("cannot pop a constant"),
                Segment::Local => self.emit.pop_local_n(*arg1),
                Segment::Argument => self.emit.pop_argument_n(*arg1),
                Segment::Temp => self.emit.pop_temp_n(*arg1),
                Segment::Static => self.emit.pop_static_n(*arg1),
                Segment::That => self.emit.pop_that_n(*arg1),
                Segment::This => self.emit.pop_this_n(*arg1),
                Segment::Pointer => self.emit.pop_ptr_n(*arg1),
            },
            CommandDetails::Arithmetic(ArithmeticType::Add) => self.emit.add(),
            CommandDetails::Arithmetic(ArithmeticType::Eq) => self.emit.eq(),
            CommandDetails::Arithmetic(ArithmeticType::Sub) => self.emit.sub(),
            CommandDetails::Arithmetic(ArithmeticType::Neg) => self.emit.neg(),
            CommandDetails::Arithmetic(ArithmeticType::Gt) => self.emit.gt(),
            CommandDetails::Arithmetic(ArithmeticType::Lt) => self.emit.lt(),
            CommandDetails::Arithmetic(ArithmeticType::And) => self.emit.and(),
            CommandDetails::Arithmetic(ArithmeticType::Or) => self.emit.or(),
            CommandDetails::Arithmetic(ArithmeticType::Not) => self.emit.not(),

            CommandDetails::Label(symbol) => self.emit.label(symbol),
            CommandDetails::Goto(symbol) => self.emit.goto(symbol.as_str()),
            CommandDetails::IfGoto(symbol) => self.emit.ifgoto(symbol.as_str()),
            CommandDetails::Function { n_vars, symbol } => {
                self.emit.function(*n_vars, symbol.as_str())
            }
            CommandDetails::Return => self.emit._return(),
            CommandDetails::Call { n_args, symbol } => self.emit.call(*n_args, symbol.as_str()),
        }
    }
}
