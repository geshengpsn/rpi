use std::time::{Duration, Instant};

use opencv::{
    core::Size,
    videoio::{VideoWriter, VideoWriterTrait},
};

use rpi::usb_camera::spawn_usb_camera;

fn main() {
    let (tx, rx) = crossbeam::channel::unbounded();
    spawn_usb_camera(tx, 0, 1280, 720, 30);
    let fourcc = VideoWriter::fourcc('m', 'p', '4', 'v').unwrap();
    let mut vw = VideoWriter::new("test.mp4", fourcc, 30., Size::new(1280, 720), true).unwrap();

    println!("start");
    let start = Instant::now();
    while let Ok((img, _)) = rx.recv() {
        vw.write(&img).unwrap();
        if start.elapsed() >= Duration::from_secs(5) {
            break;
        }
    }
    println!("end");
    // vw.release()
}
