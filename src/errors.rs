use std::{io, sync::PoisonError};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    OpenCV(opencv::Error),
    V4L2(io::Error),
    JPEGDecoder(zune_jpeg::errors::DecodeErrors),
    ChannelSend,
    ChannelRecv,
    Lock,
}

impl From<opencv::Error> for Error {
    fn from(value: opencv::Error) -> Self {
        Error::OpenCV(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::V4L2(value)
    }
}

impl From<zune_jpeg::errors::DecodeErrors> for Error {
    fn from(value: zune_jpeg::errors::DecodeErrors) -> Self {
        Error::JPEGDecoder(value)
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Error::Lock
    }
}
