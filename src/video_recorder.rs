use std::{
    process::Command,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{spawn, JoinHandle},
};

pub struct VideoRecorder {
    is_recording: bool,
    backend: JoinHandle<()>,
    tx: Sender<VideoRecorderSignal>,
}

enum VideoRecorderSignal {
    Start(String),
    End,
}

impl VideoRecorder {
    pub fn new(framerate: u32, width: u32, height: u32, camera_index: usize) -> Self {
        let (tx, rx) = channel();
        let backend = Self::run_backend(framerate, width, height, camera_index, rx);
        Self {
            is_recording: false,
            backend,
            tx,
        }
    }

    pub fn start_record(&mut self, path: String) {
        // is recording
        if !self.is_recording {
            // send command
            if self.tx.send(VideoRecorderSignal::Start(path)).is_ok() {
                self.is_recording = true;
            }
        }
    }

    pub fn end_record(&mut self, path: &str) {
        if self.is_recording {
            // send command
            if self.tx.send(VideoRecorderSignal::End).is_ok() {
                self.is_recording = false;
            }
        }
    }

    fn run_backend(
        framerate: u32,
        width: u32,
        height: u32,
        camera_index: usize,
        rx: Receiver<VideoRecorderSignal>,
    ) -> JoinHandle<()> {
        spawn(move || {
            let mut running_command = None;
            while let Ok(signal) = rx.recv() {
                match signal {
                    VideoRecorderSignal::Start(path) => {
                        running_command = Option::Some(
                            video_record_commmand(framerate, width, height, camera_index, &path)
                                .spawn()
                                .unwrap(),
                        );
                    }
                    VideoRecorderSignal::End => {
                        running_command.as_mut().unwrap().kill().unwrap();
                        running_command = None;
                    }
                }
            }
        })
    }
}

fn video_record_commmand(
    framerate: u32,
    width: u32,
    height: u32,
    camera_index: usize,
    path: &str,
) -> Command {
    Command::new(format!("ffmpeg -f v4l2 -framerate {framerate} -video_size {width}x{height} -i /dev/video{camera_index} {path}"))
}
