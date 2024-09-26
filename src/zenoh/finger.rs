use clap::Parser;
use opencv::{
    core::{Mat, Vector, VectorToVec, CV_8UC3},
    imgcodecs::imencode_def,
    imgproc::{cvt_color_def, COLOR_RGB2BGR},
};
use rpi::{
    ArucoFinder, ArucoFinderSetting, ArucoIntrinsic, Camera, CameraDistortion,
    CameraIntrinsic, FingerForceData, SoftFinger,
};
use zenoh::prelude::sync::*;

#[derive(Parser, Debug)]
struct Args {
    // #[arg(default_value_t = left)]
    direct: String,
    // #[arg(short, long)]
    path: String,
    // #[arg(short, long)]
    usb: u32,

    fps: u32,
}
fn main() {
    let args = Args::parse();
    let width: u32 = 640;
    let height = 480;
    let fps = args.fps;
    let (a, b) = match args.usb {
        0 => (1, 1),
        1 => (0, 2),
        2 => (0, 1),
        3 => (1, 2),
        _ => {
            panic!("bad input:{}, only support 0,1,2,3", args.usb)
        }
    };
    let path = format!("/dev/v4l/by-path/platform-xhci-hcd.{a}-usb-0:{b}:1.0-video-index0");
    let mut camera = Camera::new_with_path(&path, width, height, fps).unwrap();

    let mut bgr_mat = Mat::default();

    let soft_finger = SoftFinger::new_pt(&args.path);
    let mut arucos = vec![];
    let session = zenoh::open(config::default()).res().unwrap();
    let is_right = match args.direct.as_str() {
        "right" => true,
        "left" => false,
        _ => {
            panic!("left or right")
        }
    };
    let base_key = if is_right {
        "finger/right"
    } else {
        "finger/left"
    };
    let force_pub = session
        .declare_publisher(format!("{base_key}/force"))
        .res()
        .unwrap();
    let image_pub = session
        .declare_publisher(format!("{base_key}/image"))
        .res()
        .unwrap();
    // let cmd_subscriber = session.declare_subscriber("cmd/record").res().unwrap();

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
    let aruco_finder = ArucoFinder::new(setting);
    // let mut csv_file = CSVFile::<FingerForceData>::new();
    let mut v = Vector::<u8>::new();
    loop {
        let (rgb_raw_data, time_stamp) = match camera.capture() {
            Ok((rgb_raw_data, time_stamp)) => (rgb_raw_data, time_stamp),
            Err(_e) => {
                // println!("{e:?}");
                continue;
            }
        };

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
        imencode_def(".jpg", &bgr_mat, &mut v).unwrap();
        image_pub.put(v.to_vec()).res().unwrap();
        aruco_finder
            .find(&rbg_img, time_stamp, &mut arucos)
            .unwrap();
        let force_data = FingerForceData {
            force: arucos.first().map(|aruco| soft_finger.predict_force(aruco)),
            time_stamp,
        };
        // println!("{force_data:?}");
        force_pub
            .put(serde_json::to_value(force_data).unwrap())
            .res()
            .unwrap();
    }
}
