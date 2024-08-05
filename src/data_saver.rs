use std::{io::BufWriter, time::Duration};

use opencv::{
    core::{Mat, Size},
    videoio::{VideoWriter, VideoWriterTrait, VideoWriterTraitConst},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    StartRecord(String),
    EndRecord,
}

pub trait FrameData: serde::Serialize + Send + Clone + 'static {
    fn time_stamp(&self) -> Duration;
}

pub trait DataFile {
    type Param: Copy;
    type Data;
    fn new() -> Self;
    fn is_started(&self) -> bool;
    fn start_new(&mut self, path: &str, param: Self::Param);
    fn record(&mut self, data: Self::Data);
    fn end(&mut self);
}

#[derive(Clone, Copy)]
pub struct VideoDesc {
    width: u32,
    height: u32,
    fps: u32,
}

impl DataFile for VideoWriter {
    type Param = VideoDesc;

    type Data = (Mat, Duration);

    fn new() -> Self {
        VideoWriter::default().unwrap()
    }

    fn is_started(&self) -> bool {
        self.is_opened().unwrap()
    }

    fn start_new(&mut self, path: &str, param: Self::Param) {
        let fourcc = VideoWriter::fourcc('m', 'p', '4', 'v').unwrap();
        self.open(
            path,
            fourcc,
            param.fps as f64,
            Size::new(param.width as i32, param.height as i32),
            true,
        )
        .unwrap();
    }

    fn record(&mut self, data: Self::Data) {
        self.write(&data.0).unwrap();
    }

    fn end(&mut self) {
        self.release().unwrap();
    }
}

pub struct CSVFile<FD> {
    csv_wtr: Option<csv::Writer<BufWriter<std::fs::File>>>,
    _p: std::marker::PhantomData<FD>,
}

impl<FD: FrameData> DataFile for CSVFile<FD> {
    type Param = ();

    type Data = FD;

    fn new() -> Self {
        Self {
            csv_wtr: None,
            _p: std::marker::PhantomData,
        }
    }

    fn is_started(&self) -> bool {
        self.csv_wtr.is_some()
    }

    fn start_new(&mut self, path: &str, _param: Self::Param) {
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
        self.csv_wtr = Some(wtr);
    }

    fn record(&mut self, data: Self::Data) {
        if let Some(wtr) = self.csv_wtr.as_mut() {
            wtr.serialize(data).expect("serialize");
        }
    }

    fn end(&mut self) {
        if let Some(wtr) = self.csv_wtr.as_mut() {
            wtr.flush().expect("flush csv");
        }
        self.csv_wtr = None;
    }
}

// pub async fn video_save(
//     data_rx: mpsc::Receiver<(Mat, Duration)>,
//     cmd_rx: mpsc::Receiver<Command>,
//     width: u32,
//     height: u32,
//     fps: u32,
// ) {
//     data_save::<VideoWriter>(data_rx, cmd_rx, VideoDesc { width, height, fps }).await;
// }

// pub async fn frame_data_save<FD: FrameData>(
//     data_rx: mpsc::Receiver<FD>,
//     signal_rx: mpsc::Receiver<Command>,
// ) {
//     data_save::<CSVFile<FD>>(data_rx, signal_rx, ()).await;
// }

// pub async fn data_save<DF: DataFile>(
//     mut data_rx: mpsc::Receiver<DF::Data>,
//     mut cmd_rx: mpsc::Receiver<Command>,
//     param: DF::Param,
// ) {
//     let mut f = DF::new();
//     loop {
//         tokio::select! {
//             Some(data) = async {
//                 if f.is_started() {
//                     // data_rx.try_recv()
//                     data_rx.recv().await
//                     // video_wtr.write(&img).unwrap();
//                 } else {
//                     None
//                 }
//             } => {
//                 // debug!("write");
//                 f.record(data);
//             },

//             Some(cmd) = cmd_rx.recv() => {
//                 match cmd {
//                     Command::StartRecord(path) => {
//                         // info!("recv StartRecord command");
//                         f.start_new(path.as_str(), param);
//                     }
//                     Command::EndRecord => {
//                         // info!("recv EndRecord command");
//                         if f.is_started() {
//                             f.end();
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

// pub async fn video_saver(
//     mut data_rx: mpsc::Receiver<(Mat, Duration)>,
//     mut cmd_rx: mpsc::Receiver<Command>,
//     width: u32,
//     height: u32,
//     fps: u32,
// ) {
//     let fourcc = VideoWriter::fourcc('m', 'p', '4', 'v').unwrap();
//     let mut video_wtr: VideoWriter = VideoWriter::default().unwrap();
//     loop {
//         tokio::select! {
//             Some((img, _)) = async{
//                 if video_wtr.is_opened().unwrap() {
//                     data_rx.recv().await
//                     // video_wtr.write(&img).unwrap();
//                 } else {
//                     None
//                 }
//             } => {
//                 if video_wtr.is_opened().unwrap() {
//                     // debug!("write");
//                     video_wtr.write(&img).unwrap();
//                 }
//             },

//             Some(cmd) = cmd_rx.recv() => {
//                 match cmd {
//                     Command::StartRecord(path) => {
//                         video_wtr
//                             .open(
//                                 &path,
//                                 fourcc,
//                                 fps as f64,
//                                 Size::new(width as i32, height as i32),
//                                 true,
//                             )
//                             .unwrap();
//                     }
//                     Command::EndRecord => {
//                         if video_wtr.is_opened().unwrap() {
//                             video_wtr.release().unwrap();
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }
