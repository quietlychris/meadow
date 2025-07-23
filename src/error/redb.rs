use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors coming from `redb`-based operations
#[derive(Clone, Debug, Error, Eq, PartialEq, Serialize, Deserialize)]
pub enum RedbError {
    /// Transparent error around `redb`'s [`TransactionError`]
    // TO_DO: This should be transparent, but upstream is missing required traits
    #[error("Based on redb `TransactionError`")]
    TransactionError,
    #[error("Baed on redb `DatabaseError`")]
    DatabaseError,
}
