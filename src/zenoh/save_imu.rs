use rpi::{CSVFile, DataFile, IMUData, RecordCommand};
use zenoh::prelude::sync::*;
fn main() {
    let session = zenoh::open(config::default()).res().unwrap();
    let data_subscriber = session.declare_subscriber("imu/data").res().unwrap();
    let cmd_subscriber = session.declare_subscriber("cmd/record").res().unwrap();
    let mut csv_file = CSVFile::<IMUData>::new();

    while let Ok(sample) = data_subscriber.recv() {
        if let Ok(cmd) = cmd_subscriber.try_recv() {
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
            let json = sample.value.try_into().unwrap();
            let data = serde_json::from_value::<IMUData>(json).unwrap();
            csv_file.record(data);
        }
    }
}
