
use std::fs::File;
use std::io::{BufWriter};
use indoc::indoc;
use rusty_parser::str;
use crate::parser::Segment;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;

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
        // let lines_count = lines.clone().count();
        // let lines = lines.take(
        //     lines_count.checked_sub(3)
        //     .unwrap_or(lines_count)
        // );
        for line in lines {
            // if line is instruction, display the number
            if !line.starts_with("//") && !line.starts_with("(") && !line.is_empty() {
                //ignore newline characters
                //let mut line_bits = line.split("\n");
                //let line = line_bits.next().unwrap();

                let no = self.emitted_instructions_count;
                self.write_fmt(format_args!("{:90}//{:3}\n", line, no)).unwrap();
                self.emitted_instructions_count += 1;
            }

            else {
                self.write_fmt(format_args!("{:90}\n", line)).unwrap();
            }


        }
        // self.writer.write_fmt(format_args!("\n")).unwrap();
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
    fn stack_to_d(&mut self) {
        self.stack_to_a();
        self.emitln("D=A        ");
    }

    // take a from the local segment at offset 0 and place it on stack
    fn pop_local_a(&mut self) {

        self.emitln(indoc! {r"
            @LCL
            A=M // A = *LCL
            D=M // A = first local value
            @SP
            A=M // SP = *SP
            M=D // put first local on top of stack
            @SP
            A=M // SP = *SP
            M=M+1   // increase stack pointer
        "});

        // todo!("backwards");
        // self.emitln(indoc! {r"
        //     // emit_lcl_pop
        //     @LCL
        //     D=M
        //     M=D-1
        //     A=M
        //     D=M
        //     M=0 // zero the deallocted stack item for debugging convieneince
        //     A=D"});
    }


    // pub fn pop_argument_a(&mut self) {
    //     todo!("backwards");
    //     self.emitln(indoc! {r"
    //         // emit_arg_pop
    //         @ARG
    //         D=M
    //         M=D-1
    //         A=M
    //         D=M
    //         M=0 // zero the deallocted stack item for debugging convieneince
    //         A=D"});
    // }

    // take the argument at offset n and place it on the stack
    pub fn pop_argument_n(&mut self, n: i16) {
        self.emitln(indoc! {r"
            @ARG
            D=M // D = *ARG"});
        self.emitln(&format!("@{}   //A=offset", n));
        self.emitln(indoc! {r"
            D=D+A   // D = n-th argument's address
            D=M     // D = n-th argument
            @SP
            A=M     // A = *SP
            M=D     // top of stack = n-th argument
            @SP
            A=M
            M=M+1   // increase stack pointer"});
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

    // tested
    pub fn push_const(&mut self, val: i16) {
        self.assign_a(val);
        self.emitln(indoc! {r"
            D=A
            @SP
            A=M         // A=*sp
            M=D         // **sp = D
            @SP
            M=M+1       //*sp = *sp+1
        "});

        // self.assign_a(val);
        // self.a_to_stack();
        // self.emitln("");
    }

    fn assign_a(&mut self, value: i16) {
        let value_string = format!(indoc!{r"
            @{} // A = {}"},value, value);

        self.emitln(&value_string.as_str());
    }

    fn push_local(&mut self) {
        self.stack_to_a();
        self.emit_push_local_a();
        self.emitln("");
    }

    pub fn push_local_n(&mut self, n: i16) {
        if n == 0 {
            self.push_local();
        } else {
            todo!("push local n")
        }
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
    }

    // take a value from the that segment at offset n and place it on the stack
    pub fn pop_that_n(&mut self, n: i16) {
        self.emitln(indoc! {r"
            @THAT
            D=M         // D = *THAT"});
        self.emitln(&format!("@{}   //A=offset", n));
        self.emitln(indoc! {r"
            A=A+D   // A = address of 'that' number n
            D=M     // D = value of 'that' numner n
            @SP
            A=M // A = *SP
            M=D // top of stack is set to 'that' number n
            @SP
            A=M
            M=M+1   // increase stack pointer"});
    }

    pub fn pop_temp_n(&mut self, n: i16) {
        self.emitln(indoc! {r"
            @TEMP
            D=M // D = *TEMP"});

        self.emitln(&format!("@{}   //A=offset", n));
        self.emitln(indoc! {r"
            A=A+D   // A = address of 'TEMP' number n
            D=M     // D = value of 'TEMP' number n
            @SP
            A=M // A = *SP
            M=D // top of stack is set to 'TEMP' number n
            @SP
            A=M
            M=M+1   // increase stack pointer"});
    }

    pub fn pop_this_n(&mut self, n: i16) {
        self.emitln(indoc! {r"
            @THIS
            D=M // D = *THIS
        "});

        self.emitln(&format!("@{}   //A=offset", n));
        self.emitln(indoc! {r"
            A=D+A   // A = offset of nth this
            D=M     // A = nth this value
            @SP
            A=M // SP = *SP
            M=D // put first argument on top of stack
            @SP
            A=M // SP = *SP
            M=M+1   // increase stack pointer"});
    }

    // pub fn pop_this_n(&mut self, n: i16) {
    //     todo!()
    // }

    // take the value at top of arguments segment and place it on the stack
    // pub fn emit_pop_this(&mut self) {
    //     self.emitln(indoc! {r"
    //         @THIS
    //         A=M // A = *THIS
    //         D=M // A = first argument value
    //         @SP
    //         A=M // SP = *SP
    //         M=D // put first argument on top of stack
    //         @SP
    //         A=M // SP = *SP
    //         M=M+1   // increase stack pointer
    //     "});
    // }

    // take value off the stack and write it to the 'that' segment at offset n
    pub fn push_that_n(&mut self, n: i16) {
        todo!()
        // self.emitln(indoc! {r"
        //     @SP
        //     A=M     // A = *SP
        //     M=M-1   // decrease stack pointer
        // "});
        // self.emitln(&format!("@{} //A=offset", n));
        // self.emitln(indoc! {r"
        //     D=A
        //     @THAT
        //     A=M     // A = *THAT
        //     D=A+D   // D = address of 'that' number n
        //
        //     @SP
        //     A=M     // A = *SP
        //     A=M     // A = top of stack
        //     M=D     // Write address of 'that' number n to top of stack
        //
        //
        //     A=M     // A = address of 'that' number n
        //
        //     @SP
        //     A=M     // A = *SP
        //     M=0     // zero top of stack
        // "});


        // self.emitln(indoc! {r"
        //
        // "});
    }

    pub fn push_arg_n(&mut self, n:i16) {
        todo!("push arg n")
    }

    pub fn push_temp_n(&mut self, n: i16) {
        todo!("push temp n")
    }

    pub fn push_static_n(&mut self, n: i16) {
        todo!("push static n")
    }

    pub fn push_this_n(&mut self, n: i16) {
        todo!("push this n");
    }

    pub fn push_ptr_n(&mut self, n: i16) {
        todo!("push ptr n")
    }

    pub fn pop_static_n(&mut self, n: i16) {
        todo!("pop static n")
    }

    pub fn pop_local_n(&mut self, n: i16) {
        if n==0 {
            self.pop_local_a();
            return;
        }

        todo!("pop local n")
    }

    pub fn pop_ptr_n(&mut self, n: i16) {
        todo!("pop ptr n")
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
}