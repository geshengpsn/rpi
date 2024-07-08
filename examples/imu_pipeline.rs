use std::io::stdin;

use crossbeam::channel::unbounded;

use rpi::{
    data_saver::{spawn_data_saver, Signal}, data_streamer::spawn_data_streaming, imu::{spawn_imu, IMU}
};

fn main() {
    let (tx, rx) = unbounded();
    let (tx1, rx1) = unbounded();
    let (sig_tx, sig_rx) = unbounded();
    let imu = IMU::new("/dev/i2c-1");
    let _j1 = spawn_imu(imu, tx);
    let _j2 = spawn_data_streaming(rx, Some(tx1), "0.0.0.0:8080", "/imu".into());
    let _j3 = spawn_data_saver(rx1, None, sig_rx);
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