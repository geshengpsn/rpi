use std::{thread::{sleep, spawn}, time::Duration};

use crossbeam::channel::unbounded;

fn main() {
    let (tx, rx) = unbounded();
    let rx1 = rx.clone();
    spawn(move ||{
        while let Ok(data) = rx1.recv() {
            println!("rx1: {data}")
        }  
    });
    spawn(move ||{
        while let Ok(data) = rx.recv() {
            println!("rx: {data}")
        }  
    });

    let mut count = 0;
    loop {
        tx.send(format!("{}", count)).unwrap();
        count += 1;
        sleep(Duration::from_secs(1));
    }
}