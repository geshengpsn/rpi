use crossbeam::channel::unbounded;
use rpi::imu::spawn_imu;

fn main() {
    let (tx, rx) = unbounded();
    spawn_imu(tx);
    while let Ok(data) = rx.recv() {
        println!("{data:?}");
    }    
}
