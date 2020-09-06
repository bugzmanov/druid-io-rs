use std::sync::Arc;
use std::sync::atomic::Ordering::{self, SeqCst};
use std::sync::atomic::AtomicUsize;

pub trait BrokersPool {
    fn broker(&self) -> &String;
}

pub struct StaticPool {
    brokers: Vec<String>,
    selection: SelectionStategy,
}

impl StaticPool {
    pub fn new(brokers: Vec<String>, stategy: SelectionStategy) -> Self {
        StaticPool {
            brokers: brokers,
            selection: stategy,
        }
    }
}
impl BrokersPool for StaticPool {
    fn broker(&self) -> &String {
        self.selection.select(&self.brokers)
    }
}
pub enum SelectionStategy {
    Constant,
    // Random,
    RoundRobin(Arc<AtomicUsize>),
}


fn get_and_increment(index: &AtomicUsize, max_val: usize) -> usize {
    loop {
        let val = index.load(SeqCst);
        let new_val = if val >= max_val { 1 } else { val + 1 };
        if index.compare_and_swap(val, new_val, SeqCst) == val {
            return if val >= max_val { 0 as usize } else { val };
        }
    }
}
impl SelectionStategy {
    pub fn select<'a, T>(&self, list: &'a Vec<T>) -> &'a T {
        if let SelectionStategy::RoundRobin(ref index) = *self {
            let idx = get_and_increment(index, list.len());
            list.get(idx).unwrap()
        } else {
            list.get(0).unwrap()
        }
    }

    pub fn round_robin() -> SelectionStategy {
        SelectionStategy::RoundRobin(Arc::new(AtomicUsize::new(0)))
    }
    pub fn constant() -> SelectionStategy {
        SelectionStategy::Constant
    }

    pub fn default_for<T>(list: &Vec<T>) -> SelectionStategy {
        if list.len() == 1 {
            SelectionStategy::constant()
        } else {
            SelectionStategy::round_robin()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_round_robin_selection() {
       let nodes = vec!["localhost:8081".to_string(), "localhost:80".to_string(), "example.com:8080".to_string()];

       let strategy = SelectionStategy::default_for(&nodes);
       assert_eq!(strategy.select(&nodes), "localhost:8081");
       assert_eq!(strategy.select(&nodes), "localhost:80");
       assert_eq!(strategy.select(&nodes), "example.com:8080");
       assert_eq!(strategy.select(&nodes), "localhost:8081");
       assert_eq!(strategy.select(&nodes), "localhost:80");
       assert_eq!(strategy.select(&nodes), "example.com:8080");
    } 
    #[test]
    fn test_constaint() {
       let nodes = vec!["localhost:8081".to_string(), "localhost:80".to_string(), "example.com:8080".to_string()];

       let strategy = SelectionStategy::constant();
       assert_eq!(strategy.select(&nodes), "localhost:8081");
       assert_eq!(strategy.select(&nodes), "localhost:8081");
       assert_eq!(strategy.select(&nodes), "localhost:8081");
    } 
}