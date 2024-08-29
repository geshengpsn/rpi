use clap::Parser;
use opencv::{
    core::{Mat, Vector, VectorToVec, CV_8UC3},
    imgcodecs::imencode_def,
    imgproc::{cvt_color_def, COLOR_RGB2BGR},
};
use rpi::Camera;
use zenoh::prelude::sync::*;

#[derive(Parser, Debug)]
struct Args {
    usb: u32,
    // #[arg(short, long, default_value_t = 1920)]
    // width: u32,

    // #[arg(short, long, default_value_t = 1080)]
    // height: u32,

    // #[arg(short, long, default_value_t = 30)]
    // fps: u32,
}

fn main() {
    let args = Args::parse();
    let width = 1280;
    let height = 720;
    let fps = 60;

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

    let session = zenoh::open(config::default()).res().unwrap();
    let compress_pub = session.declare_publisher("camera").res().unwrap();

    let mut bgr_mat = Mat::default();
    let mut v = Vector::<u8>::new();
    loop {
        let (rgb_raw_data, _time) = camera.capture().unwrap();
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
        compress_pub.put(v.to_vec()).res().unwrap();
    }
}
