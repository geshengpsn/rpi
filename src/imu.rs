use std::{thread::{spawn, JoinHandle}, time::Instant};
use crossbeam::channel::Sender;
use serde::Serialize;
use linux_embedded_hal::{Delay, I2cdev};
use mpu6050_dmp::{
    address::Address, calibration::CalibrationParameters, quaternion::Quaternion, sensor::Mpu6050,
};
use crate::data_saver::FrameData;

#[derive(Debug, Clone, Serialize)]
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
}

impl IMU {
    pub fn new(path: &str) -> Self {
        let i2c = I2cdev::new(path).unwrap();
        let mpu6050 = Mpu6050::new(i2c, Address::default()).unwrap();
        Self {
            mpu6050,
            buf: [0; 28],
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

    fn init(&mut self) {
        self.mpu6050.initialize_dmp(&mut Delay).unwrap();
    }
}

pub fn spawn_imu(mut imu: IMU, tx: Sender<IMUData>) -> JoinHandle<()> {
    spawn(move || {
        imu.init();
        let start = Instant::now();
        loop {
            let len = imu.mpu6050.get_fifo_count().expect("get_fifo_count");
            if len >= 28 {
                let time_stamp = start.elapsed();
                imu.mpu6050.read_fifo(&mut imu.buf).expect("read_fifo");
                let quat = Quaternion::from_bytes(&imu.buf[..16]).unwrap().normalize();
                let quat = nalgebra::UnitQuaternion::new_normalize(nalgebra::Quaternion::new(
                    quat.w, quat.x, quat.y, quat.z,
                ));
                tx.send(IMUData { quat, time_stamp }).expect("send imudata");
            }
        }
    })
}
