use std::{io::BufWriter, thread::spawn, time::Duration};

use crossbeam::channel::{Receiver, Sender};
use opencv::{
    core::{Mat, Size},
    videoio::{VideoWriter, VideoWriterTrait, VideoWriterTraitConst},
};

pub enum Signal {
    Start(String),
    End,
}

pub trait FrameData: serde::Serialize + Send + Clone + 'static {
    fn time_stamp(&self) -> Duration;
}

pub fn spawn_video_saver(
    data_rx: Receiver<(Mat, Duration)>,
    data_tx: Option<Sender<(Mat, Duration)>>,
    signal_rx: Receiver<Signal>,
    width: u32,
    height: u32,
    fps: u32,
) {
    spawn(move || {
        let fourcc = VideoWriter::fourcc('m', 'p', '4', 'v').unwrap();
        let mut video_wtr: VideoWriter = VideoWriter::default().unwrap();
        while let Ok(data) = data_rx.recv() {
            if let Some(tx) = data_tx.as_ref() {
                tx.send(data.clone()).unwrap();
            }
            if video_wtr.is_opened().unwrap() {
                video_wtr.write(&data.0).unwrap();
            }
            match signal_rx.try_recv() {
                Ok(s) => match s {
                    Signal::Start(path) => {
                        video_wtr
                            .open(
                                &path,
                                fourcc,
                                fps as f64,
                                Size::new(width as i32, height as i32),
                                true,
                            )
                            .unwrap();
                    }
                    Signal::End => {
                        if video_wtr.is_opened().unwrap() {
                            video_wtr.release().unwrap();
                        }
                    }
                },
                Err(crossbeam::channel::TryRecvError::Disconnected) => {
                    // abort
                    panic!("spawn_video_saver signal tx closed")
                }
                _ => {}
            }
        }
    });
}

pub fn spawn_data_saver<FD: FrameData>(
    data_rx: Receiver<FD>,
    data_tx: Option<Sender<FD>>,
    signal_rx: Receiver<Signal>,
) {
    spawn(move || {
        let mut csv_wtr: Option<csv::Writer<BufWriter<std::fs::File>>> = None;
        while let Ok(data) = data_rx.recv() {
            if let Some(tx) = data_tx.as_ref() {
                tx.send(data.clone()).unwrap();
            }
            if let Some(wtr) = csv_wtr.as_mut() {
                wtr.serialize(data).expect("serialize");
            }
            match signal_rx.try_recv() {
                Ok(s) => match s {
                    Signal::Start(path) => {
                        let f = std::fs::OpenOptions::new()
                            .write(true)
                            .create(true)
                            .truncate(true)
                            .open(path)
                            .expect("file");
                        let buf_wtr = BufWriter::new(f);
                        let wtr = csv::WriterBuilder::new()
                            .has_headers(false)
                            .from_writer(buf_wtr);
                        csv_wtr = Some(wtr);
                    }
                    Signal::End => {
                        if let Some(wtr) = csv_wtr.as_mut() {
                            wtr.flush().expect("flush csv");
                        }
                        csv_wtr = None
                    }
                },
                Err(crossbeam::channel::TryRecvError::Disconnected) => {
                    // abort
                    panic!("spawn_data_saver signal tx closed")
                }
                _ => {}
            }
        }
    });
}
