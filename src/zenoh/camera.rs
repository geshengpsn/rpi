use opencv::{
    core::{Mat, Size, Vector, VectorToVec, CV_8UC3},
    imgcodecs::imencode_def,
    imgproc::{cvt_color_def, COLOR_RGB2BGR},
    videoio::{VideoWriter, VideoWriterTrait, VideoWriterTraitConst},
};
use rpi::{Camera, RecordCommand};
use zenoh::prelude::sync::*;

fn main() {
    let width = 1280;
    let height = 720;
    let fps = 60;

    let session = zenoh::open(config::default()).res().unwrap();
    let compress_pub = session
        .declare_publisher("camera")
        .res()
        .unwrap();
    let cmd_subscriber = session.declare_subscriber("cmd/record").res().unwrap();

    let mut camera = Camera::new(0, width, height, fps).unwrap();
    let mut bgr_mat = Mat::default();
    let mut v = Vector::<u8>::new();
    let mut vw = VideoWriter::default().unwrap();

    let mut count = 0;

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
        if let Ok(cmd) = cmd_subscriber.try_recv() {
            let cmd_json = cmd.value.try_into().unwrap();
            let cmd = serde_json::from_value::<RecordCommand>(cmd_json).unwrap();
            match cmd {
                RecordCommand::Start { env_camera, .. } => {
                    if !vw.is_opened().unwrap() {
                        let fourcc = VideoWriter::fourcc('m', 'p', '4', 'v').unwrap();
                        vw.open(
                            &env_camera,
                            fourcc,
                            30.,
                            Size::new(width as i32, height as i32),
                            true,
                        )
                        .unwrap();
                        count = 0;
                    }
                }
                RecordCommand::End => {
                    if vw.is_opened().unwrap() {
                        vw.release().unwrap();
                        println!("frame count{count}");
                    }
                }
            }
        }
        if vw.is_opened().unwrap() {
            vw.write(&bgr_mat).unwrap();
            count += 1;
        }
    }
}
