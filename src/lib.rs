mod errors;
use errors::Result;
use serde::{Deserialize, Serialize};

mod usb_camera;
pub use usb_camera::Camera;

mod aruco_finder;
pub use aruco_finder::{
    Aruco, ArucoFinder, ArucoFinderSetting, ArucoIntrinsic, CameraDistortion, CameraIntrinsic,
};

mod soft_finger;
pub use soft_finger::{FingerForceData, Force, SoftFinger};

mod data_saver;
pub use data_saver::{CSVFile, Command, DataFile};

// mod data_streamer;
mod imu;
pub use imu::{IMUData, IMU};

// mod rx_hub;
mod ssd1306_screen;

#[derive(Debug, Serialize, Deserialize)]
pub enum RecordCommand {
    Start {
        imu: String,
        env_camera: String,
        left_finger: String,
        right_finger: String,
    },
    End,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AngleData {
    pub data: u16,
    pub time_stamp: std::time::Duration,
}