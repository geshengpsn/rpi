use clap::Parser;
use opencv::{
    core::{Mat, CV_8UC3},
    imgproc::{cvt_color_def, COLOR_RGB2BGR},
};
use rpi::{
    ArucoFinder, ArucoFinderSetting, ArucoIntrinsic, CSVFile, Camera, CameraDistortion,
    CameraIntrinsic, DataFile, FingerForceData, RecordCommand, SoftFinger,
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
}
fn main() {
    let args = Args::parse();
    let width: u32 = 640;
    let height = 360;
    let fps = 330;
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
    let key = if is_right {
        "finger/right/force"
    } else {
        "finger/left/force"
    };
    let force_pub = session.declare_publisher(key).res().unwrap();
    let cmd_subscriber = session.declare_subscriber("cmd/record").res().unwrap();

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
    let mut csv_file = CSVFile::<FingerForceData>::new();
    loop {
        let (rgb_raw_data, time_stamp) = match camera.capture() {
            Ok((rgb_raw_data, time_stamp)) => (rgb_raw_data, time_stamp),
            Err(e) => {
                println!("{e:?}");
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
        aruco_finder
            .find(&bgr_mat, time_stamp, &mut arucos)
            .unwrap();
        let force_data = FingerForceData {
            force: arucos.first().map(|aruco| soft_finger.predict_force(aruco)),
            time_stamp,
        };
        force_pub
            .put(serde_json::to_value(force_data).unwrap())
            .res()
            .unwrap();
        if let Ok(cmd) = cmd_subscriber.try_recv() {
            let cmd_json = cmd.value.try_into().unwrap();
            let cmd = serde_json::from_value::<RecordCommand>(cmd_json).unwrap();
            match cmd {
                RecordCommand::Start {
                    left_finger,
                    right_finger,
                    ..
                } => {
                    if !csv_file.is_started() {
                        if is_right {
                            csv_file.start_new(&right_finger, ())
                        } else {
                            csv_file.start_new(&left_finger, ())
                        };
                    }
                }
                RecordCommand::End => {
                    if csv_file.is_started() {
                        csv_file.end()
                    }
                }
            }
        }
    }
}
