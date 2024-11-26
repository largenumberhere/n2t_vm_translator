use std::fs::File;
use std::sync::Arc;

/// Specifies a type that is able to emit hack assembly instructions.
pub trait EmitAsm<C> {
    /// Re-construct an emitter with any previous context.
    fn with_context(context: C, stream: Arc<File>) -> Self where Self: Sized {
        Self::new(stream)
    }

    /// Create a new emitter with default configuration.
    fn new(stream: Arc<File>) -> Self;

    /// Finalize work of emitter and snapshot the internal state.
    fn close(self) -> C;

    /// If bootstrapping is requested by user, this function does it.
    fn emit_init(&mut self);

    /// For any always required initialization.
    fn prelude(&mut self);

    /// Insert in a comment into the generated code.
    fn comment(&mut self, args: std::fmt::Arguments) -> std::io::Result<()>;

    /// Push the value given onto the stack.
    fn push_const(&mut self, val: i16);

    fn push_local_n(&mut self, offset: i16);
    fn push_arg_n(&mut self, offset: i16);
    fn push_temp_n(&mut self, offset: i16);
    fn push_static_n(&mut self, offset: i16);
    fn push_ptr_n(&mut self, offset: i16);
    fn pop_local_n(&mut self, offset: i16);
    fn pop_argument_n(&mut self, offset: i16);
    fn pop_temp_n(&mut self, offset: i16);
    fn pop_static_n(&mut self, offset: i16);
    fn pop_that_n(&mut self, offset: i16);
    fn pop_this_n(&mut self, offset: i16);
    fn call(&mut self, n_args: i16, symbol: &str);
    fn _return(&mut self);
    fn function(&mut self, n_vars: i16, symbol: &str);
    fn goto(&mut self, symbol: &str);
    fn or(&mut self);
    fn lt(&mut self);
    fn gt(&mut self);
    fn sub(&mut self);
    fn neg(&mut self);
    fn push_this_n(&mut self, offset: i16);
    fn push_that_n(&mut self, offset: i16);
    fn pop_ptr_n(&mut self, offset: i16);
    fn add(&mut self);
    fn eq(&mut self);
    fn and(&mut self);
    fn label(&mut self, symbol: &str);
    fn ifgoto(&mut self, symbol: &str);
    fn not(&mut self);

}

pub trait EContext : Default + Sized + Clone{

}