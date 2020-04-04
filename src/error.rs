use std::{io, ffi};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ReqwestError(reqwest::Error),
    IoError(io::Error),
    CStringNulError(ffi::NulError),
    NonSuccessStatusError(),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::ReqwestError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<ffi::NulError> for Error {
    fn from(err: ffi::NulError) -> Self {
        Error::CStringNulError(err)
    }
}
