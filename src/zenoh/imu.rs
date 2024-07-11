use rpi::{
    {CSVFile, DataFile},
    {IMUData, IMU},
    RecordCommand,
};
use zenoh::prelude::sync::*;

fn main() {
    let session = zenoh::open(config::default()).res().unwrap();
    let data_pub = session
        .declare_publisher("imu/data")
        .res()
        .unwrap()
        .priority(Priority::RealTime)
        .congestion_control(CongestionControl::Drop);
    let calibration_cmd_sub = session.declare_subscriber("imu/cmd").res().unwrap();
    let record_cmd_subscriber = session.declare_subscriber("cmd/record").res().unwrap();
    let mut imu = IMU::new("/dev/i2c-1");
    imu.init();
    let mut csv_file = CSVFile::<IMUData>::new();
    loop {
        if let Ok(_cmd) = calibration_cmd_sub.try_recv() {
            println!("calibrate start");
            imu.calibrate();
            println!("calibrate end");
        }
        let data = imu.read();
        if let Ok(cmd) = record_cmd_subscriber.try_recv() {
            let cmd_json = cmd.value.try_into().unwrap();
            let cmd = serde_json::from_value::<RecordCommand>(cmd_json).unwrap();
            match cmd {
                RecordCommand::Start { imu, .. } => {
                    if !csv_file.is_started() {
                        csv_file.start_new(&imu, ())
                    }
                }
                RecordCommand::End => {
                    if csv_file.is_started() {
                        csv_file.end()
                    }
                }
            }
        }
        if csv_file.is_started() {
            csv_file.record(data.clone());
        }
        let put = data_pub.put(serde_json::to_value(data).unwrap());
        put.res().unwrap();
    }
}
