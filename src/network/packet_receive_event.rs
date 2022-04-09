use std::{
  io::Result,
  cell::RefCell,
  sync::{
    Mutex, Arc,
  }, net::Shutdown,
};

use crate::events::{event::Event, subscription::Subscription, Invokable, Subscribable};

use super::packet_connection::PacketConnection;

pub struct PacketReceiveEvent {
  packet_connection: RefCell<PacketConnection>,
  receive_event: RefCell<Event<Vec<u8>>>,
  started: Arc<Mutex<bool>>,
  stop: Arc<Mutex<bool>>,
}

impl PacketReceiveEvent {
  pub fn new(packet_connection: PacketConnection) -> PacketReceiveEvent {
    PacketReceiveEvent {
      packet_connection: RefCell::new(packet_connection),
      receive_event: RefCell::new(Event::new()),
      started: Arc::new(Mutex::new(false)),
      stop: Arc::new(Mutex::new(false)),
    }
  }

  pub fn start(&self) {
    // guarantee only one thread can ever pass through and execute loop
    if !self.locked_start_check() {
      return;
    }

    while !*self.stop.lock().unwrap() {
      match self.packet_connection.borrow_mut().receive() {
        Ok(v) => self.receive_event.borrow_mut().invoke(&v),
        _ => self.stop().unwrap(),
      };
    }

    return;
  }

  fn locked_start_check(&self) -> bool {
    return self.compare_exchange_set_bool(&self.started);
  }

  fn locked_stop_check(&self) -> bool {
    return self.compare_exchange_set_bool(&self.stop);
  }

  // returns true if the bool was set to true
  fn compare_exchange_set_bool(&self, bool_mutex: &Mutex<bool>) -> bool {
    let mut lock = bool_mutex.lock().unwrap();
    if *lock {
      return false;
    }

    *lock = true;
    return true;
  }

  pub fn stop(&self) -> Result<()> {
    if !self.locked_stop_check() {
      return Ok(());
    }
    
    self.packet_connection.borrow_mut().shutdown(Shutdown::Both)?;
    Ok(())
  }

  pub fn subscribe(&mut self, subscriber: Box<dyn Fn(&Vec<u8>) + Send + Sync + 'static>) -> Subscription<Vec<u8>> {
    return self.receive_event.borrow_mut().subscribe(subscriber);
  }

  pub fn try_clone(&self) -> Result<PacketReceiveEvent> {
    Ok(PacketReceiveEvent{
      packet_connection: RefCell::new(self.packet_connection.borrow().try_clone()?),
      receive_event: RefCell::new(Event::new()),
      started: self.started.clone(),
      stop: self.stop.clone(),
    })
  }
}
