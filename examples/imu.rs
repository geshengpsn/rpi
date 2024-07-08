use crossbeam::channel::unbounded;
use rpi::imu::{spawn_imu, IMU};

fn main() {
    let (tx, rx) = unbounded();
    let imu = IMU::new("/dev/i2c-1");
    spawn_imu(imu, tx);
    while let Ok(data) = rx.recv() {
        println!("{data:?}");
    }
}
