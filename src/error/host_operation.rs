use core::fmt::{Display, Formatter};
use serde::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HostOperation {
    Success,
    SetFailure,
    GetFailure,
    ConnectionError,
}

impl std::error::Error for HostOperation {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use HostOperation::*;
        match *self {
            Success => None,
            SetFailure => None,
            GetFailure => None,
            ConnectionError => None,
        }
    }
}

impl Display for HostOperation {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        use HostOperation::*;
        match *self {
            Success => write!(f, "Success Host-side operation"),
            SetFailure => write!(f, "Unsuccessful Host-side SET operation"),
            GetFailure => write!(f, "Unsuccessful Host-side SET operation"),
            ConnectionError => write!(f, "Unsuccessful Host connection"),
        }
    }
}

impl HostOperation {
    pub fn as_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(&self).unwrap()
    }
}
