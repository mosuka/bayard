use iron::typemap::Key;

use bayard_client::index::client::IndexClient;

pub mod server;
pub mod handler;

#[derive(Copy, Clone)]
pub struct Client;

impl Key for Client {
    type Value = IndexClient;
}
