pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[macro_export]
macro_rules! say_hello {
    () => {
        println!("Привет!");
    };
}

#[macro_export]
macro_rules! print_all {
    ($($item:expr),*) => {
        $(println!("Ты блять {}", $item);)*
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
