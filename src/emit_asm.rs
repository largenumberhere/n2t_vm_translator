use std::cmp::PartialEq;
use std::fs::File;
use std::io::{BufWriter};
use indoc::indoc;
use rusty_parser::str;
use crate::parser::Segment;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::sync::Arc;
use crate::parser::Segment::This;


struct SymbolGenerator {
    next_id: usize,

}
impl SymbolGenerator {
    fn new() -> SymbolGenerator {
        SymbolGenerator {
            next_id: 0,

        }
    }

    fn next_commented(&mut self, label_start: &str) -> String  {

        assert!(label_start.split_whitespace().skip(1).next().is_none(), "no whitespace allowed in labels");
        let mut out = String::new();
        out.write_fmt(format_args!("_{}_L{}", label_start, self.next_id)).unwrap();

        self.next_id += 1;

        return out;
    }
}

pub struct Emitter {
    writer: BufWriter<Arc<File>>,
    symbol_generator: SymbolGenerator,
    emitted_instructions_count: usize
}


impl Emitter {
    pub fn new(stream: Arc<File>) -> Emitter {
        Emitter {
            writer: BufWriter::new(stream),
            symbol_generator: SymbolGenerator::new(),
            emitted_instructions_count: 0
        }
    }

    pub fn comment(&mut self, args: std::fmt::Arguments) -> std::io::Result<()> {
        self.write_fmt(args)
    }
    fn write_fmt(&mut self, args: std::fmt::Arguments) -> std::io::Result<()> {
        self.writer.write_fmt(args)
    }

    pub fn emit_init(&mut self) {
    self.emitln(indoc! {r"
        @Sys.init
        0;JMP
    "});
        // // initalize segments
        // // 1. clear the memory if needed
        //
        // let asm: &str= indoc!{r"
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
            // if line is instruction, display the instruction number for debugging convenience
            if !line.starts_with("//") && !line.starts_with("(") && !line.is_empty() {
                let no = self.emitted_instructions_count;
                self.write_fmt(format_args!("{:90}//{:3}\n", line, no)).unwrap();
                self.emitted_instructions_count += 1;
            }
            else {
                self.write_fmt(format_args!("{:90}\n", line)).unwrap();
            }
        }
    }

    fn emit_label_start(&mut self, symbol: &str) {
        self.emitln(&format!("({})", symbol))
    }

    // puts the last item into register A
    // clobbers A,D
    // tested
    fn stack_to_a(&mut self) {
        self.emitln(indoc! {r"
            @SP
            M=M-1       // Decrement stack pointer
            A=M         // A = Stack pointer
            A=M         // D = old top of stack"});
    }

    // tested
    // clobbers, A, D
    fn stack_to_d(&mut self) {
        self.emitln(indoc! {r"
            @SP
            M=M-1       // Decrement stack pointer
            A=M         // A = Stack pointer
            D=M         // D = old top of stack"});
    }



    // push the item in register A onto the stack
    fn a_to_stack(&mut self) {
        self.emitln(indoc! {r"
            D=A
            @SP
            M=M+1       // increase stack pointer
            A=M-1       // get top of stack
            M=D         // top of stack = D"});
    }


    fn emit_push_local_a(&mut self) {
        self.emitln(indoc! {r"
            // emit_push_a
            D=A
            @LCL
            A=M     // A=*LCL
            M=D     // **LCL = D
            @SP
            M=M+1   //*LCL = *LCL+1"});
    }

    fn assign_a(&mut self, value: i16) {
        let value_string = format!(indoc!{r"
            @{} // A = {}"},value, value);

        self.emitln(&value_string.as_str());
    }

    // tested
    pub fn push_const(&mut self, val: i16) {
        self.assign_a(val);
        self.emitln(indoc! {r"
            D=A
            @SP
            M=M+1       // increase stack pointer
            A=M-1       // get stack address to write to
            M=D         // stack top = D"});
        self.emitln("");
    }

    // tested
    pub fn add(&mut self) {
        // D = pop1
        self.stack_to_d();

        self.emitln(indoc!{r"
            @SP         // add last item in stack to D
            M=M-1       // decrement stack pointer
            A=M         // A = stack pointer
            A=M         // A = pop2
            D=D+A       // D = pop1 + pop2
            A=D         // A = result"});
        self.a_to_stack();

        self.emitln("");
    }

    // tested
    pub fn eq(&mut self) {
        // todo!("needs reviewing");
        let is_eq = self.symbol_generator.next_commented("is_eq");
        let end = self.symbol_generator.next_commented("end");
        // D = pop1
        self.stack_to_d();
        self.emitln(indoc!(r"
            @SP
            M=M-1       // decrease stack pointer
            @SP
            A=M
            A=M         // A = pop2
            D=A-D       // D = pop2 - pop1"));

        self.emitln(&format!("@{}", is_eq));
        self.emitln(indoc!(r"
            D;JEQ       // jump to is_eq if pop1 == pop2, else fallthrough
            D=0         // D = 0 designating false"));
        self.emitln(&format!("@{}", end));
        self.emitln(indoc!(r"
            0;JMP       // jump to end"));

        self.emit_label_start(is_eq.as_str());
        self.emitln(indoc!(r"
            D=-1         // D = 1 designating true"));
        self.emit_label_start(end.as_str());
        self.emitln(indoc! {r"
            @SP
            M=M+1       // increase stack pointer
            A=M-1       // get pointer to top of stack
            M=D         // write result on stack"});

        self.emitln("");
    }

    // tested!
    pub fn sub(&mut self) {
        self.stack_to_d();
        self.emitln(indoc! {r"
            @SP
            A=M-1       // get pointer to top item in stack
            A=M         // A = value from stack
            D=A-D       // D = pop2 - pop1
            @SP
            A=M-1
            M=D         // write result on stack"});

        self.emitln("");
    }

      // tested!
    pub fn lt(&mut self) {
        let is_lt = self.symbol_generator.next_commented("is_lt");
        let is_not_lt = self.symbol_generator.next_commented("is_not_lt");
        let lt_end = self.symbol_generator.next_commented("lt_end");

        // D = pop1
        self.stack_to_d();
        self.emitln(indoc!{r"
            @SP
            A=M-1       // A = address of top item on stack
            A=M         // A = pop2
            D=A-D       // D = pop2 - pop1"});

        self.emitln(&format!("@{}   //A = is_lt", is_lt));
        self.emitln(indoc! {r"
            D;JLT       // if pop1  < pop2 then goto is_lt, else fallthrough"});
        self.emit_label_start(is_not_lt.as_str());
        self.emitln(indoc! {r"
            D=0"});
        self.emitln(&format!("@{}", lt_end));
        self.emitln(indoc! {r"
            0;JMP"});

        self.emit_label_start(is_lt.as_str());
        self.emitln(indoc!{r"
            D=-1"});

        self.emit_label_start(lt_end.as_str());
        self.emitln(indoc!{r"
            @SP
            A=M-1
            M=D         // write value to top of stack"});

        self.emitln("");
    }

    // tested
    pub fn gt(&mut self) {
        let end = self.symbol_generator.next_commented("gt_end");
        let is_gt = self.symbol_generator.next_commented("is_gt");

        // D = pop1
        self.stack_to_d();
        self.emitln(indoc! {r"
            @SP
            M=M-1       // decrease stack pointer
            A=M         // A = address of stack top
            D=M-D       // pop2 - pop1"});
        self.emitln(&format!("@{}   //A = is_gt", is_gt));
        self.emitln(indoc!{r"
            D;JGT   // if pop2 > pop1 then goto is_gt, else
            D=0     // not gt"});
        self.emitln(&format!("@{}   //A = end", end));
        self.emitln(indoc!(r"
            0;JMP"));
        self.emit_label_start(is_gt.as_str());
        self.emitln(indoc! {r"
            D=-1    //yes gt"});
        self.emit_label_start(end.as_str());
        self.emitln(indoc! {r"
            A=D"});
        self.a_to_stack();
        self.emitln("");
    }

    // tested
    pub fn neg(&mut self) {
        self.emitln(indoc! {r"
            @SP
            A=M-1           // A = pointer to last item on stack
            D=M             // D = last item on stack
            D=-D            // negate value
            M=D             // write result on stack"});
        self.emitln("");
    }

    // tested
    pub fn or(&mut self) {
        // D = pop1
        self.stack_to_d();
        self.emitln(indoc! {r"
            @SP
            A=M-1       // pointer to last item on stack
            M=M|D       // write result to stack"});
        self.emitln("");
    }

    // tested
    pub fn not(&mut self) {
        self.emitln(indoc! {r"
            @SP
            A=M-1       // A = pointer to last item on stack
            D=M         // D = value from stack

            D=!D        // calculate
            M=D         // write result to stack"});
        self.emitln("");
    }

    pub fn and(&mut self) {
        // D = first item
        self.stack_to_d();
        self.emitln(indoc! {r"
            @SP
            A=M-1       // A = 2nd item from stack
            M=M&D       // write result to stack, overwriting 2nd item"});
        self.emitln("");
    }

    fn segment_symbol_str(&self, segment: Segment, offset: i16) -> &str {
        return match segment {
            Segment::Local => {"LCL"}
            Segment::Constant => {unreachable!("Constant is not a real segment")}
            Segment::Argument => {"ARG"}
            Segment::Temp => {"TMP"}
            Segment::Static => {"STATIC"}
            Segment::That => {"THAT"}
            Segment::This => {"THIS"}
            Segment::Pointer => { "THIS"}
        };
    }

    // move the value at offset n from the segment onto the stack
    fn pop_non_stack_segment(&mut self, segment: Segment, offset:i16) {
        // let not_temp_segment;
        let segment_symbol = self.segment_symbol_str(segment, offset);

        // D = address of segment start
        match segment {
            Segment::Pointer => {
                self.emitln(&format!("@{}", segment_symbol));
                self.emitln(indoc! {r"
                D=A         // D = segment start"});
            },
            Segment::Temp => {
                self.emitln(indoc! {r"
                @5
                D=A
            "});
            },
            _=> {
                self.emitln(&format!("@{}", segment_symbol));
                self.emitln(indoc! {r"
                D=M         // D = segment start"});
            }
        }

        // A = segment offset
        self.assign_a(offset);
        self.emitln(indoc! {r"
            D=D+A      // D = pointer to destination in segment
            @SP
            A=M         // A = stack pointer
            M=D         // write segment destination to stack
            @SP
            M=M+1       // increase stack pointer"});

        self.emitln(indoc! {r"
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
            M=M-1       // Decrease stack pointer"});

        self.emitln("");

    }

    // move the value from the stack to the segment at offset n
    fn push_non_stack_segment(&mut self, segment: Segment, offset:i16) {

        let segment_symbol = self.segment_symbol_str(segment, offset);

        // D = address of segment start
        match segment {
            Segment::Pointer => {
                self.emitln(&format!("@{}", segment_symbol));
                self.emitln(indoc! {r"
                D=A         // D = segment start"});
            },
            Segment::Temp => {
                self.emitln(indoc! {r"
                @5
                D=A
            "});
            },
            _=> {
                self.emitln(&format!("@{}", segment_symbol));
                self.emitln(indoc! {r"
                D=M         // D = segment start"});
            }
        }

        // A = segment offset
        self.assign_a(offset);
        self.emitln(indoc! {r"
            A=A+D   // A = pointer to read from
            D=M     // D = value in segment
            @SP
            M=M+1   // increase stack pointer
            A=M-1   // A = value at top of stack
            M=D     // write value to stack"});
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

    pub fn push_arg_n(&mut self, n:i16) {
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

    const user_label_prefix: &'static str = "user_";
    pub fn label(&mut self, symbol: &str) {
        self.emitln(&format! {r"({}{})", Self::user_label_prefix, symbol});
    }

    // jump to the symbol if stack top > 0
    pub fn ifgoto(&mut self, symbol: &str) {
        self.stack_to_d();
        self.emitln(&format! {r"@{}{}", Self::user_label_prefix, symbol});
        self.emitln(indoc!{r"
            D;JNE"});
        self.emitln("");
    }

    pub fn goto(&mut self, symbol: &str) {
        self.emitln(&format! {r"@{}{}", Self::user_label_prefix, symbol});
        self.emitln(indoc!{r"
            0;JMP"});
        self.emitln("");
    }

    // a function declaration
    pub fn function(&mut self, n_vars: i16, symbol: &str) {
        // // initialize locals of count n_vars
        // self.emitln(indoc! {r"
        //     @LCL
        // "});
        // for i in 0..n_vars {
        //     self.emitln(indoc!{r"
        //         M=0     // zero the local variable
        //         A=A+1   // increment counter
        //     "});
        // }
        // inject label. Todo: make it comply with mangling rules
        self.emit_label_start(symbol);

    }

    // tested
    pub fn _return(&mut self) {
        // move return value to args[0] of caller
        // &args[0] = return
        self.emitln(indoc! {r"
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
        "});







        // todo!();
    }

    pub fn call(&mut self, n_vars: i16, symbol: &str) {
        // todo!();
    }
}