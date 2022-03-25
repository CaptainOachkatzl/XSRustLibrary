pub mod events;
pub mod network;

#[cfg(test)]
mod tests {
  
use std::io::Result;
use std::time::Duration;
use std::{cell::RefCell, rc::Rc, thread};
  use std::net::{TcpListener, TcpStream};

  use crate::events::{event::Event, one_shot_event::OneShotEvent, Invokable, Subscribable};
use crate::network::tcp_packet_connection::TcpPacketConnection;

  #[test]
  fn event_test() {
    let counter = Rc::new(RefCell::new(0));
    let counter_result = counter.clone();
    let mut event = Event::<i32>::new();

    let callback = move |x: &i32| {
      let mut counter_value = counter.borrow_mut();
      *counter_value += 1;
      assert_eq!(*x, 3);
    };

    let _sub = event.subscribe(Box::new(callback.clone()));
    event.invoke(3);
    let _sub = event.subscribe(Box::new(callback));

    assert_eq!(*counter_result.borrow(), 1);
  }

  #[test]
  fn one_shot_test() {
    let counter = Rc::new(RefCell::new(0));
    let counter_result = counter.clone();
    let mut event = OneShotEvent::<i32>::new();

    let callback = move |x: &i32| {
      let mut counter_value = counter.borrow_mut();
      *counter_value += 1;
      assert_eq!(*x, 3);
    };

    let _sub = event.subscribe(Box::new(callback.clone()));
    event.invoke(3);
    let _sub = event.subscribe(Box::new(callback));

    assert_eq!(*counter_result.borrow(), 2);
  }

  #[test]
  fn connect_test() { 
    thread::spawn(|| {
      start_listening().unwrap();
    });

    thread::sleep(Duration::from_secs(2));

    match connect_to_localhost() {
      Ok(()) => assert!(true),
      Err(_) => assert!(false),
    }
  }

  fn start_listening() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:1234").unwrap();
    let accept_stream: TcpStream = listener.accept().unwrap().0;

    let mut accept_connection = TcpPacketConnection::new(accept_stream);
    accept_connection.send(b"test123")?;
    Ok(())
  }

  fn connect_to_localhost() -> std::io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:1234")?;
    let mut connection = TcpPacketConnection::new(stream);
    assert_eq!(connection.receive().unwrap().len(), 7);
    Ok(())
  }
}
