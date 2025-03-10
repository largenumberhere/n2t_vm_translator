//! An hack assembly emitter that prioritizes small assembly size

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

pub struct CompactEmitter {
    writer: BufWriter<Arc<File>>,
    symbol_generator: SymbolGenerator,
    emitted_instructions_count: usize,
    func_emitter: FuncEmitter
}



impl EmitAsm<CEmitterContext> for CompactEmitter {
    fn with_context(context: CEmitterContext, stream: Arc<File>) -> Self {
        Self::with_context(context, stream)
    }

    fn new(stream: Arc<File>) -> Self {
        Self::new(stream)
    }

    fn close(self) -> CEmitterContext {
        self.close()
    }

    fn emit_init(&mut self) {
        self.emit_init();
    }

    fn prelude(&mut self) {
        self.prelude();
    }

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
        self.call(n_args, symbol)
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
pub struct CEmitterContext {
    emitted_instructions_count: usize,
    func_emitter: FuncEmitter
}

impl Default for CEmitterContext {
    fn default() -> Self {
        Self {
            emitted_instructions_count: 0,
            func_emitter: FuncEmitter::new()
        }
    }
}

impl EContext for CEmitterContext {}
const LOGIC_TRUE: i16 = -1;
const LOGIC_FALSE: i16 = 0;

impl CompactEmitter {
    pub fn close(self) -> CEmitterContext {
        return CEmitterContext {
            emitted_instructions_count: self.emitted_instructions_count,
            func_emitter: self.func_emitter,
        };
    }

    pub fn new(stream: Arc<File>) -> Self {
        Self {
            writer: BufWriter::new(stream),
            symbol_generator: SymbolGenerator::new(),
            emitted_instructions_count: 0,
            func_emitter: FuncEmitter::new()
        }
    }

    pub fn with_context(emitter_context: CEmitterContext, stream: Arc<File>) -> Self {
        Self {
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

    pub fn prelude(&mut self) {
        let end = self.symbol_generator.next_commented("end_prelude");
        // skip the following in the case that init is not enabled
        emit_fmt_hack!(r"
            @{end}
            0;JMP
        ");

        self.emit_label_start("neg_proc");
        emit_fmt_hack!(r"
            // expects a return address passed in D
            @R15
            M=D     // stow return addres
            @SP
            A=M-1
            D=M     // D = value of item in stack
            D=-D    // negate D
            M=D     // write D back to stack

            @R15
            A=M
            0;JMP   // return
        ");

        self.emit_label_start(end.as_str());
    }

    pub fn emit_init(&mut self) {
        emit_hack! {r"
            @Sys.init
            0;JMP

            // end of program - halt
            @_L_DEADLOOP
            (_L_DEADLOOP)
            0;JMP

        "};
        self.emitln("");

        // // initalize segments
        // // 1. clear the memory if needed
        //
        // let asm: &str= hack_str!{r"
        //     // // emit_init: write 256 to stack pointer
        //     // @256
        //     // D=A
        //     // @0
        //     // M=D // todo: LCL, ARG, THIS, THAT"};
        //
        // // 2. setup segment pointers
        // self.emitln(asm);
        // self.emitln("");
        // self.writer.write_fmt(format_args!("{}\n\n", asm))
        //     .unwrap();

        // panic!("{}", a);
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
    // clobbers A,D
    // tested
    fn stack_to_a(&mut self) {
        emit_hack! {r"
            @SP
            M=M-1       // Decrement stack pointer
            A=M         // A = Stack pointer
            A=M         // D = old top of stack
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
        // self.assign_a(val);
        // emit_hack! {r"
        //     D=A
        //     @SP
        //     M=M+1       // increase stack pointer
        //     A=M-1       // get stack address to write to
        //     M=D         // stack top = D
        // "};
        // self.emitln("");

        emit_fmt_hack!(r"
            @{val}
            D=A
            @SP
            A=M
            M=D
            @SP
            M=M+1
        ");
        self.emitln("");
    }

    // tested
    pub fn add(&mut self) {
        // D = pop1
        self.stack_to_d();

        emit_hack! {r"
            @SP         // add last item in stack to D
            M=M-1       // decrement stack pointer
            A=M         // A = stack pointer
            A=M         // A = pop2
            D=D+A       // D = pop1 + pop2
            A=D         // A = result
        "};
        self.a_to_stack();
        self.emitln("");
    }



    // seems to check out
    pub fn eq(&mut self) {
        let is_equal = self.symbol_generator.next_commented("is_eq");
        let not_equal = self.symbol_generator.next_commented("not_equal");
        let end = self.symbol_generator.next_commented("end");
        self.stack_to_d();
        emit_fmt_hack!{r"
            // D = value from stack
            @SP
            A=M-1       // A = address of top item in stack
            A=M         // A = top value from stack
            D=D-A       // D = difference of values from stack
            @{is_equal}
            D;JEQ       // if pop1 == pop2, goto is_equal, else goto not_equal

            ({not_equal})
                D={LOGIC_FALSE}
                @{end}      // goto end
                0;JMP
            ({is_equal})
                D={LOGIC_TRUE}
            ({end})
            @SP
            A=M-1       // grab pointer to top item in stack
            M=D         // write to stack
        "};
        self.emitln("");
    }

    // tested!
    pub fn sub(&mut self) {
        self.stack_to_d();
        emit_hack! {r"
            @SP
            A=M-1       // get pointer to top item in stack
            A=M         // A = value from stack
            D=A-D       // D = pop2 - pop1
            @SP
            A=M-1
            M=D         // write result on stack"
        };

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
        let return_label = self.symbol_generator.next_commented("neg_ret");
        emit_fmt_hack!(r"
            @{return_label}
            D=A
            @neg_proc
            0;JMP
        ");
        self.emit_label_start(return_label.as_str());
        // emit_hack! {r"
        //     @SP
        //     A=M-1           // A = pointer to last item on stack
        //     D=M             // D = last item on stack
        //     D=-D            // negate value
        //     M=D             // write result on stack
        // "};
        // self.emitln("");
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

        if n_vars > 0 {
            emit_hack! {r"
                @LCL
                A=M
                D=M
                @R14
                M=D
            "};

            // zero the locals
            for _ in 0..n_vars {
                emit_hack! {r"
                    @R14
                    M=0

                    @R14
                    M=M+1
                "};
            }
        }
    }



    // tested
    pub fn _return(&mut self) {
        // move return value to args[0] of caller
        // &args[0] = return
        emit_hack! {r"
            // return
            // insert return value to arg[0] of caller
                // load caller's argument segment ARG= *LCL-3
                @LCL
                A=M     // A = address of first local
                A=M
                A=A-1
                A=A-1
                A=A-1   // A = address of caller's ARG
                A=M
                D=M     // A = caller's ARG
                @ARG
                A=M
                M=D     // write segment value

                // pop item off stack
                @SP
                M=M-1       // Decrement stack pointer
                A=M         // A = Stack pointer
                D=M         // D = old top of stack
                // write item to arg0
                @ARG
                A=M         // address of first argument
                M=D         // arg0 = item 1

            // discard stack
            @ARG
            D=M+1           // end of caller's stackframe
            @SP
            M=D             // set stack pointer to just underneath previous function stackframe

            // stow return address address
            @LCL
            A=M
            A=A-1
            A=A-1
            A=A-1
            A=A-1
            A=A-1   // D = address of return address
            D=A     // D = address of return address
            @R13
            M=D     // mem[13] = return address address

            // recover caller's segment pointers
                // load caller's THAT segment THAT = *LCL-1
                @LCL
                A=M     // A = address of first local
                A=A-1   // A = address of caller's THAT
                D=M     // D = caller's semgnet value
                @THAT
                M=D

                // load caller's THIS segment THIS = *LCL-2
                @LCL
                A=M     // A = address of first local
                A=A-1   // A = address of caller's THAT
                A=A-1   // A = address of caller's THIS
                D=M     // D = caller's semgnet value
                @THIS
                M=D

                // load caller's ARG segment ARG = *LCL-3
                @LCL
                A=M     // A = address of first local
                A=A-1   // A = address of caller's THAT
                A=A-1   // A = address of caller's THIS
                A=A-1   // A = address of caller's ARG
                D=M     // D = caller's semgnet value
                @ARG
                M=D

                // load caller's LCL segment
                @LCL
                A=M     // A = address of first local
                A=A-1   // A = address of caller's THAT
                A=A-1   // A = address of caller's THIS
                A=A-1   // A = address of caller's ARG
                A=A-1   // A = address of caller's LCL
                D=M     // D = caller's semgnet value
                @LCL
                M=D

            // jump to return address
            @R13
            A=M
            A=M         // A = return address
            0;JMP       // jump to return address
        "};
        self.emitln("");
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

    pub fn call(&mut self, n_args: i16, callee_symbol: &str) {
        let caller_return = fmt_hack!("{}$ret.{}", callee_symbol, self.func_emitter.call());

        // set Ram[13] to address of first argument
        emit_hack! {r"
            // save stack address of first argument
            @SP
            D=M
        "};
        self.assign_a(n_args);
        emit_hack! {r"
            D=D-A
            @R13
            M=D
        "};

        emit_hack! {r"// save a return address"};
        emit_fmt_hack!("@{}", caller_return.as_str());
        emit_hack! {r"
            D=A
            @SP
            A=M
            M=D
            @SP
            M=M+1
        "};

        // save caller stackframe
        self.segment_pointer_to_stack(Segment::Local);
        self.segment_pointer_to_stack(Segment::Argument);
        self.segment_pointer_to_stack(Segment::This);
        self.segment_pointer_to_stack(Segment::That);


        emit_hack!{r"
            // setup segment pointers for callee
                // ARG = address of first argument passed
                @R13
                D=M
                @ARG
                M=D

                // LCL = start of caller's stackframe
                @SP
                D=M
                @LCL
                A=M
                M=D
            // jump

        "};

        // jump
        emit_fmt_hack!("@{}", callee_symbol);
        emit_hack! {r"
            0;JMP
        "};

        // declare callee return address
        self.emit_label_start(caller_return.as_str());

        self.emitln("");
    }
}
