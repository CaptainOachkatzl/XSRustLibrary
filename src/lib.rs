pub mod events;
pub mod network;

#[cfg(test)]
mod tests {

  use std::io::Result;
  use std::net::{TcpListener, TcpStream, Shutdown};
  use std::sync::{Arc, Barrier};
  use std::{cell::RefCell, rc::Rc, thread};

  use crate::events::{event::Event, one_shot_event::OneShotEvent, Invokable, Subscribable};
  use crate::network::packet_connection::PacketConnection;

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
    // weird way to implement a signal, maybe replace with better fit
    let listening_barrier: Arc<Barrier> = Arc::new(Barrier::new(2));
    let listening_copy = listening_barrier.clone();

    let listener_thread = thread::spawn(move || {
      start_listening(listening_copy).unwrap();
    });

    listening_barrier.wait();

    match connect_to_localhost() {
      Ok(()) => assert!(true),
      Err(_) => assert!(false),
    }

    listener_thread.join().unwrap();
  }

  fn start_listening(listening: Arc<Barrier>) -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:1234").unwrap();

    listening.wait();

    let accept_stream: TcpStream = listener.accept().unwrap().0;
    let mut accept_connection = PacketConnection::new(accept_stream, 1024);
    accept_connection.send(b"test123")?;
    accept_connection.send(b"abc")?;
    accept_connection.send(&[5 as u8; 10*1024*1024])?;
    Ok(())
  }

  fn connect_to_localhost() -> std::io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:1234")?;
    let mut connection = PacketConnection::new(stream, 1024);
    assert_eq!(connection.receive().unwrap().len(), 7);
    assert_eq!(connection.receive().unwrap().len(), 3);
    let big_data = connection.receive().unwrap();
    assert_eq!(big_data.len(), 10*1024*1024);
    assert_eq!(big_data, [5 as u8; 10*1024*1024]);
    Ok(())
  }

  #[test]
  fn multithread_connection_access() {

    let _listener = TcpListener::bind("0.0.0.0:2345").unwrap();
    let stream = TcpStream::connect("127.0.0.1:2345").unwrap();
    let stream_copy = stream.try_clone().unwrap();
    let receive_thread = thread::spawn(move || {
      dummy_send(stream_copy);
    });

    dummy_send(stream);
    receive_thread.join().unwrap();
  }

  fn dummy_send(stream: TcpStream) {
    let mut packet_connection = PacketConnection::new(stream, 1024);
    packet_connection.send(&[0 as u8; 8]).expect("sending failed");
  }

  #[test]
  fn shutdown_test() {
    let _listener = TcpListener::bind("0.0.0.0:4567").unwrap();
    let stream = TcpStream::connect("127.0.0.1:4567").unwrap();

    let connection = PacketConnection::new(stream, 1024);
    let mut connection2 = connection.try_clone().unwrap();

    let receive_thread = thread::spawn(move || {
      assert!(connection2.receive().is_err());
    });

    connection.shutdown(Shutdown::Both).unwrap();
    receive_thread.join().unwrap();
  }
}
