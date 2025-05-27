use monitor::{print_all, say_hello};



fn main() {
    let a = monitor::add(10, 15);
    let output = format!("Число: {}", a);
    println!("{}", output);
    say_hello!();
    print_all!(1, "текст", false);
}
