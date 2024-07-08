use rpi::{
    aruco_finder::{
        spawn_aruco_finder, ArucoFinderSetting, ArucoIntrinsic, CameraDistortion, CameraIntrinsic,
    },
    usb_camera::spawn_usb_camera,
    soft_finger::SoftFinger
};

fn main() {
    let (mat_tx, mat_rx) = crossbeam::channel::unbounded();
    spawn_usb_camera(mat_tx, 0, 1280, 720, 30);
    let cx = 655.3664;
    let cy = 367.5246;
    let fx = 971.2252;
    let fy = 970.7470;
    let k1 = 0.0097;
    let k2 = -0.00745;
    let k3 = 0.00;
    let p1 = 0.00;
    let p2 = 0.00;
    let setting = ArucoFinderSetting {
        aruco_intrinsic: ArucoIntrinsic::new_with_marker_length(0.05),
        camera_intrinsic: CameraIntrinsic { cx, cy, fx, fy },
        camera_distortion: CameraDistortion::from_5_params(k1, k2, p1, p2, k3),
    };
    let (arucos_tx, arucos_rx) = crossbeam::channel::unbounded();
    spawn_aruco_finder(mat_rx, arucos_tx, setting);
    let sf = SoftFinger::new_pt("./model.pth");
    while let Ok(arucos) = arucos_rx.recv() {
        if !arucos.is_empty() {
            let f = sf.predict_force(arucos[0]);
            println!("{f:.2?}");
        }
    }
}
