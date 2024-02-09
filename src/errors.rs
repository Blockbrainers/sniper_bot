use std::{error::Error, fmt};

// Define a wrapper for errors that implements Send
pub struct SendableError(Box<dyn Error + Send>);

impl SendableError {
    // Create a new SendableError from any error type that implements Error + Send
    pub fn new<E: Error + Send + 'static>(err: E) -> Self {
        SendableError(Box::new(err))
    }
}

impl fmt::Debug for SendableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SendableError: {:?}", self.0)
    }
}

impl fmt::Display for SendableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SendableError: {}", self.0)
    }
}

impl Error for SendableError {}

impl From<String> for SendableError {
    fn from(message: String) -> Self {
        SendableError(Box::new(std::io::Error::new(std::io::ErrorKind::Other, message)))
    }
}

impl<'a> From<&'a str> for SendableError {
    fn from(message: &'a str) -> Self {
        SendableError(Box::new(std::io::Error::new(std::io::ErrorKind::Other, message.to_string())))
    }
}