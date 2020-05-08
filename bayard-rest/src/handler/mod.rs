use iron::typemap::Key;

use bayard_client::index::client::IndexClient;

pub mod bulk_delete;
pub mod bulk_set;
pub mod commit;
pub mod delete;
pub mod get;
pub mod merge;
pub mod rollback;
pub mod schema;
pub mod search;
pub mod set;
pub mod status;

#[derive(Copy, Clone)]
pub struct Client;

impl Key for Client {
    type Value = IndexClient;
}
