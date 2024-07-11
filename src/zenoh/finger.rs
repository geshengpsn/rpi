use opencv::{
    core::{Mat, CV_8UC3},
    imgproc::{cvt_color_def, COLOR_RGB2BGR},
};
use rpi::{ArucoFinder, Camera, FingerForceResult, SoftFinger, ArucoFinderSetting, ArucoIntrinsic, CameraIntrinsic, CameraDistortion};
use zenoh::prelude::sync::*;

fn main() {
    let width = 1280;
    let height = 720;
    let fps = 60;
    let cx = 655.3664;
    let cy = 367.5246;
    let fx = 971.2252;
    let fy = 970.7470;
    let k1 = 0.0097;
    let k2 = -0.00745;
    let k3 = 0.00;
    let p1 = 0.00;
    let p2 = 0.00;

    let mut camera = Camera::new(2, width, height, fps).unwrap();

    let mut bgr_mat = Mat::default();

    let soft_finger = SoftFinger::new_pt("./model.pth");
    let mut arucos = vec![];
    let session = zenoh::open(config::default()).res().unwrap();
    let force_pub = session
        .declare_publisher("left_finger/force")
        .res()
        .unwrap();
    // let cmd_subscriber = session.declare_subscriber("cmd/record").res().unwrap();
    let setting = ArucoFinderSetting {
        aruco_intrinsic: ArucoIntrinsic::new_with_marker_length(0.05),
        camera_intrinsic: CameraIntrinsic { cx, cy, fx, fy },
        camera_distortion: CameraDistortion::from_5_params(k1, k2, p1, p2, k3),
    };
    let aruco_finder = ArucoFinder::new(setting);
    // let (tx, rx) = channel();
    // spawn(move || {
    //     let mut arucos = vec![];
    //     let session = zenoh::open(config::default()).res().unwrap();
    //     let force_pub = session.declare_publisher("left_finger/force").res().unwrap();
    //     let cmd_subscriber = session.declare_subscriber("cmd/record").res().unwrap();
    //     let setting = ArucoFinderSetting {
    //         aruco_intrinsic: ArucoIntrinsic::new_with_marker_length(0.05),
    //         camera_intrinsic: CameraIntrinsic { cx, cy, fx, fy },
    //         camera_distortion: CameraDistortion::from_5_params(k1, k2, p1, p2, k3),
    //     };
    //     let aruco_finder = ArucoFinder::new(setting);
    //     while let Ok((bgr_mat, time_stamp)) = rx.recv() {

    //     }
    // });
    loop {
        // let start = Instant::now();
        let (rgb_raw_data, time_stamp) = camera.capture().unwrap();
        let rbg_img = unsafe {
            Mat::new_rows_cols_with_data_unsafe_def(
                height as i32,
                width as i32,
                CV_8UC3,
                rgb_raw_data.as_ptr() as *mut _,
            )
        }
        .unwrap();
        cvt_color_def(&rbg_img, &mut bgr_mat, COLOR_RGB2BGR).unwrap();
        aruco_finder
            .find(&bgr_mat, time_stamp, &mut arucos)
            .unwrap();
        if let Some(aruco) = arucos.first() {
            let f = soft_finger.predict_force(aruco);
            force_pub
                .put(serde_json::to_value(FingerForceResult::Force(f)).unwrap())
                .res()
                .unwrap();
        } else {
            force_pub
                .put(serde_json::to_value(FingerForceResult::NoAruco).unwrap())
                .res()
                .unwrap();
        }
    }
}
