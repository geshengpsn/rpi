use std::time::{Duration, Instant};

use rpi::AngleData;
use socketcan::{CanDataFrame, CanSocket, EmbeddedFrame, Socket, StandardId};
use zenoh::prelude::sync::*;

fn main() {
    let session = zenoh::open(config::default()).res().unwrap();
    let can = CanSocket::open("can0").unwrap();
    let start = Instant::now();
    let data_pub = session
        .declare_publisher("angle/data")
        .res()
        .unwrap()
        .priority(Priority::RealTime)
        .congestion_control(CongestionControl::Drop);
    let mut count = 0;
    loop {
        can.write_frame(
            &CanDataFrame::new(StandardId::new(1).unwrap(), &[0x04, 0x01, 0x01, 0x00]).unwrap(),
        )
        .unwrap();
        if let Ok(data_frame) = can.read_frame() {
            let time_stamp = start.elapsed();
            let raw = data_frame.data();
            let data = u16::from_le_bytes([raw[3], raw[4]]);
            count += 1;
            if count >= 3 {
                data_pub
                    .put(serde_json::to_value(AngleData { data, time_stamp }).unwrap())
                    .res()
                    .unwrap();
                count = 0;
            }
        }
    }
}
