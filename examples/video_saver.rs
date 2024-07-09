use rpi::{
    data_saver::{video_saver, Command},
    usb_camera::usbcamera,
};
use std::{io::stdin, time::Duration};
use tokio::sync::{broadcast, mpsc};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();
    let (tx, rx) = mpsc::channel(8);
    let (cmd_tx, cmd_rx) = mpsc::channel(8);
    tokio::join!(
        usbcamera(tx, 0, 1280, 720, 60),
        video_saver(rx, cmd_rx, 1280, 720, 60),
        handle_cmd(cmd_tx),
    );
}

async fn handle_cmd(cmd_tx: mpsc::Sender<Command>) {
    cmd_tx
        .send(Command::StartRecord("test.mp4".into()))
        .await
        .unwrap();
    println!("start");
    tokio::time::sleep(Duration::from_secs(3)).await;
    cmd_tx.send(Command::EndRecord).await.unwrap();
    println!("end");
}
