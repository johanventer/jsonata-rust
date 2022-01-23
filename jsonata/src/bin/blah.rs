use jsonata::ast::Ast;
use jsonata::frame::Frame;
use jsonata::json::Number;
use jsonata::value::Value;
use jsonata::ValuePtr;

// sizeof Value: 232

pub fn main() {
    println!(
        "sizeof String: {}",
        std::mem::size_of::<std::string::String>()
    );
    println!("sizeof Box<str>: {}", std::mem::size_of::<Box<str>>());
    println!("sizeof &str: {}", std::mem::size_of::<&str>());
    println!("sizeof Frame: {}", std::mem::size_of::<Frame>());
    println!("sizeof Number: {}", std::mem::size_of::<Number>());
    println!("sizeof Value: {}", std::mem::size_of::<ValuePtr>());
    println!("sizeof ValueKind: {}", std::mem::size_of::<Value>());
    println!("sizeof Ast: {}", std::mem::size_of::<Ast>());
}
