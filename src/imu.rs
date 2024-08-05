use crate::data_saver::FrameData;
use linux_embedded_hal::{Delay, I2cdev};
use mpu6050_dmp::{
    address::Address, calibration::CalibrationParameters, quaternion::Quaternion, sensor::Mpu6050,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;
// use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IMUData {
    quat: nalgebra::UnitQuaternion<f32>,
    time_stamp: std::time::Duration,
}

impl FrameData for IMUData {
    fn time_stamp(&self) -> std::time::Duration {
        self.time_stamp
    }
}

pub struct IMU {
    mpu6050: Mpu6050<I2cdev>,
    buf: [u8; 28],
    start: Instant,
    has_init: bool,
}

impl IMU {
    pub fn new(path: &str) -> Self {
        let i2c = I2cdev::new(path).unwrap();
        let mpu6050 = Mpu6050::new(i2c, Address::default()).unwrap();

        Self {
            mpu6050,
            buf: [0; 28],
            start: Instant::now(),
            has_init: false,
        }
    }

    pub fn calibrate(&mut self) {
        let cali_param = CalibrationParameters::new(
            mpu6050_dmp::accel::AccelFullScale::G2,
            mpu6050_dmp::gyro::GyroFullScale::Deg2000,
            mpu6050_dmp::calibration::ReferenceGravity::ZN,
        );
        self.mpu6050.calibrate(&mut Delay, &cali_param).unwrap();
    }

    pub fn init(&mut self) {
        self.mpu6050.initialize_dmp(&mut Delay).unwrap();
        self.has_init = true;
    }

    pub fn read(&mut self) -> IMUData {
        assert!(self.has_init);
        loop {
            let len = self.mpu6050.get_fifo_count().expect("get_fifo_count");
            if len >= 28 {
                let time_stamp = self.start.elapsed();
                self.mpu6050.read_fifo(&mut self.buf).expect("read_fifo");
                let quat = Quaternion::from_bytes(&self.buf[..16]).unwrap().normalize();
                return IMUData {
                    quat: nalgebra::UnitQuaternion::new_normalize(nalgebra::Quaternion::new(
                        quat.w, quat.x, quat.y, quat.z,
                    )),
                    time_stamp,
                };
            }
        }
    }
}

// #[derive(Debug)]
// pub enum IMUCommand {
//     Calibration,
// }

// #[derive(Debug)]
// pub enum IMUCommandReturn {
//     CalibrationFinished,
// }

// pub async fn run_imu(
//     mut imu: IMU,
//     tx: mpsc::Sender<IMUData>,
//     mut cmd_rx: mpsc::Receiver<(IMUCommand, oneshot::Sender<IMUCommandReturn>)>,
// ) {
//     assert!(imu.has_init);
//     loop {
//         tokio::select! {
//             _ = async {
//                 let data = imu.read();
//                 tx.send(data).await.expect("send imudata");
//                 // debug!("send imu data");
//             } => {}

//             Some((cmd, sender)) = cmd_rx.recv() => {
//                 match cmd {
//                     IMUCommand::Calibration => {
//                         // info!("calibration start");
//                         imu.calibrate();
//                         // info!("calibration end");
//                         sender.send(IMUCommandReturn::CalibrationFinished).unwrap();
//                     },
//                 }
//             }
//         }
//     }
// }
