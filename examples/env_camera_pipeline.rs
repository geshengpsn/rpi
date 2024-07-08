use crossbeam::channel::unbounded;
use rpi::{
    aruco_finder::{
        spawn_aruco_finder, ArucoFinderSetting, ArucoIntrinsic, CameraDistortion, CameraIntrinsic,
    },
    data_saver::spawn_video_saver,
    data_streamer::spawn_video_streaming,
    rx_hub::rx_hub3,
    usb_camera::spawn_usb_camera,
};

fn main() {
    let (tx, rx) = unbounded();
    let (rx1, rx2, rx3) = rx_hub3(rx);
    // let (aruco_tx, aruco_rx) = unbounded();
    let (sig_tx, sig_rx) = unbounded();

    let j = spawn_usb_camera(tx, 0, 1280, 720, 60);
    spawn_video_saver(rx1, sig_rx, 1280, 720, 60);
    spawn_video_streaming(rx2, "0.0.0.0:8080", "/video".into());
    // let cx = 655.3664;
    // let cy = 367.5246;
    // let fx = 971.2252;
    // let fy = 970.7470;
    // let k1 = 0.0097;
    // let k2 = -0.00745;
    // let k3 = 0.00;
    // let p1 = 0.00;
    // let p2 = 0.00;
    // let setting = ArucoFinderSetting {
    //     aruco_intrinsic: ArucoIntrinsic::new_with_marker_length(0.05),
    //     camera_intrinsic: CameraIntrinsic { cx, cy, fx, fy },
    //     camera_distortion: CameraDistortion::from_5_params(k1, k2, p1, p2, k3),
    // };
    // spawn_aruco_finder(rx3, aruco_tx, setting);
    // while let Ok(arucos) = aruco_rx.recv() {
    //     if !arucos.is_empty() {
    //         println!("{arucos:?}");
    //     }
    // }
    j.join().unwrap();
}
