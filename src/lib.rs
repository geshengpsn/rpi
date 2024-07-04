mod net;

mod errors;
pub use errors::Result;
pub mod usb_camera;
pub mod aruco_finder;

// mod video_recorder;

pub mod soft_finger;

mod data_saver;
mod data_streamer;
mod imu;
mod screen;