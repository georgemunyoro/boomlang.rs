mod lexer;
mod parser;

fn main() {
    // let complex = "test = out(1)\nx = func{out(args.name)}\nout('Hello, World')";
    let simple = "x = 1";

    let mut ps = parser::Parser::new(simple);
    for node in ps.parse() {
        println!("{}", node);
    }
}
