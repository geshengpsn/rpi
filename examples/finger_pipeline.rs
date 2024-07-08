fn main() {
//     let (tx, rx) = unbounded();
//     let (tx1, rx1) = unbounded();
//     let (sig_tx, sig_rx) = unbounded();
//     spawn_usb_camera(tx, 0, 1280, 720, 30);
//     spawn_video_saver(rx, Some(tx1), sig_rx, 1280, 720, 30);
//     spawn_video_streaming(rx1, None, "0.0.0.0:8080", "/video".into());
//     loop {
//         let mut input = String::new();
//         stdin().read_line(&mut input).expect("input");
//         match input.as_str() {
//             "start\n" => {
//                 sig_tx.send(Signal::Start("test.mp4".into())).unwrap();
//             }
//             "stop\n" => {
//                 sig_tx.send(Signal::End).unwrap();
//             }
//             input => {
//                 println!("bad input:{input}")
//             }
//         }
//     }
}