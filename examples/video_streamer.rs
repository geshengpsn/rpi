use rpi::{
    data_streamer::video_stream,
    usb_camera::{mat_from_ptr, Camera},
};
use tokio::{join, sync::mpsc};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let (tx, rx) = mpsc::channel(1);
    join!(
        async {
            let mut cam = Camera::new(0, 1280, 720, 60).expect("camera bad parameters");
            loop {
                let (raw_img, ts) = cam.capture().unwrap();
                let img = mat_from_ptr(raw_img.as_ptr(), 1280, 720).unwrap();
                tx.send((img, ts)).await.unwrap();
                tracing::debug!("camera capture");
            }
        },
        video_stream(rx, "0.0.0.0:8080", "/video"),
    );
}
