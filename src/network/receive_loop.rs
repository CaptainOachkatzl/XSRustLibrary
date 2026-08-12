use std::{
    error::Error,
    net::Shutdown,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use crate::{
    connection::Connection,
    events::{Invokable, Subscribable, event::Event},
};

type EventHandler = dyn Fn(&Vec<u8>) + Send + Sync;

pub struct ReceiveLoop {
    stop: Arc<AtomicBool>,
}

impl ReceiveLoop {
    pub fn start<T: Connection<ErrorType = E> + Send + 'static, E: Error>(
        mut connection: T,
        packet_received_handlers: Vec<Box<EventHandler>>,
    ) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let receive_loop = Self { stop: stop.clone() };

        thread::spawn(move || {
            let mut receive_event = Event::new();
            let mut receive_buffer = Vec::new();
            let _subscriptions: Vec<_> = packet_received_handlers
                .into_iter()
                .map(|handler| receive_event.subscribe(handler))
                .collect();

            while !stop.load(Ordering::Relaxed) {
                let receive_result = connection.receive_into(&mut receive_buffer);
                match receive_result {
                    Ok(_) => receive_event.invoke(&receive_buffer),
                    Err(_) => break,
                }
            }
            connection.shutdown(Shutdown::Read).unwrap();
        });

        receive_loop
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }

    pub fn stopped(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }
}
