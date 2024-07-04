use rpi::Result;

fn main() -> Result<()> {
    // init();
    multi_process()
}

fn multi_process() -> Result<()> {
    // // // image mat tx rx
    // let (tx, rx) = unbounded();

    // // // arucos data tx rx
    // let (aruco_tx, aruco_rx) = unbounded();

    // let j1 = aruco_camera_capture(tx, 0, 1280, 720, 30);

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
    // let j2 = find_arucos_process(rx, aruco_tx, setting);
    // // j1.join().unwrap()?;
    // // j2.join().unwrap()?;

    Ok(())
}

// fn aruco_camera_capture(
//     tx: Sender<(Mat, TimeSpec)>,
//     aruco_camera_index: usize,
//     width: u32,
//     height: u32,
//     fps: u32,
// ) -> Result<()> {
//     let mut cam = Camera::new(aruco_camera_index, width, height, fps)?;
//     loop {
//         let (raw_img, md) = cam.capture()?;
//         let img = unsafe {
//             Mat::new_rows_cols_with_data_unsafe_def(
//                 height as i32,
//                 width as i32,
//                 CV_8UC3,
//                 raw_img.as_ptr() as *mut _,
//             )
//         }?;
//         let mut res_img = Mat::default();
//         cvt_color_def(&img, &mut res_img, COLOR_RGB2BGR)?;
//         tx.send((
//             res_img,
//             TimeSpec::new(md.timestamp.sec, md.timestamp.usec * 1000),
//         ))?;
//     }
// }

// fn find_arucos_process(
//     rx: Receiver<(Mat, TimeSpec)>,
//     aruco_tx: Sender<Vec<Aruco>>,
//     setting: ArucoFinderSetting,
// ) -> Result<()> {
//     let aruco_finder = ArucoFinder::new(setting);
//     let mut arucos = vec![];
//     loop {
//         let (img, time_stamp) = rx.recv()?;
//         aruco_finder.find(img, time_stamp, &mut arucos)?;
//     }
// }
