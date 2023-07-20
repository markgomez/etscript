use std::{
    env,
    ffi::{CStr, CString},
    fs,
    io::{self, Write},
    process,
};

fn repl() {
    println!("ETscript 0.1.0 (press Control-C to quit)");

    loop {
        let mut line = String::new();

        print!(">>> ");
        io::stdout().flush().ok();
        io::stdin().read_line(&mut line).ok();
        if let Some(byte) = line.as_bytes().last() {
            if *byte as char == '\n' {
                line.pop();
            }
        }
        if let Some(byte) = line.as_bytes().last() {
            if *byte as char == '\r' {
                line.pop();
            }
        }
        let c_string = CString::new(line).unwrap();
        unsafe {
            let result = etscript_core::interpret(c_string.as_ptr());
            let str = CStr::from_ptr((*result).value).to_str().unwrap();

            println!("{str}");
            etscript_core::free_result(result);
        };
    }
}

fn file(path: &str) {
    let source = fs::read_to_string(path).expect("Contents of a file should have been read.");
    let c_string = CString::new(source).unwrap();
    unsafe {
        let result = etscript_core::interpret(c_string.as_ptr());
        let str = CStr::from_ptr((*result).value).to_str().unwrap();

        println!("{str}");
        etscript_core::free_result(result);
    };
}

fn main() {
    match env::args().count() {
        1 => repl(),
        2 => file(&env::args().last().unwrap()),
        _ => {
            eprintln!("Usage: etscript [file]");
            process::exit(64);
        }
    }
}
