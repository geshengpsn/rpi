use std::{io, time::Duration};

use crate::Result;
// use opencv::{
//     core::{Mat, CV_8UC3},
//     imgproc::{cvt_color_def, COLOR_RGB2BGR},
// };

use std::fmt::Debug;
// use tokio::sync::mpsc;
use v4l::{
    buffer::Metadata,
    frameinterval::FrameIntervalEnum,
    framesize::FrameSizeEnum,
    io::traits::CaptureStream,
    prelude::MmapStream,
    video::{capture::Parameters, Capture},
    Device, Format, FourCC, Fraction,
};

pub struct Camera<'a> {
    device: Device,
    stream: Option<MmapStream<'a>>,
    fps: u32,
    width: u32,
    height: u32,
    format: FourCC,
    // index: usize,
    rgb_buffer: Vec<u8>,
}

impl Debug for Camera<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Camera")
            .field("fps", &self.fps)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("format", &self.format.to_string())
            .finish()
    }
}

impl Camera<'_> {
    pub fn new_with_path(path: &str, width: u32, height: u32, fps: u32) -> Result<Self> {
        let dev = Device::with_path(path).unwrap_or_else(|_| panic!("can not found camera:{path}"));
        Self::new_from_device(dev, width, height, fps)
    }

    pub fn new(index: usize, width: u32, height: u32, fps: u32) -> Result<Self> {
        let dev = Device::new(index).unwrap_or_else(|_| panic!("can not found camera{index}"));
        Self::new_from_device(dev, width, height, fps)
    }

    fn new_from_device(device: Device, width: u32, height: u32, fps: u32) -> Result<Self> {
        // let device = Device::new(index).unwrap_or_else(|_| panic!("can not found camera{index}"));
        let mut choosed_format = None;
        for format in device
            .enum_formats()
            .unwrap_or_else(|_| panic!("enum formats fail (camera)"))
        {
            for frame_size in device
                .enum_framesizes(format.fourcc)
                .unwrap_or_else(|_| panic!("enum framesizes fail (camera)"))
            {
                if let FrameSizeEnum::Discrete(size) = frame_size.size {
                    for fi in device
                        .enum_frameintervals(format.fourcc, size.width, size.height)
                        .unwrap_or_else(|_| panic!("enum frameintervals fail (camera)"))
                    {
                        if let FrameIntervalEnum::Discrete(fraction) = fi.interval {
                            if size.width == width
                                && size.height == height
                                && fraction.denominator == fps
                            {
                                choosed_format = Some((width, height, format.fourcc));
                            }
                        }
                    }
                }
            }
        }

        // let caps = device.query_controls().unwrap();
        // device.set_control(Control{
        //     id: 0x00980900,
        //     value: v4l::control::Value::Integer(64)
        // }).unwrap();
        // println!("{caps}");
        if choosed_format.is_none() {
            // 可能是参数设置不好，don’t panic
            return Err(io::Error::other("no camera availbale"))?;
        }

        let (width, height, fourcc) = choosed_format.unwrap();
        let real_format = device
            .set_format(&Format::new(width, height, fourcc))
            .unwrap_or_else(|_| panic!("set format fail (camera)"));
        let real_params = device
            .set_params(&Parameters::new(Fraction::new(1, fps)))
            .unwrap_or_else(|_| panic!("set params fail (camera)"));
        let mut cam = Camera {
            stream: None,
            device,
            // index,
            fps: real_params.interval.denominator,
            format: real_format.fourcc,
            width: real_format.width,
            height: real_format.height,
            rgb_buffer: vec![0u8; (real_format.height * real_format.width * 3) as usize],
        };
        cam.open();
        Ok(cam)
    }

    fn open(&mut self) {
        let stream = MmapStream::new(&self.device, v4l::buffer::Type::VideoCapture)
            .unwrap_or_else(|_| panic!("new mmap stream fail (camera)"));
        // stream.start()?;
        self.stream = Some(stream);
    }

    pub fn capture_mjpeg(&mut self) -> Result<(Vec<u8>, Duration)> {
        assert!(self.stream.is_some());

        // let start = std::time::Instant::now();
        let (raw_mjpeg, Metadata { timestamp, .. }) = self.stream.as_mut().unwrap().next()?;
        // println!("      stream next {:?}", start.elapsed());

        Ok((
            raw_mjpeg.to_vec(),
            Duration::new(timestamp.sec as u64, (timestamp.usec * 1000) as u32),
        ))
    }

    pub fn capture(&mut self) -> Result<(&[u8], Duration)> {
        use zune_jpeg::JpegDecoder;

        // let start = std::time::Instant::now();
        let (raw_mjpeg, Metadata { timestamp, .. }) = self.stream.as_mut().unwrap().next()?;
        // println!("      stream next {:?}", start.elapsed());

        // let start = std::time::Instant::now();
        let mut decoder = JpegDecoder::new(raw_mjpeg);
        decoder.decode_into(&mut self.rgb_buffer)?; // shouldn't happend
                                                            // println!("      decode {:?}", start.elapsed());

        Ok((
            &self.rgb_buffer,
            Duration::new(timestamp.sec as u64, (timestamp.usec * 1000) as u32),
        ))
    }
}

// pub fn mat_from_ptr(ptr: *const u8, width: i32, height: i32) -> Result<Mat> {
//     let img =
//         unsafe { Mat::new_rows_cols_with_data_unsafe_def(height, width, CV_8UC3, ptr as *mut _) }?;
//     let mut res_img = Mat::default();
//     cvt_color_def(&img, &mut res_img, COLOR_RGB2BGR)?;
//     Ok(res_img)
// }

// pub async fn usbcamera(
//     tx: mpsc::Sender<(Mat, Duration)>,
//     aruco_camera_index: usize,
//     width: u32,
//     height: u32,
//     fps: u32,
// ) {
//     let mut cam =
//         Camera::new(aruco_camera_index, width, height, fps).expect("camera bad parameters");
//     loop {
//         let (raw_img, ts) = cam.capture().unwrap();
//         let img = mat_from_ptr(raw_img.as_ptr(), width as i32, height as i32).unwrap();
//         // tx.tr
//         tx.send((img, ts)).await.unwrap();
//         // tracing::debug!("camera capture");
//     }
// }
