
use project_proc_macros::hello_fn_proc_macro;
#[test]
fn hello_proc_macro_test() {


    assert_eq!("Hello world", hello_fn_proc_macro!());
}

