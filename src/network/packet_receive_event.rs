use std::{
  io::Result,
  cell::RefCell,
  sync::{
    atomic::{AtomicBool, Ordering},
  }, net::Shutdown,
};

use crate::events::{event::Event, subscription::Subscription, Invokable, Subscribable};

use super::packet_connection::PacketConnection;

pub struct PacketReceiveEvent {
  packet_connection: RefCell<PacketConnection>,
  receive_event: RefCell<Event<Vec<u8>>>,
  started: AtomicBool,
  stop:AtomicBool,
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

    return;
  }

  fn locked_start_check(&self) -> bool {
    return self.set_atomic_bool(&self.started);
  }

  fn locked_stop_check(&self) -> bool {
    return self.set_atomic_bool(&self.stop);
  }

  // returns true if the bool was set to true
  fn set_atomic_bool(&self, value: &AtomicBool) -> bool {
    return value.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok();
  }

  pub fn stop(&self) -> Result<()> {
    if !self.locked_stop_check() {
      return Ok(());
    }
    
    self.packet_connection.borrow().shutdown(Shutdown::Both)?;
    Ok(())
  }

  pub fn subscribe(&mut self, subscriber: Box<dyn Fn(&Vec<u8>) + Send + Sync + 'static>) -> Subscription<Vec<u8>> {
    return self.receive_event.borrow_mut().subscribe(subscriber);
  }

  pub fn try_clone(&self) -> Result<PacketReceiveEvent> {
    Ok(PacketReceiveEvent{
      packet_connection: RefCell::new(self.packet_connection.borrow().try_clone()?),
      receive_event: RefCell::new(Event::new()),
      started: AtomicBool::new(self.started.load(Ordering::SeqCst)),
      stop: AtomicBool::new(self.stop.load(Ordering::SeqCst)),
    })
  }
}
