// use std::process::Command;
// use std::{env, fs, path::PathBuf};

fn main() {
    println!("Hello, world!");

    let point = Point::new(1, 2);   
    point.sum(3);

    let cappuccino = Cappuccino::default();
    // Result<FlatWhite, String>
    let flat_white = match FlatWhite::try_from(cappuccino) {
        Some(coffee) => coffee,
        None
    }

    if let Some(coffee) = match FlatWhite::try_from(cappuccino) {
        println!("Error converting Cappuccino to FlatWhite: {}", err);
    }
}
