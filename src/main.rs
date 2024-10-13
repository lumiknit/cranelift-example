pub mod compiler;
pub mod expr;
pub mod runtime;

fn main() {
    // Read the first argument, the file name
    let filename = std::env::args()
        .nth(1)
        .expect("Usage: cranelift-expr <filename>");

    // Read the file content
    let content = std::fs::read_to_string(&filename).expect("Failed to read the file");

    // Parse the content into Expr
    let expr = expr::parse_expr(content.as_str()).expect("Failed to parse the expression");

    println!("Parsed expression: {:?}", expr);

    // Compile the expression
    let func = compiler::compile_expr(&expr).expect("Failed to compile the expression");

    // Take inputs from command line
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

    // Call the function
    let result = func.call(inputs[0], inputs[1], inputs[2], inputs[3]);
    println!("Result: {}", result);
}
