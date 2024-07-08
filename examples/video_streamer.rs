use crossbeam::channel::unbounded;
use rpi::{data_streamer::spawn_video_streaming, usb_camera::spawn_usb_camera};

fn main() {
    let (tx, rx) = unbounded();
    let j1 = spawn_usb_camera(tx, 0, 1280, 720, 30);
    let j2 = spawn_video_streaming(rx, None, "0.0.0.0:8080", "/video".into());

    j1.join().unwrap();
    j2.join().unwrap();
}