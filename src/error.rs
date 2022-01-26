// TO_DO: Integrate error codes into the rest of the library
#![allow(unused_variables)]

use core::fmt::{Display, Formatter};

/// This is the error type used by Postcard
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
pub enum Error {
    /// Nodes are required to be assigned a topic before they can be built
    RequireTopic,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        use Error::*;
        write!(
            f,
            "{}",
            match self {
                RequireTopic => "Nodes are required to be assigned a topic before they can be built",
            }
        )
    }
}

/// This is the Result type used by Postcard.
pub type Result<T> = ::core::result::Result<T, Error>;
