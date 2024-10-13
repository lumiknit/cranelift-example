use rand::Rng;

/// Print a value and return without modifying it
pub fn print(v: i32) -> i32 {
    println!("{}", v);
    v
}

/// Pick random integer between 0 and v
pub fn rand(v: i32) -> i32 {
    let result = rand::thread_rng().gen_range(0..v);
    println!("Rand({}) = {}", v, result);
    result
}
