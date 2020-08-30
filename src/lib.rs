#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate serde_json;

pub mod client;
pub mod query;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
