use core::fmt::{Display, Formatter};
use serde::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HostOperation {
    SetFailure,
    GetFailure,
    ConnectionError,
}

impl std::error::Error for HostOperation {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use HostOperation::*;
        match *self {
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
            SetFailure => write!(f, "Unsuccessful Host-side SET operation"),
            GetFailure => write!(f, "Unsuccessful Host-side SET operation"),
            ConnectionError => write!(f, "Unsuccessful Host connection"),
        }
    }
}
