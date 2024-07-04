use std::{io, time::Instant};

use embedded_graphics::{
    draw_target::{DrawTarget, DrawTargetExt},
    geometry::{Dimensions, Point},
    image::{Image, ImageRaw},
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::{Bgr888, Rgb565, Rgb888, RgbColor},
    text::{Alignment, Text},
    Drawable,
};
use opencv::{
    core::{Mat, MatTraitConstManual, Size},
    imgproc::{resize, INTER_LINEAR}, videoio::{VideoCapture, VideoCaptureTrait, CAP_V4L},
};
use rpi_st7789v2_driver::{Command, Driver};

pub struct Screen {
    driver: Driver,
}

impl Screen {
    pub fn new() -> io::Result<Self> {
        let mut driver = Driver::new(Default::default()).map_err(io::Error::other)?;
        driver.init().map_err(io::Error::other)?;
        driver.probe_buffer_length().map_err(io::Error::other)?;

        Ok(Screen { driver })
    }

    pub fn draw_image(&mut self, mat: &Mat) -> io::Result<()> {
        

        let mut buffer = self.driver.image();
        buffer.bounding_box();
        let mut dst = Mat::default();
        let rect = buffer.bounding_box();
        println!("{mat:?}");
        resize(
            &mat,
            &mut dst,
            Size::new(rect.size.width as i32, rect.size.height as i32),
            0.,
            0.,
            INTER_LINEAR,
        )
        .map_err(io::Error::other)?;
        let data = dst.data_bytes().map_err(io::Error::other)?;
        let image_raw = ImageRaw::<Bgr888>::new(data, rect.size.width);
        let image = Image::new(&image_raw, Point::zero());

        let mut target = buffer.color_converted::<Bgr888>();
        image.draw(&mut target).map_err(io::Error::other)?;

        self.driver.print((0, 0), &buffer).unwrap();

        
        Ok(())
    }
}

pub fn test_screen() {
    let mut screen = Screen::new().unwrap();
    let mut vc = VideoCapture::new(0, CAP_V4L).unwrap();
    let mut image = Mat::default();
    loop {
        let start = Instant::now();
        let res = vc.read(&mut image);
        if res.is_ok() {
            screen.draw_image(&image).unwrap();
        }
        println!("{:?}", start.elapsed());
    }
}
