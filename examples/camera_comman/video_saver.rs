use std::io::stdin;

use rpi::{
    data_saver::{spawn_video_saver, Signal},
    usb_camera::spawn_usb_camera,
};

fn main() {
    let (tx, rx) = crossbeam::channel::unbounded();
    let (sig_tx, sig_rx) = crossbeam::channel::unbounded();
    spawn_usb_camera(tx, 0, 1280, 720, 30);
    spawn_video_saver(rx, None, sig_rx, 1280, 720, 30);
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("input");
        match input.as_str() {
            "start\n" => {
                sig_tx.send(Signal::Start("test.mp4".into())).unwrap();
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
