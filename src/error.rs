#![allow(missing_docs)]

use failure::Fail;

//#[cfg(not(feature = "tls"))]
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO error: {}", error)]
    Io {
        #[cause]
        error: std::io::Error,
    },
    #[fail(display = "Failed to convert to UTF-8: {}", error)]
    Utf8 {
        #[cause]
        error: std::string::FromUtf8Error,
    },
    #[fail(display = "Failed to parse IRC message: {}", error)]
    ParseError {
        #[cause]
        error: pircolate::error::MessageParseError,
    },
    #[cfg(feature = "tls")]
    #[fail(display = "TLS error: {}", error)]
    Tls {
        #[cause]
        error: native_tls::Error,
    },
    #[fail(display = "Connection to remote host was reset.")]
    ConnectionReset,
    #[fail(display = "Unexpected error")]
    UnexpectedError,
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::Io { error }
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Error {
        Error::Utf8 { error }
    }
}

impl From<pircolate::error::MessageParseError> for Error {
    fn from(error: pircolate::error::MessageParseError) -> Error {
        Error::ParseError { error }
    }
}

#[cfg(feature = "tls")]
impl From<native_tls::Error> for Error {
    fn from(error: native_tls::Error) -> Error {
        Error::Tls { error }
    }
}
