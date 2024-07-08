use crossbeam::channel::unbounded;
use rpi::{
    data_streamer::spawn_data_streaming,
    imu::{spawn_imu, IMU},
};

fn main() {
    tracing_subscriber::fmt::init();
    let (tx, rx) = unbounded();
    let imu = IMU::new("/dev/i2c-1");
    let j1 = spawn_imu(imu, tx);
    let j2 = spawn_data_streaming(rx, None, "0.0.0.0:8080", "/imu".into());

    j1.join().unwrap();
    j2.join().unwrap();
}
