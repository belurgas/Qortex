pub mod config;
pub mod logger;
pub mod macros;

// DELETE
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

// DELETE
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
