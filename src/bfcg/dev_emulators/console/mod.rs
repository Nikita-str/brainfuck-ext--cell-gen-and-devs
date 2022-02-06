mod console_inner;
pub mod console_utf8;
pub mod console_num;
pub mod console_ascii;

pub use console_utf8::DevConsoleUtf8;
pub use console_num::DevConsoleNum;
pub use console_ascii::DevConsoleAscii;