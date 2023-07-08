use std::{
    cell::RefCell,
    net::Shutdown,
    sync::atomic::{AtomicBool, Ordering},
};

use displaydoc::Display;
use thiserror::Error;

use crate::{
    events::{event::Event, subscription::Subscription, Invokable, Subscribable},
    packet_connection,
};

use super::packet_connection::PacketConnection;

type EventHandler = dyn Fn(&Vec<u8>) + Send + Sync;

#[derive(Debug, Display, Error)]
pub enum Error {
    /// {0}
    PacketConnection(#[from] packet_connection::Error),
}

pub struct PacketReceiveEvent {
    packet_connection: RefCell<PacketConnection>,
    receive_event: RefCell<Event<Vec<u8>>>,
    started: AtomicBool,
    stop: AtomicBool,
}

impl PacketReceiveEvent {
    pub fn new(packet_connection: PacketConnection) -> PacketReceiveEvent {
        PacketReceiveEvent {
            packet_connection: RefCell::new(packet_connection),
            receive_event: RefCell::new(Event::new()),
            started: AtomicBool::new(false),
            stop: AtomicBool::new(false),
        }
    }

    pub fn start(&self) {
        // guarantee only one thread can ever pass through and execute loop
        if !self.locked_start_check() {
            return;
        }

        while !self.stop.load(Ordering::SeqCst) {
            let receive_result = self.packet_connection.borrow_mut().receive();
            match receive_result {
                Ok(v) => self.receive_event.borrow_mut().invoke(&v),
                _ => self.stop().unwrap(),
            };
        }
    }

    fn locked_start_check(&self) -> bool {
        self.set_atomic_bool(&self.started)
    }

    fn locked_stop_check(&self) -> bool {
        self.set_atomic_bool(&self.stop)
    }

    // returns true if the bool was set to true
    fn set_atomic_bool(&self, value: &AtomicBool) -> bool {
        value.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok()
    }

    pub fn stop(&self) -> Result<(), Error> {
        if !self.locked_stop_check() {
            return Ok(());
        }

        self.packet_connection.borrow().shutdown(Shutdown::Both)?;
        Ok(())
    }

    pub fn subscribe(&mut self, subscriber: Box<EventHandler>) -> Subscription<Vec<u8>> {
        self.receive_event.borrow_mut().subscribe(subscriber)
    }

    pub fn try_clone(&self) -> Result<PacketReceiveEvent, Error> {
        Ok(PacketReceiveEvent {
            packet_connection: RefCell::new(self.packet_connection.borrow().try_clone()?),
            receive_event: RefCell::new(Event::new()),
            started: AtomicBool::new(self.started.load(Ordering::SeqCst)),
            stop: AtomicBool::new(self.stop.load(Ordering::SeqCst)),
        })
    }
}
