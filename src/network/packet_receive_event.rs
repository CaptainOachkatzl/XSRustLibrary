use crate::events::{event::Event, subscription::Subscription, Subscribable, Invokable};

use super::packet_connection::PacketConnection;

pub struct PacketReceiveEvent {
  packet_connection: PacketConnection,
  receive_event: Event<Vec<u8>>,
  started: bool,
}

impl PacketReceiveEvent {

  pub fn new(packet_connection: PacketConnection) -> PacketReceiveEvent {
    PacketReceiveEvent { packet_connection, receive_event: Event::new(), started: false }
  }

  pub fn start(&mut self) {
    if self.started {
      return;
    }

    self.started = true;

    loop {
      self.receive_event.invoke(&self.packet_connection.receive().unwrap());
    }
  }

  pub fn subscribe(&mut self, subscriber: fn(&Vec<u8>)) -> Subscription<Vec<u8>> {
    return self.receive_event.subscribe(subscriber);
  }
}