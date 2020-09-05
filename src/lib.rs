extern crate serde_json;

pub mod client;
pub mod query;
pub mod serialization;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
