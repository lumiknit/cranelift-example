//! The main file of cranelift example.

pub mod compiler;
pub mod expr;
pub mod runtime;

fn main() {
    // Usage: cranelift-expr <source file name>
    // Read source file whose name is passed as argument

    let filename = std::env::args()
        .nth(1)
        .expect("Usage: cranelift-expr <filename>");

    let content = std::fs::read_to_string(&filename).expect("Failed to read the file");

    // Parse the file content into an expressions

    let expr = expr::parse_expr(content.as_str()).expect("Failed to parse the expression");
    println!("Parsed expression: {:?}", expr);

    // Compile the expression into a function using compiler

    let func = compiler::compile_expr(&expr).expect("Failed to compile the expression");

    // The program will takes 4 numbers as input.
    // Read a line from stdin and split it into 4 numbers.

    let mut buf = String::new();
    println!("Enter 4 numbers: ");
    std::io::stdin()
        .read_line(&mut buf)
        .expect("Failed to read the input");
    // Split and map the input to i32
    let mut inputs: Vec<i32> = buf
        .split_whitespace()
        .map(|s| s.parse().expect("Failed to parse the input"))
        .collect();
    while inputs.len() < 4 {
        // Fill 0
        inputs.push(0);
    }

    // Call the compiled function.

    let result = func.call(inputs[0], inputs[1], inputs[2], inputs[3]);
    println!("Result: {}", result);
}
