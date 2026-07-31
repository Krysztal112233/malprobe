pub mod files;

use sea_orm::{ConnectionTrait, TransactionTrait};

/// Anything that can serve as a database connection for helper operations:
/// both `DatabaseConnection` and `Transaction` implement this, so helpers can
/// run inside a caller-managed transaction.
pub trait SafeTransactionConnectionTrait: TransactionTrait + ConnectionTrait + Sync + Send {}

impl<T> SafeTransactionConnectionTrait for T where
    T: TransactionTrait + ConnectionTrait + Send + Sync
{
}
