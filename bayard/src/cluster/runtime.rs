use std::time::Duration;

use bytes::{BufMut, Bytes, BytesMut};
use foca::{Identity, Notification, Runtime, Timer};
use tracing::debug;

pub struct AccumulatingRuntime<T> {
    pub to_send: Vec<(T, Bytes)>,
    pub to_schedule: Vec<(Duration, Timer<T>)>,
    pub notifications: Vec<Notification<T>>,
    buf: BytesMut,
}

impl<T: Identity> Runtime<T> for AccumulatingRuntime<T> {
    // Notice that we'll interact to these via pop(), so we're taking
    // them in reverse order of when it happened.
    // That's perfectly fine, the order of items from a single interaction
    // is irrelevant. A "nicer" implementation could use VecDeque or
    // react directly here instead of accumulating.

    fn notify(&mut self, notification: Notification<T>) {
        debug!(?notification, "Notification received.");
        self.notifications.push(notification);
    }

    fn send_to(&mut self, to: T, data: &[u8]) {
        debug!(?to, ?data, "Sending data.");
        let mut packet = self.buf.split();
        packet.put_slice(data);
        self.to_send.push((to, packet.freeze()));
    }

    fn submit_after(&mut self, event: Timer<T>, after: Duration) {
        debug!(?event, ?after, "Scheduling event.");
        // We could spawn+sleep here
        self.to_schedule.push((after, event));
    }
}

impl<T> Default for AccumulatingRuntime<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AccumulatingRuntime<T> {
    pub fn new() -> Self {
        Self {
            to_send: Vec::new(),
            to_schedule: Vec::new(),
            notifications: Vec::new(),
            buf: BytesMut::new(),
        }
    }

    pub fn backlog(&self) -> usize {
        debug!(
            "Backlog: to_send={}, to_schedule={}, notifications={}",
            self.to_send.len(),
            self.to_schedule.len(),
            self.notifications.len()
        );
        self.to_send.len() + self.to_schedule.len() + self.notifications.len()
    }
}
