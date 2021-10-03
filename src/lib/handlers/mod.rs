pub mod edits;
pub mod sms;
pub mod trigger;
pub mod ww;

use crate::db::Error as DBError;
use flobot_lib::handler::Error as HandlerError;

impl From<DBError> for HandlerError {
    fn from(e: DBError) -> Self {
        match e {
            DBError::Migration(e) => HandlerError::Other(e),
            DBError::Database(e) => HandlerError::Database(e),
        }
    }
}
