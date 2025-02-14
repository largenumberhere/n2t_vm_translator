//! A hack assembly emitter that prioritizes ease of implementation

use std::fs::File;
use std::io::BufWriter;

use std::fmt::{Arguments, Write as FmtWrite};
use std::io::Write as IoWrite;
use std::sync::Arc;
use crate::transformer::Segment;
use hack_macro::{emit_fmt_hack, emit_hack, fmt_hack, hack_str};
use crate::transformer::emit::{EContext, EmitAsm};

struct SymbolGenerator {
    next_id: usize,
}

impl SymbolGenerator {
    fn new() -> SymbolGenerator {
        SymbolGenerator { next_id: 0 }
    }

    fn next_commented(&mut self, label_start: &str) -> String {
        assert!(
            label_start.split_whitespace().skip(1).next().is_none(),
            "no whitespace allowed in labels"
        );
        let mut out = String::new();
        out.write_fmt(format_args!("_{}_L{}", label_start, self.next_id))
            .unwrap();

        self.next_id += 1;

        return out;
    }
}

#[derive(Clone)]
struct FuncEmitter {
    returns: usize,
    calls: usize,
}

impl FuncEmitter {
    fn new() -> FuncEmitter {
        FuncEmitter { returns: 0, calls: 0 }
    }

    // create a new unique id for a call label
    fn call(&mut self) -> usize {
        let ret = self.calls;
        self.calls+=1;

        return ret;
    }

    fn ret(&mut self) -> usize {
        let ret = self.returns;
        self.returns+=1;

        return ret;
    }
}

pub struct SimpleEmitter {
    writer: BufWriter<Arc<File>>,
    symbol_generator: SymbolGenerator,
    emitted_instructions_count: usize,
    func_emitter: FuncEmitter
}


impl EmitAsm<SContext> for SimpleEmitter {
    fn with_context(context: SContext, stream: Arc<File>) -> Self {
        Self::with_context(context, stream)
    }

    fn new(stream: Arc<File>) -> Self {
        Self::new(stream)
    }

    fn close(self) -> SContext {
        self.close()
    }

    fn emit_init(&mut self) {
        self.emit_init();
    }

    fn prelude(&mut self) {}

    fn comment(&mut self, args: Arguments) -> std::io::Result<()> {
        self.comment(args)
    }

    fn push_const(&mut self, val: i16) {
        self.push_const(val)
    }

    fn push_local_n(&mut self, offset: i16) {
        self.push_local_n(offset)
    }

    fn push_arg_n(&mut self, offset: i16) {
        self.push_arg_n(offset)
    }

    fn push_temp_n(&mut self, offset: i16) {
        self.push_temp_n(offset)
    }

    fn push_static_n(&mut self, offset: i16) {
        self.push_static_n(offset)
    }

    fn push_ptr_n(&mut self, offset: i16) {
        self.push_ptr_n(offset)
    }

    fn pop_local_n(&mut self, offset: i16) {
        self.pop_local_n(offset)
    }

    fn pop_argument_n(&mut self, offset: i16) {
        self.pop_argument_n(offset)
    }

    fn pop_temp_n(&mut self, offset: i16) {
        self.pop_temp_n(offset)
    }

    fn pop_static_n(&mut self, offset: i16) {
        self.pop_static_n(offset)
    }

    fn pop_that_n(&mut self, offset: i16) {
        self.pop_that_n(offset)
    }

    fn pop_this_n(&mut self, offset: i16) {
        self.pop_this_n(offset)
    }

    fn call(&mut self, n_args: i16, symbol: &str) {
        self.call(n_args, symbol, false)
    }

    fn _return(&mut self) {
        self._return()
    }

    fn function(&mut self, n_vars: i16, symbol: &str) {
        self.function(n_vars, symbol)
    }

    fn goto(&mut self, symbol: &str) {
        self.goto(symbol)
    }

    fn or(&mut self) {
        self.or()
    }

    fn lt(&mut self) {
        self.lt()
    }

    fn gt(&mut self) {
        self.gt()
    }

    fn sub(&mut self) {
        self.sub()
    }

    fn neg(&mut self) {
        self.neg()
    }

    fn push_this_n(&mut self, offset: i16) {
        self.push_this_n(offset)
    }

    fn push_that_n(&mut self, offset: i16) {
        self.push_that_n(offset)
    }

    fn pop_ptr_n(&mut self, offset: i16) {
        self.pop_ptr_n(offset)
    }

    fn add(&mut self) {
        self.add()
    }

    fn eq(&mut self) {
        self.eq()
    }

    fn and(&mut self) {
        self.and()
    }

    fn label(&mut self, symbol: &str) {
        self.label(symbol)
    }

    fn ifgoto(&mut self, symbol: &str) {
        self.ifgoto(symbol)
    }

    fn not(&mut self) {
        self.not()
    }
}

#[derive(Clone)]
pub struct SContext {
    emitted_instructions_count: usize,
    func_emitter: FuncEmitter
}

impl Default for SContext {
    fn default() -> Self {
        Self {
            emitted_instructions_count: 0,
            func_emitter: FuncEmitter::new()
        }
    }
}

impl EContext for SContext {}

const LOGIC_TRUE: i16 = -1;
const LOGIC_FALSE: i16 = 0;

impl SimpleEmitter {
    pub fn close(self) -> SContext {
        return SContext {
            emitted_instructions_count: self.emitted_instructions_count,
            func_emitter: self.func_emitter,
        };
    }

    pub fn new(stream: Arc<File>) -> SimpleEmitter {
        SimpleEmitter {
            writer: BufWriter::new(stream),
            symbol_generator: SymbolGenerator::new(),
            emitted_instructions_count: 0,
            func_emitter: FuncEmitter::new()
        }
    }

    pub fn with_context(emitter_context: SContext, stream: Arc<File>) -> Self {
        SimpleEmitter {
            writer: BufWriter::new(stream),
            symbol_generator: SymbolGenerator::new(),
            emitted_instructions_count: emitter_context.emitted_instructions_count,
            func_emitter: FuncEmitter::new()
        }
    }

    pub fn comment(&mut self, args: std::fmt::Arguments) -> std::io::Result<()> {
        self.write_fmt(args)
    }
    fn write_fmt(&mut self, args: std::fmt::Arguments) -> std::io::Result<()> {
        self.writer.write_fmt(args)
    }

    pub fn emit_init(&mut self) {
        emit_hack! {r"
            @256
            D=A
            @SP
            M=D         // initialise stack pointer

            @1
            D=-A
            @LCL
            M=D

            @2
            D=-A
            @ARG
            M=D

            @3
            D=-A
            @THIS
            M=D

            @4
            D=-A
            @THAT
            M=D        // initialize segment pointers to a known value
        "};

        self.call(0,"Sys.init", true);
        self.emitln("");

    }

    fn emitln(&mut self, str: &str) {
        let lines = str.split('\n');
        for line in lines {
            let line = line.trim();

            // if line is instruction, display the instruction number for debugging convenience
            if !line.starts_with("//") && !line.starts_with("(") && !line.is_empty() {
                let no = self.emitted_instructions_count;
                self.write_fmt(format_args!("{:90}//{:3}\n", line, no))
                    .unwrap();
                self.emitted_instructions_count += 1;
            } else {
                self.write_fmt(format_args!("{:90}\n", line)).unwrap();
            }
        }
    }

    fn emit_label_start(&mut self, symbol: &str) {
        emit_fmt_hack!("({})", symbol);
    }


    // puts the last item into register A
    // clobbers A
    // tested
    fn stack_to_a(&mut self) {
        emit_hack! {r"
            @SP
            M=M-1       // Decrement stack pointer
            A=M         // A = Stack pointer
            A=M         // A = old top of stack
        "};
    }

    // tested
    // clobbers, A, D
    fn stack_to_d(&mut self) {
        emit_hack! {r"
            @SP
            M=M-1       // Decrement stack pointer
            A=M         // A = Stack pointer
            D=M         // D = old top of stack
        "};
    }

    // push the item in register A onto the stack
    fn a_to_stack(&mut self) {
        emit_hack! {r"
            D=A
            @SP
            M=M+1       // increase stack pointer
            A=M-1       // get top of stack
            M=D         // top of stack = D
        "};
    }

    fn d_to_stack(&mut self) {
        emit_hack! {r"
            @SP
            M=M+1       // increase stack pointer
            A=M-1       // get top of stack
            M=D         // top of stack = D
        "};
    }

    fn emit_push_local_a(&mut self) {
        emit_hack! {r"
            // emit_push_a
            D=A
            @LCL
            A=M     // A=*LCL
            M=D     // **LCL = D
            @SP
            M=M+1   //*LCL = *LCL+1
        "};
    }

    fn assign_a(&mut self, value: i16) {
        emit_fmt_hack!(r"
            @{0} // A = {0}
        ", value);
    }

    fn assign_a_str(&mut self, value: &str) {
        emit_fmt_hack!(r"
            @{0}    // A = {0}
        ", value);
    }

    // tested
    pub fn push_const(&mut self, val: i16) {
        self.const_to_stack(val);
    }

    pub fn add(&mut self) {
        self.stack_to_temp(TempRegister::T0);
        self.stack_to_temp(TempRegister::T1);
        self.add_temp(TempRegister::T0, TempRegister::T0, TempRegister::T1);
        self.temp_to_stack(TempRegister::T0);

    }


    pub fn eq(&mut self) {
        self.stack_to_temp(TempRegister::T0);
        self.stack_to_temp(TempRegister::T1);
        self.sub_temp(TempRegister::T0, TempRegister::T0, TempRegister::T1);
        self.not_zero_tmp(TempRegister::T0);
        self.temp_to_stack(TempRegister::T0);

    }

    pub fn sub(&mut self) {
        self.stack_to_temp(TempRegister::T0);
        self.stack_to_temp(TempRegister::T1);
        self.sub_temp(TempRegister::T0, TempRegister::T1, TempRegister::T0);
        self.temp_to_stack(TempRegister::T0);

        self.emitln("");
    }

    // tested!
    pub fn lt(&mut self) {
        let is_lt = self.symbol_generator.next_commented("is_lt");
        let is_not_lt = self.symbol_generator.next_commented("is_not_lt");
        let end = self.symbol_generator.next_commented("end");

        self.stack_to_d();
        emit_fmt_hack!(r"
            // D = value from stack
            @SP
            A=M-1   // address of top item in stack
            A=M     // value from top of stack
            D=A-D   // D = stack[0] - stack[1]
            @{is_lt}
            D;JLT   // if true, goto is_lt, else goto is_not_lt
            ({is_not_lt})
                D={LOGIC_FALSE}
            @{end}
            0;JMP
            ({is_lt})
                D={LOGIC_TRUE}
            ({end})
            @SP
            A=M-1   // address of top item in stack
            M=D      // write value to stack
        ");

        self.emitln("");
    }

    // tested
    pub fn gt(&mut self) {
        let is_gt = self.symbol_generator.next_commented("is_gt");
        let is_not_gt = self.symbol_generator.next_commented("is_not_gt");
        let end = self.symbol_generator.next_commented("end");
        self.stack_to_d();
        emit_fmt_hack!(r"
            // D = item from stack
            @SP
            A=M-1   // address of top item in stack
            A=M     // D = 2nd item from stack
            D=D-A   // D = stack[0] - stack[1]
            @{is_gt}
            D;JLT   // if true, goto is_gt, else goto is_not_gt
            ({is_not_gt})
                D={LOGIC_FALSE}
                @{end}
                0;JMP
            ({is_gt})
                D={LOGIC_TRUE}
            ({end})
            @SP
            A=M-1   // address of top item in stack
            M=D     // write result to top of stack
        ");
        self.emitln("");
    }

    // tested
    pub fn neg(&mut self) {
        emit_hack! {r"
            @SP
            A=M-1           // A = pointer to last item on stack
            D=M             // D = last item on stack
            D=-D            // negate value
            M=D             // write result on stack
        "};
        self.emitln("");
    }

    // tested
    pub fn or(&mut self) {
        // D = pop1
        self.stack_to_d();
        emit_hack! {r"
            @SP
            A=M-1       // pointer to last item on stack
            M=M|D       // write result to stack
        "};
        self.emitln("");
    }

    // tested
    pub fn not(&mut self) {
        emit_hack! {r"
            @SP
            A=M-1       // A = pointer to last item on stack
            D=M         // D = value from stack

            D=!D        // calculate
            M=D         // write result to stack
        "};
        self.emitln("");
    }

    pub fn and(&mut self) {
        // D = first item
        self.stack_to_d();
        emit_hack! {r"
            @SP
            A=M-1       // A = 2nd item from stack
            M=M&D       // write result to stack, overwriting 2nd item
        "};
        self.emitln("");
    }

    fn segment_symbol_str(&self, segment: Segment, _offset: i16) -> &str {
        return match segment {
            Segment::Local => "LCL",
            Segment::Constant => {
                unreachable!("Constant is not a real segment")
            }
            Segment::Argument => "ARG",
            Segment::Temp => "TMP",
            Segment::Static => "STATIC",
            Segment::That => "THAT",
            Segment::This => "THIS",
            Segment::Pointer => "THIS",
        };
    }

    // move the value at offset n from the segment onto the stack
    fn pop_non_stack_segment(&mut self, segment: Segment, offset: i16) {
        // let not_temp_segment;
        let segment_symbol = self.segment_symbol_str(segment, offset);

        // D = address of segment start
        match segment {
            Segment::Pointer => {
                emit_fmt_hack!{"@{}", segment_symbol};
                emit_hack! {r"
                    D=A         // D = segment start
                "};
            }
            Segment::Temp => {
                emit_hack! {r"
                    @5
                    D=A
                "};
            }
            _ => {
                emit_fmt_hack!{"@{}", segment_symbol};
                emit_hack! {r"
                    D=M         // D = segment start
                "};
            }
        }

        // A = segment offset
        self.assign_a(offset);
        emit_hack! {r"
            D=D+A      // D = pointer to destination in segment
            @SP
            A=M         // A = stack pointer
            M=D         // write segment destination to stack
            @SP
            M=M+1       // increase stack pointer
        "};

        emit_hack! {r"
            @SP
            M=M-1       // decrease stack pointer
            M=M-1       // decrease stack pointer
            A=M
            D=M         // read value from stack
            @SP
            M=M+1       // Increase stack pointer
            A=M         // A = stack pointer
            A=M         // A = segment destination
            M=D         // write to the segment destination
            @SP
            M=M-1       // Decrease stack pointer
        "};

        self.emitln("");
    }

    // move the value from the stack to the segment at offset n
    fn push_non_stack_segment(&mut self, segment: Segment, offset: i16) {
        let segment_symbol = self.segment_symbol_str(segment, offset);

        // D = address of segment start
        match segment {
            Segment::Pointer => {
                emit_fmt_hack!{"@{}", segment_symbol};
                emit_hack! {r"
                    D=A         // D = segment start
                "};
            }
            Segment::Temp => {
                emit_hack! {r"
                    @5
                    D=A
                "};
            }
            _ => {
                emit_fmt_hack!{"@{}", segment_symbol};
                emit_hack! {r"
                    D=M         // D = segment start
                "};
            }
        }

        // A = segment offset
        self.assign_a(offset);
        emit_hack! {r"
            A=A+D   // A = pointer to read from
            D=M     // D = value in segment
            @SP
            M=M+1   // increase stack pointer
            A=M-1   // A = value at top of stack
            M=D     // write value to stack
        "};
        self.emitln("");
    }

    // take the argument at offset n and place it on the stack
    pub fn pop_argument_n(&mut self, n: i16) {
        self.pop_non_stack_segment(Segment::Argument, n);
    }

    pub fn push_local_n(&mut self, n: i16) {
        self.push_non_stack_segment(Segment::Local, n);
    }

    // take a value from the that segment at offset n and place it on the stack
    pub fn pop_that_n(&mut self, n: i16) {
        self.pop_non_stack_segment(Segment::That, n);
    }

    pub fn pop_temp_n(&mut self, n: i16) {
        self.pop_non_stack_segment(Segment::Temp, n);
    }

    pub fn pop_this_n(&mut self, n: i16) {
        self.pop_non_stack_segment(Segment::This, n);
    }

    // take value off the stack and write it to the 'that' segment at offset n
    pub fn push_that_n(&mut self, n: i16) {
        self.push_non_stack_segment(Segment::That, n);
    }

    pub fn push_arg_n(&mut self, n: i16) {
        self.push_non_stack_segment(Segment::Argument, n);
    }

    pub fn push_temp_n(&mut self, n: i16) {
        self.push_non_stack_segment(Segment::Temp, n);
    }

    pub fn push_static_n(&mut self, n: i16) {
        self.push_non_stack_segment(Segment::Static, n);
    }

    pub fn push_this_n(&mut self, n: i16) {
        self.push_non_stack_segment(Segment::This, n);
    }

    pub fn push_ptr_n(&mut self, n: i16) {
        self.push_non_stack_segment(Segment::Pointer, n);
    }

    pub fn pop_static_n(&mut self, n: i16) {
        self.pop_non_stack_segment(Segment::Static, n);
    }

    pub fn pop_local_n(&mut self, n: i16) {
        self.pop_non_stack_segment(Segment::Local, n);
    }

    pub fn pop_ptr_n(&mut self, n: i16) {
        self.pop_non_stack_segment(Segment::Pointer, n);
    }

    const USER_LABEL_PREFIX: &'static str = "user_";
    // declare the start of a label. mangles the label
    pub fn label(&mut self, symbol: &str) {
        emit_fmt_hack!{r"({}{})", Self::USER_LABEL_PREFIX, symbol};
    }

    // jump to the symbol if stack top > 0
    pub fn ifgoto(&mut self, symbol: &str) {
        self.stack_to_d();
        emit_fmt_hack! {r"@{}{}", Self::USER_LABEL_PREFIX, symbol};
        emit_hack! {r"
            D;JNE
        "};
        self.emitln("");
    }

    pub fn goto(&mut self, symbol: &str) {
        emit_fmt_hack! {r"@{}{}", Self::USER_LABEL_PREFIX, symbol};
        emit_hack! {r"
            0;JMP
        "};
        self.emitln("");
    }



    // a function declaration
    pub fn function(&mut self, n_vars: i16, symbol: &str) {
        // inject label. Todo: make it comply with mangling rules
        self.emit_label_start(symbol);

        // create room for locals on stack and zero them
        for _ in 0..n_vars {
            emit_fmt_hack!(r"
                @SP
                A=M     // get address to top of stack
                M=0     // zero stack
                @SP
                M=M+1   // increase stack pointer
            ");
        }
    }

    // passes SimpleFunction test
    pub fn _return(&mut self) {
        /*
            - return the value
            - recover callee's segment registers
            - return
            - drop the stack, including all except first arg


            endframe = RAM[13]
            returnAddress = RAM[14]
            temp = RAM[15]


        */

        let endframe_ptr = 13;
        let return_address_ptr = 14;
        let callee_arg_ptr = 15;

        // *endframe_ptr = *LCL
        emit_fmt_hack!(r"
            @LCL
            D=M
            @{endframe_ptr}
            M=D
        ");

        // *return_address_ptr = * ((*endframe_ptr) + 5)
        emit_fmt_hack!(r"
            @{endframe_ptr}
            D=M // D = address of end of frame
            @5
            A=D-A   // D = address of address of return address
            D=M     // D = return address
            @{return_address_ptr}
            M=D
        ");

        // *callee_arg_ptr = *ARG
        emit_fmt_hack!(r"
            @ARG
            D=M
            @{callee_arg_ptr}
            M=D
        ");

        // **ARG = pop()
        self.stack_to_d();
        emit_fmt_hack!(r"
            @ARG
            A=M
            M=D
        ");

        // SP = *ARG + 1
        emit_fmt_hack!(r"
            @{callee_arg_ptr}
            D=M // D = *ARG
            D=D+1   // D = *ARG + 1
            @SP
            M=D     // *SP = (*ARG +1)
        ");

        // set caller's THAT to (endframe-1)
        emit_fmt_hack!(r"
            @{endframe_ptr}
            A=M
            A=A-1
            D=M
            @THAT
            M=D
        ");


        // set caller's THIS to (endframe -2)
        emit_fmt_hack!(r"
            @{endframe_ptr}
            A=M
            A=A-1
            A=A-1
            D=M
            @THIS
            M=D
        ");


        // set caller's ARG to (endframe -3)
        emit_fmt_hack!(r"
            @{endframe_ptr}
            A=M
            A=A-1
            A=A-1
            A=A-1
            D=M
            @ARG
            M=D
        ");

        // set caller's LCL to (endframe -4)
        emit_fmt_hack!(r"
            @{endframe_ptr}
            A=M
            A=A-1
            A=A-1
            A=A-1
            A=A-1
            D=M
            @LCL
            M=D
        ");

        // return
        emit_fmt_hack!(r"
            @{return_address_ptr}
            A=M
            0;JMP
        ");
    }


    fn segment_pointer_to_stack(&mut self, segment: Segment) {
        // let not_temp_segment;
        let segment_symbol = self.segment_symbol_str(segment, -1);

        // D = address of segment start
        match segment {
            Segment::Pointer => {
                emit_fmt_hack!("@{}", segment_symbol);
                emit_hack! {r"
                    D=A         // D = segment start
                "};
            }
            Segment::Temp => {panic!("temp has no segment pointer"); }
            _ => {
                emit_fmt_hack!("@{}", segment_symbol);
                emit_hack! {r"
                    D=M         // D = segment start
                "};
            }
        }

        // A = segment offset
        emit_hack! {r"
            @SP
            A=M         // A = stack pointer
            M=D         // write segment pointer to stack
            @SP
            M=M+1       // increase stack pointer
        "};
    }

    // clobbers A
    fn sp_at_offset(&mut self, mut offset: i16) {
        if offset > 0 {
            panic!("negative offset expected");
        }

        emit_fmt_hack!(r"
            @SP
            A=M
        ");

        // println!("offset {}", offset);
        while offset < 0 {
            // println!("A=A-1");
            emit_hack!("A=A-1");
            offset +=1;
        }

    }

    pub fn call(&mut self, mut n_args: i16, callee_symbol: &str, first_call: bool) {
        assert!(n_args >= 0);

        let ret_label = fmt_hack!("{}$ret.{}", callee_symbol, self.func_emitter.call());

        // save stackframe
            // make room for values
            /*
                | Offset |  Usage                   |
                |--------|--------------------------|
                | *SP -6 |  caller's last argument (if any)  |
                | *SP -5 |  return address          |
                | *SP -4 |  caller's LCL            |
                | *SP -3 |  caller's ARG            |
                | *SP -2 |  caller's THIS           |
                | *SP -1 |  caller's THAT           |
                | *SP -0 |  end of stack            |

                - save caller's segment pointers
                - insert return label
                - update ARG, LCL, and SP
                - jump to function prologue
            */

            // Increase stack pointer by 5 to reserve room for stackframe
            emit_fmt_hack!(r"
                @5
                D=A
                @SP
                M=M+D
            ");

            // offset in stack to write the values
            let caller_return_address = -5;
            let caller_lcl_offset = -4;
            let caller_arg_offset = -3;
            let caller_this_offset = -2;
            let caller_that_offset = -1;


            // write return address to stack
            emit_fmt_hack!(r"
                @{ret_label}
                D=A
            ");
            self.sp_at_offset(caller_return_address);
            emit_fmt_hack!(r"
                M=D
            ");

            // write LCL to stack
            emit_fmt_hack!(r"
                @LCL
                D=M
            ");
            self.sp_at_offset(caller_lcl_offset);
            emit_fmt_hack!(r"
                M=D
            ");

            // write ARG to stack
            emit_fmt_hack!(r"
                @ARG
                D=M
            ");
            self.sp_at_offset(caller_arg_offset);
            emit_fmt_hack!(r"
                M=D
            ");

            // write THIS to stack
            emit_fmt_hack!(r"
                @THIS
                D=M
            ");
            self.sp_at_offset(caller_this_offset);
            emit_fmt_hack!(r"
                M=D
            ");

            // write THAT to stack
            emit_fmt_hack!(r"
                @THAT
                D=M
            ");
            self.sp_at_offset(caller_that_offset);
            emit_fmt_hack!(r"
                M=D
            ");

            // set LCL to same as SP
            emit_fmt_hack!(r"
                @SP
                D=M
                @LCL
                M=D
            ");


        // update segment registers
            // set arg pointer to first argument
            // it must point to the first argument, or caller_return_address if none
            self.sp_at_offset(caller_return_address);
            if (n_args > 0) {
                for i in 0..n_args {
                    emit_fmt_hack!(r"
                    A=A-1
                ");
                }
            }
            emit_fmt_hack!(r"
                D=A
                @ARG
                M=D
            ");

            // jump to the function
            emit_fmt_hack!(r"
                @{callee_symbol}
                0;JMP   // complete the function call
            ");

            // declare return label
            emit_fmt_hack!(r"
                ({ret_label})
            ");

    }

    fn temp_to_stack(&mut self ,reg: TempRegister) {
        let register_offset = reg as usize;
        emit_fmt_hack!(r"
            @{0}
            D=M     // D = value of tmp register
        ", register_offset);

        self.d_to_stack();

    }
    fn stack_to_temp(&mut self, reg: TempRegister) {
        let register_offset = reg as usize;

        self.stack_to_d();  // A = pop stack

        emit_fmt_hack!{r"
            @{}
            M=D
        ", register_offset};
    }
    fn const_to_stack(&mut self, value: i16) {
        self.assign_a(value);   // A = value

        emit_fmt_hack!(r"
            D=A
        ");

        self.d_to_stack();
    }

    fn const_to_temp(&mut self, value: i16, reg: TempRegister) {
        let register_offset = reg as usize;
        self.assign_a(value);
        emit_fmt_hack!(r"
            D=A
            @{0}
            M=D
        ", register_offset);

    }


    // if the tmp register is zero, set it to LOGIC_TRUE, else set it to LOGIC_FALSE
    fn not_zero_tmp(&mut self, tmp: TempRegister) {
        let is_zero = self.symbol_generator.next_commented("is_eq");
        let not_zero = self.symbol_generator.next_commented("not_equal");
        let end = self.symbol_generator.next_commented("end");

        emit_fmt_hack!(r"
            @{0}
            D=M

            @{is_zero}
            D;JEQ       // if 0, goto is_equal, else goto not_equal

            ({not_zero})
                D={LOGIC_FALSE}
                @{end}      // goto end
                0;JMP
            ({is_zero})
                D={LOGIC_TRUE}
            ({end})

            // Temp = result
            @{0}
            M=D
        ", tmp as usize);
    }

    // tmp_out = tmp1 + tmp2
    fn add_temp(&mut self ,tmp_out: TempRegister, tmp1: TempRegister, tmp2: TempRegister) {
        let register1_offset = tmp1 as usize;
        let register2_offset = tmp2 as usize;
        let registerout_offset = tmp_out as usize;

        emit_fmt_hack!(r"
            @{0}
            D=M     // D = value of tmp1


            @{1}
            D=D+M   // D = value of value of tmp1 + value of tmp2

            @{2}
            M=D     // value of tmp_out = value of tmp1 + value of tmp2

        ",register1_offset, register2_offset, registerout_offset );
    }

    // tmp_out = tmp1 - tmp2
    fn sub_temp(&mut self, tmp_out: TempRegister, tmp1: TempRegister, tmp2: TempRegister) {
        let register1_offset = tmp1 as usize;
        let register2_offset = tmp2 as usize;
        let registerout_offset = tmp_out as usize;

        emit_fmt_hack!(r"
            @{0}
            D=M     // D = value of tmp1


            @{1}
            D=D-M   // D = value of value of tmp1 - value of tmp2

            @{2}
            M=D     // value of tmp_out = value of tmp1 - value of tmp2

        ",register1_offset, register2_offset, registerout_offset );
    }

}

#[repr(usize)]
enum TempRegister {
    T0 = 13,
    T1 = 14,
    T2 = 15,
}