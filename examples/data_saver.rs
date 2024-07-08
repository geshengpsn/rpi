use std::io::stdin;

use crossbeam::channel::unbounded;
use rpi::{
    data_saver::{spawn_data_saver, Signal},
    imu::spawn_imu,
};

fn main() {
    let (tx, rx) = unbounded();
    let (sig_tx, sig_rx) = unbounded();
    spawn_imu(tx);
    spawn_data_saver(rx, sig_rx);
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("input");
        match input.as_str() {
            "start\n" => {
                sig_tx.send(Signal::Start("test.imu.csv".into())).unwrap();
            }
            "stop\n" => {
                sig_tx.send(Signal::End).unwrap();
            }
            input => {
                println!("bad input:{input}")
            }
        }
    }
}
