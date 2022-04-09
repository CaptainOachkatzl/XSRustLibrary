pub mod network;
pub mod events;

#[cfg(test)]
mod tests {

use std::io::Result;
  use std::net::{TcpListener, TcpStream, Shutdown};
  use std::sync::{Arc, Barrier, Mutex};
use std::{cell::RefCell, rc::Rc, thread};

  use crate::events::InvokableOnce;
use crate::events::{event::Event, one_shot_event::OneShotEvent, Invokable, Subscribable};
  use crate::network::packet_connection::PacketConnection;
  use crate::network::packet_receive_event::PacketReceiveEvent;

  #[test]
  fn event_test() {
    let mut event = Event::<Arc<Mutex<i32>>>::new();
    let counter = Arc::new(Mutex::new(0));
    let counter2 = counter.clone();

    let handler = |i: &Arc<Mutex<i32>>| {
      *i.lock().unwrap() += 1;
    };

    let _subscription = event.subscribe(Box::from(handler));

    let thread = thread::spawn(move || {
      event.invoke(&counter2);
    });

    thread.join().unwrap();
    assert_eq!(*counter.lock().unwrap(), 1);
  }

  #[test]
  fn one_shot_test() {
    let mut event = OneShotEvent::<Rc<RefCell<i32>>>::new();
    let counter = Rc::new(RefCell::new(0));

    let callback = Box::new(|x: &Rc<RefCell<i32>>| {
      *x.borrow_mut() += 1;
    });

    let _sub = event.subscribe(callback.clone());
    event.invoke(counter.clone());
    let _sub = event.subscribe(callback);

    assert_eq!(*counter.as_ref().borrow(), 2);
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

    assert!(connect_to_localhost().is_ok());

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
  fn receive_event_test() {

    let listening_barrier: Arc<Barrier> = Arc::new(Barrier::new(2));
    let listening_barrier2 = listening_barrier.clone();

    let listener = TcpListener::bind("0.0.0.0:3456").unwrap();
    let stream = TcpStream::connect("127.0.0.1:3456").unwrap();

    let counter = Arc::new(Mutex::new(0));
    let counter2 = counter.clone();

    let connection = PacketConnection::new(stream, 1024);
    let mut event = PacketReceiveEvent::new(connection);
    let event2 = event.try_clone().unwrap();

    let receive_thread = thread::spawn(move || {

      let receive_callback = Box::new(move |_data: &Vec<u8>| {
        let mut guard = counter.lock().unwrap();
        *guard += 1;
        listening_barrier.wait();
      });
      
      let _sub = event.subscribe(receive_callback);
      event.start();
    });

    let mut accept_stream = PacketConnection::new(listener.accept().unwrap().0, 1024);
    accept_stream.send(&[0 as u8; 4]).unwrap();

    listening_barrier2.wait();
    
    event2.stop().unwrap();

    // wait for shutdown and acknowledge it -> prevents timeout of receive thread
    assert!(accept_stream.receive().is_err());
    
    receive_thread.join().unwrap();
    assert_eq!(*counter2.lock().unwrap(), 1);
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
