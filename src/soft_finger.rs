use burn::tensor::activation::leaky_relu;
use burn_ndarray::{NdArray, NdArrayDevice};
use nalgebra::Vector6;

use crate::aruco_finder::Aruco;

use burn::record::{NamedMpkFileRecorder, Recorder};
use burn::{
    module::Module,
    nn::{Linear, LinearConfig},
    record::FullPrecisionSettings,
    tensor::{backend::Backend, Tensor},
};

// nn.Linear(6, 1000),
// nn.Linear(1000, 100),
// nn.LeakyReLU(),
// nn.Linear(100, 50),
// nn.LeakyReLU(),
// nn.Linear(50, 6)
#[derive(Module, Debug)]
struct Net<B: Backend> {
    l0: Linear<B>,
    l2: Linear<B>,
    // relu1: LeakyRelu<B>,
    l4: Linear<B>,
    // relu2: LeakyRelu<B>,
    l6: Linear<B>,
}

impl<B: Backend> Net<B> {
    pub fn init(device: &B::Device) -> Self {
        let l0 = LinearConfig::new(6, 1000).init(device);
        let l2 = LinearConfig::new(1000, 100).init(device);
        let l4 = LinearConfig::new(100, 50).init(device);
        let l6 = LinearConfig::new(50, 6).init(device);
        Net { l0, l2, l4, l6 }
    }

    pub fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        let x = self.l0.forward(x);
        let x = self.l2.forward(x);
        let x = leaky_relu(x, 0.01);
        let x = self.l4.forward(x);
        let x = leaky_relu(x, 0.01);
        self.l6.forward(x)
    }
}

fn load_model(path: &str) -> Net<NdArray> {
    type Backend = burn_ndarray::NdArray<f32>;
    let device = Default::default();
    let load_args = burn_import::pytorch::LoadArgs::new(path.into())
        // .with_debug_print()
        .with_key_remap(r"model\.(\d)", "l$1");
    let record = burn_import::pytorch::PyTorchFileRecorder::<FullPrecisionSettings>::default()
        .load(load_args, &device)
        .expect("Should decode state successfully");
    Net::<Backend>::init(&device).load_record(record)
}

pub fn convert() {
    type Backend = burn_ndarray::NdArray<f32>;
    let device = Default::default();

    // Load PyTorch weights into a model record.
    let load_args = burn_import::pytorch::LoadArgs::new("./model.pth".into())
        // .with_debug_print()
        .with_key_remap(r"model\.(\d)", "l$1");
    let record: NetRecord<Backend> =
        burn_import::pytorch::PyTorchFileRecorder::<FullPrecisionSettings>::default()
            .load(load_args, &device)
            .expect("Failed to decode state");

    // Save the model record to a file.
    let recorder = NamedMpkFileRecorder::<FullPrecisionSettings>::default();

    recorder
        .record(record, "./model".into())
        .expect("Failed to save model record");
}

pub struct SoftFinger {
    model: Net<NdArray>,
}

#[derive(Debug)]
pub struct Force {
    pub value: Vector6<f32>,
}

impl SoftFinger {
    pub fn new_pt(pt_path: &str) -> Self {
        SoftFinger {
            model: load_model(pt_path).no_grad(),
        }
    }

    pub fn predict_force(&self, aruco: Aruco) -> Force {
        // Isometry3::
        let x = self.model.forward(Tensor::<NdArray, 2>::from_data(
            [[
                aruco.trans[0] as f32,
                aruco.trans[1] as f32,
                aruco.trans[2] as f32,
                aruco.euler_angles[0] as f32,
                aruco.euler_angles[1] as f32,
                aruco.euler_angles[2] as f32,
            ]],
            &NdArrayDevice::default(),
        ));
        let data = x.to_data();
        Force {
            value: Vector6::from_vec(data.value),
        }
    }
}
