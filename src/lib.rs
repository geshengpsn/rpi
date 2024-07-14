mod errors;
use errors::Result;
use serde::{Serialize, Deserialize};

mod usb_camera;
pub use usb_camera::Camera;

mod aruco_finder;
pub use aruco_finder::{ArucoFinder, ArucoFinderSetting, Aruco, ArucoIntrinsic, CameraIntrinsic, CameraDistortion};

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
    Start{
        imu: String,
        env_camera: String,
        left_finger: String,
        right_finger: String,
    },
    End
}