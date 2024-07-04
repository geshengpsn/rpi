use serde::Serialize;


#[derive(Debug, Serialize)]
pub struct IMUData {
    v: [f64;3]
}

struct IMU {
}

impl IMU {
    fn new() -> Self {
        todo!()
    }

    fn read() -> IMUData {
        todo!()
    }
}