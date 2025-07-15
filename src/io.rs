#[macro_export]
macro_rules! printf {
    ($($arg:tt)*) => {{
        use std::io::Write;
        print!($($arg)*);
        std::io::stdout().flush().unwrap();
    }};
}
