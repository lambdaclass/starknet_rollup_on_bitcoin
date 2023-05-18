use std::println;

use bitcoin::{Transaction};

fn main() {
    let src = include_str!("test.json");
    let tx: Transaction = serde_json::from_str(src).unwrap();
    println!("{:?}", tx);
}
