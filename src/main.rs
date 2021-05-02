
use std::io;

pub mod parser;
pub mod term;



fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let result = parser::parse(input.as_str())
        .and_then(|t| {
            let mut context = vec![];
            term::infer(&mut context, t)
        });
    match result {
        Ok(term) => println!("Type is: {:?}", term),
        Err(message) => println!("{}", message)
    };

    Ok(())
}
