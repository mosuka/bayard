use iron::typemap::Key;

use bayard_client::index::client::IndexClient;

pub mod handler;
pub mod server;

#[derive(Copy, Clone)]
pub struct Client;

impl Key for Client {
    type Value = IndexClient;
}
