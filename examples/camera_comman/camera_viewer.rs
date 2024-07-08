use opencv::highgui;
use rpi::usb_camera::spawn_usb_camera;

fn main() {
    let (tx, rx) = crossbeam::channel::unbounded();
    spawn_usb_camera(tx, 0, 1280, 720, 30);
    while let Ok((img, _)) = rx.recv() {
        highgui::imshow("camera viewer", &img).unwrap();
        let code = highgui::wait_key(1).unwrap();
        if let Some('q') = char::from_u32(code as u32) {
            return;
        }
    }
}