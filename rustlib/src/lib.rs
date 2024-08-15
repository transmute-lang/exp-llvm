use std::io::{stdout, Write};

#[no_mangle]
pub extern "C" fn rustlib_print(i: i32) {
    print!("{i}");
    stdout().flush().unwrap()
}
