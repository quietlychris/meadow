use core::fmt::{Display, Formatter};
use serde::*;

#[cfg(feature = "quic")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Quic {
    QuicIssue,
    BadGenericMsg,
    OpenBi,
    RecvRead,
    Connection,
    // Error accessing an owned Endpoint
    AccessEndpoint,
    EndpointConnect,
    FindKeys,
    ReadKeys,
    FindCerts,
    ReadCerts,
    Configuration,
    EndpointCreation,
}

#[cfg(feature = "quic")]
impl std::error::Error for Quic {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use Quic::*;
        match *self {
            QuicIssue => None,
            BadGenericMsg => None,
            OpenBi => None,
            RecvRead => None,
            Connection => None,
            // Error accessing an owned Endpoint
            AccessEndpoint => None,
            EndpointConnect => None,
            FindKeys => None,
            ReadKeys => None,
            FindCerts => None,
            ReadCerts => None,
            Configuration => None,
            EndpointCreation => None,
        }
    }
}

#[cfg(feature = "quic")]
impl Display for Quic {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        use Quic::*;
        match self {
            QuicIssue => write!(f, "Generic Quic Issue"),
            BadGenericMsg => write!(f, "Error with deserialization into proper GenericMsg"),
            OpenBi => write!(f, "Error opening bidirectional stream from connection"),
            RecvRead => write!(f, "Error reading bytes from stream receiver"),
            Connection => write!(f, "Error acquiring owned connection"),
            // Error accessing an owned Endpoint
            AccessEndpoint => write!(f, "Unable to access owned Endpoint"),
            EndpointConnect => write!(f, "Unable to establish Connection to remote Endpoint"),
            FindKeys => write!(f, "Unable to find .pem key file"),
            ReadKeys => write!(f, "Error reading .pem key file"),
            FindCerts => write!(f, "Unable to find .pem certificates"),
            ReadCerts => write!(f, "Error reading .pem certificates"),
            Configuration => write!(f, "Error configuring server with certificate"),
            &EndpointCreation => write!(f, "Error creating endpoint from configuration parameters"),
        }
    }
}

#[cfg(feature = "quic")]
impl Quic {
    pub fn as_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(&self).unwrap()
    }
}
