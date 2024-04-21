pub mod oltp_config;
pub mod tcp;
pub fn add(left: usize, right: usize) -> usize {
    left + right
}
pub use oltp_config::OLTP_METER;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
