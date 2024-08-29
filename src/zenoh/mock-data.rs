use std::{
    thread::{sleep, spawn},
    time::{Duration, Instant, SystemTime},
    u16,
};

use nalgebra::Vector6;
use rpi::{AngleData, FingerForceData, Force};
use zenoh::prelude::sync::*;

fn main() {
    spawn(mock_angle_data);
    spawn(|| mock_finger_data("finger/left/force"));
    mock_finger_data("finger/right/force");
}

fn mock_angle_data() {
    let session = zenoh::open(config::default()).res().unwrap();
    let start = Instant::now();
    let data_pub = session
        .declare_publisher("angle/data")
        .res()
        .unwrap()
        .priority(Priority::RealTime)
        .congestion_control(CongestionControl::Drop);
    loop {
        sleep(Duration::from_millis(10));
        let time_stamp = start.elapsed();
        data_pub
            .put(
                serde_json::to_value(AngleData {
                    data: (SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64()
                        .cos()
                        * 1024.
                        / 2.
                        + 1024. / 2.) as u16,
                    time_stamp,
                })
                .unwrap(),
            )
            .res()
            .unwrap();
    }
}

fn mock_finger_data(key: &str) {
    let session = zenoh::open(config::default()).res().unwrap();
    let force_pub = session
        .declare_publisher(key)
        .res()
        .unwrap();
    let start = Instant::now();
    loop {
        sleep(Duration::from_micros(16666));
        let x = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
            .cos() as f32;
        let force_data = FingerForceData {
            force: Some(Force {
                value: Vector6::new(x, x, x, x, x, x),
            }),
            time_stamp: start.elapsed(),
        };
        force_pub
            .put(serde_json::to_value(force_data).unwrap())
            .res()
            .unwrap();
    }
}
