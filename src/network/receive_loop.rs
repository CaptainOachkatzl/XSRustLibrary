use std::{
    error::Error,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    connection::Connection,
    events::{Invokable, Subscribable, event::Event, subscription::Subscription},
};

type EventHandler = dyn Fn(&Vec<u8>) + Send + Sync;

pub struct ReceiveLoop<T: Connection<ErrorType = E>, E: Error> {
    connection: T,
    receive_event: Event<Vec<u8>>,
    receive_buffer: Vec<u8>,
    started: bool,
    stop: Arc<AtomicBool>,
}

impl<T: Connection<ErrorType = E>, E: Error> ReceiveLoop<T, E> {
    pub fn new(connection: T, stop: Arc<AtomicBool>) -> Self {
        Self {
            connection: connection,
            receive_event: Event::new(),
            receive_buffer: Vec::new(),
            started: false,
            stop,
        }
    }

    pub fn start(&mut self) {
        if self.started {
            return;
        }

        while !self.stop.load(Ordering::Relaxed) {
            let receive_result = self.connection.receive_into(&mut self.receive_buffer);
            match receive_result {
                Ok(_) => self.receive_event.invoke(&self.receive_buffer),
                Err(_) => return,
            }
        }
    }

    #[must_use]
    pub fn subscribe(&mut self, subscriber: Box<EventHandler>) -> Subscription<Vec<u8>> {
        self.receive_event.subscribe(subscriber)
    }
}
