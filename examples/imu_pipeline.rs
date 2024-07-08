use std::io::stdin;

use crossbeam::channel::unbounded;

use rpi::{
    data_saver::{spawn_data_saver, Signal}, data_streamer::spawn_data_streaming, imu::{spawn_imu, IMU}, rx_hub::rx_hub2
};

fn main() {
    let imu = IMU::new("/dev/i2c-1");
    let (tx, rx) = unbounded();
    let _j1 = spawn_imu(imu, tx);
    let (rx1, rx2) = rx_hub2(rx);

    let _j2 = spawn_data_streaming(rx1, "0.0.0.0:8080", "/imu".into());

    let (sig_tx, sig_rx) = unbounded();
    let _j3 = spawn_data_saver(rx2, sig_rx);

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