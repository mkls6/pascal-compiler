mod token;

fn main() {
    let my_integer = token::Integer::new(5);
    println!("My integer {}", my_integer);

    let my_real = token::Real::new(3.25);
    println!("My real {}", my_real);

    let another_real = token::Real::from_string(String::from("44.25")).unwrap();
    println!("Another real {}", another_real);
    println!("String representation: {}", another_real.as_string());

    let my_boolean = token::Boolean::from_string(String::from("true")).unwrap();
    println!("My boolean {}", my_boolean);

    let my_string = token::Str::from_string(String::from("MyString")).unwrap();
    println!("My string {}", my_string);

    println!("Actual value: {}", my_boolean.as_value())
}
