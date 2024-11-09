use std::cmp::PartialEq;
use std::fs::File;
use std::io::{BufWriter};
use indoc::indoc;
use rusty_parser::str;
use crate::parser::Segment;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
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

pub struct Emitter <W: IoWrite>{
    writer: BufWriter<W>,
    symbol_generator: SymbolGenerator,
    emitted_instructions_count: usize
}



impl<W: IoWrite> Emitter<W> {
    pub fn new(stream: W) -> Emitter<W> {
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
        // initalize segments
        // 1. clear the memory if needed

        let asm: &str= indoc!{r"
            // emit_init: write 256 to stack pointer
            @256
            D=A
            @0
            M=D // todo: LCL, ARG, THIS, THAT"};

        // 2. setup segment pointers
        self.emitln(asm);
        self.emitln("");
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
            @SP
            A=M         // A = Stack pointer
            D=M         // D = old top of stack
            @SP
            A=M         // A = stack pointer
            M=0         // zero top of stack for debugging convieneince
            A=D         // A = old top of stack     // end stack_to_a"});
    }

    // tested
    // clobbers, A, D
    fn stack_to_d(&mut self) {
        self.stack_to_a();
        self.emitln("D=A        ");
    }



    // push the item in register A onto the stack
    fn a_to_stack(&mut self) {
        self.emitln(indoc! {r"
            D=A
            @SP
            A=M         // A=*sp
            M=D         // **sp = D
            @SP
            M=M+1       //*sp = *sp+1"});
    }


    fn emit_push_local_a(&mut self) {
        self.emitln(indoc! {r"
            // emit_push_a
            D=A
            @LCL
            A=M     // A=*LCL
            M=D     // **LCL = D
            @0
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
            A=M         // A=*sp
            M=D         // **sp = D
            @SP
            M=M+1       //*sp = *sp+1"});

        // self.assign_a(val);
        // self.a_to_stack();
        self.emitln("");
    }




    // tested
    pub fn add(&mut self) {

        //let start = self.symbol_generator.next_commented("add_start");
        let end = self.symbol_generator.next_commented("add_end");
        //self.emit_label_start(start.as_str());

        // D = pop1
        self.stack_to_d();

        self.emitln(indoc!{r"
            @SP         // add last item in stack to D
            M=M-1       // decrement stack pointer
            @SP
            A=M         // A = stack pointer
            A=M         // A = pop2
            D=D+A       // D = pop1 + pop2
            @SP         // clear last item
            A=M         //*sp
            M=0         //**sp = 0
            A=D         // A = result"});

        self.a_to_stack();
        self.emit_label_start(end.as_str());
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
            A=M
            M=D         // write result on stack
            @SP
            M=M+1       // increase stack pointer"});

        self.emitln("");
        // todo!("eq");
    }

    // tested!
    pub fn sub(&mut self) {
        self.stack_to_d();
        self.emitln(indoc! {r"
            @SP
            M=M-1       // decrease stack pointer
            @SP
            A=M
            A=M         // A = value from stack
            D=A-D       // D = pop2 - pop1
            @SP
            A=M
            M=D         // write result on stack
            @SP
            M=M+1       // increase stack pointer"});

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
            M=M-1       // decrease stack pointer
            A=M         // A = *SP
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
            A=M         // A = *SP
            M=D         // **SP = val
            @SP
            M=M+1       // *SP ++ increase stack pointer"});

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
            A=M         // A = pop2
            D=A-D       // pop2 - pop1
            "});
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
    }

    // tested
    pub fn neg(&mut self) {
        self.stack_to_d();
        self.emitln(indoc! {r"
            D=-D        // calculate
            @SP
            A=M
            M=D         // write reusult to stack
            @SP
            M=M+1       // increase stack pointer"});
        self.emitln("");
    }

    // tested
    pub fn or(&mut self) {
        // D = pop1
        self.stack_to_d();
        self.emitln(indoc! {r"
            @SP
            M=M-1       // decrease stack pointer
            A=M
            M=M|D       // write result to stack
            @SP
            M=M+1       // increase stack pointer"});
        self.emitln("");
    }

    // tested
    pub fn not(&mut self) {
        self.stack_to_d();
        self.emitln(indoc! {r"
            D=!D        // calculate
            @SP
            A=M
            M=D         // write reusult to stack
            @SP
            M=M+1       // increase stack pointer"});
        self.emitln("");
    }

    pub fn and(&mut self) {
        // D = pop1
        self.stack_to_d();
        self.emitln(indoc! {r"
            @SP
            M=M-1       // decrease stack pointer
            A=M
            M=M&D       // write result to stack
            @SP
            M=M+1       // increase stack pointer"});
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
            Segment::Pointer => { "THIS"
                // match offset {
                //     0 => "THIS",
                //     1 => "THAT",
                //     _=> panic!("Pointer only takes the arguments 1 or 0")
                // }
            }
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
            // M=M+1       // Increase stack pointer
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
            A=M     // A = stack address
            M=D     // write value to stack
            @SP
            M=M+1   // increase stack pointer
        "});
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
}