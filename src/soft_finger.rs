use burn::tensor::activation::relu;
use burn_ndarray::{NdArray, NdArrayDevice};
use nalgebra::Vector6;
use serde::{Deserialize, Serialize};

use crate::aruco_finder::Aruco;
use crate::data_saver::FrameData;

use burn::record::Recorder;
use burn::{
    module::Module,
    nn::{Linear, LinearConfig},
    record::FullPrecisionSettings,
    tensor::{backend::Backend, Tensor},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerForceData {
    pub force: Option<Force>,
    pub time_stamp: std::time::Duration,
}

impl FrameData for FingerForceData {
    fn time_stamp(&self) -> std::time::Duration {
        self.time_stamp
    }
}

#[derive(Module, Debug)]
struct Net<B: Backend> {
    fc1: Linear<B>,
    fc2: Linear<B>,
    fc3: Linear<B>,
    fc4: Linear<B>,
}

impl<B: Backend> Net<B> {
    pub fn init(device: &B::Device) -> Self {
        let fc1 = LinearConfig::new(8, 150).init(device);
        let fc2 = LinearConfig::new(150, 200).init(device);
        let fc3 = LinearConfig::new(200, 200).init(device);
        let fc4 = LinearConfig::new(200, 6).init(device);
        Net { fc1, fc2, fc3, fc4 }
    }

    pub fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        let x = relu(self.fc1.forward(x));
        let x = relu(self.fc2.forward(x));
        let x = relu(self.fc3.forward(x));
        self.fc4.forward(x)
    }
}

fn load_model(path: &str) -> Net<NdArray> {
    type Backend = burn_ndarray::NdArray<f32>;
    let device = Default::default();
    let load_args = burn_import::pytorch::LoadArgs::new(path.into());
        // .with_debug_print()
        // .with_key_remap(r"model\.(\d)", "l$1");
    let record = burn_import::pytorch::PyTorchFileRecorder::<FullPrecisionSettings>::default()
        .load(load_args, &device)
        .expect("Should decode state successfully");
    Net::<Backend>::init(&device).load_record(record)
}

// pub fn convert() {
//     type Backend = burn_ndarray::NdArray<f32>;
//     let device = Default::default();

//     // Load PyTorch weights into a model record.
//     let load_args = burn_import::pytorch::LoadArgs::new("./model.pth".into())
//         // .with_debug_print()
//         .with_key_remap(r"model\.(\d)", "l$1");
//     let record: NetRecord<Backend> =
//         burn_import::pytorch::PyTorchFileRecorder::<FullPrecisionSettings>::default()
//             .load(load_args, &device)
//             .expect("Failed to decode state");

//     // Save the model record to a file.
//     let recorder = NamedMpkFileRecorder::<FullPrecisionSettings>::default();

//     recorder
//         .record(record, "./model".into())
//         .expect("Failed to save model record");
// }

pub struct SoftFinger {
    model: Net<NdArray>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Force {
    pub value: Vector6<f32>,
}

impl SoftFinger {
    pub fn new_pt(pt_path: &str) -> Self {
        SoftFinger {
            model: load_model(pt_path).no_grad(),
        }
    }

    pub fn predict_force(&self, aruco: &Aruco) -> Force {
        println!("{aruco:?}");
        let x = self.model.forward(Tensor::<NdArray, 2>::from_data(
            [[
                aruco.corners[0][0],
                aruco.corners[0][1],
                aruco.corners[1][0],
                aruco.corners[1][1],
                aruco.corners[2][0],
                aruco.corners[2][1],
                aruco.corners[3][0],
                aruco.corners[3][1],
            ]],
            &NdArrayDevice::default(),
        ));
        let data = x.to_data();
        Force {
            value: Vector6::from_vec(data.value),
        }
    }
}
