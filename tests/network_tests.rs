#[cfg(test)]
mod network_tests {
    use std::{net::*, sync::*, thread};

    use xs_rust_library::{
        connection::Connection,
        cryptography::{
            encryption::aes256_crypto::Aes256Crypto,
            key_exchange::{curve25519::Curve25519, HandshakeMode},
        },
        encrypted_connection::EncryptedConnection,
        packet_connection::PacketConnection,
        packet_receive_event::PacketReceiveEvent,
    };

    #[test]
    fn connect_test() {
        // weird way to implement a signal, maybe replace with better fit
        let listening_barrier: Arc<Barrier> = Arc::new(Barrier::new(2));
        let listening_copy = listening_barrier.clone();

        let listener_thread = thread::spawn(move || {
            start_listening(listening_copy);
        });

        listening_barrier.wait();

        assert!(connect_to_localhost().is_ok());

        listener_thread.join().unwrap();
    }

    fn start_listening(listening: Arc<Barrier>) {
        let listener = TcpListener::bind("0.0.0.0:1234").unwrap();

        listening.wait();

        let accept_stream: TcpStream = listener.accept().unwrap().0;
        let mut accept_connection = PacketConnection::new(accept_stream, 1024);
        accept_connection.send(b"test123").unwrap();
        accept_connection.send(b"abc").unwrap();
        accept_connection.send(&[5 as u8; 10 * 1024 * 1024]).unwrap();
    }

    fn connect_to_localhost() -> std::io::Result<()> {
        let stream = TcpStream::connect("127.0.0.1:1234")?;
        let mut connection = PacketConnection::new(stream, 1024);
        assert_eq!(connection.receive().unwrap().len(), 7);
        assert_eq!(connection.receive().unwrap().len(), 3);
        let big_data = connection.receive().unwrap();
        assert_eq!(big_data.len(), 10 * 1024 * 1024);
        assert_eq!(big_data, [5 as u8; 10 * 1024 * 1024]);
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

        let shutdown_stream = stream.try_clone().unwrap();

        let receive_thread = thread::spawn(move || {
            let receive_callback = Box::new(move |_data: &Vec<u8>| {
                let mut guard = counter.lock().unwrap();
                *guard += 1;
                listening_barrier.wait();
            });

            let connection = PacketConnection::new(stream, 1024);
            let mut event = PacketReceiveEvent::new(connection);

            let _sub = event.subscribe(receive_callback);
            event.start();
        });

        let mut accept_stream = PacketConnection::new(listener.accept().unwrap().0, 1024);
        accept_stream.send(&[0 as u8; 4]).unwrap();

        listening_barrier2.wait();

        shutdown_stream.shutdown(Shutdown::Both).unwrap();

        // wait for shutdown and acknowledge it -> prevents timeout of receive thread
        assert!(accept_stream.receive().is_err());

        receive_thread.join().unwrap();
        assert_eq!(*counter2.lock().unwrap(), 1);
    }

    #[test]
    fn shutdown_test() {
        let _listener = TcpListener::bind("0.0.0.0:4567").unwrap();
        let stream = TcpStream::connect("127.0.0.1:4567").unwrap();
        let stream2 = stream.try_clone().unwrap();

        let connection = PacketConnection::new(stream, 1024);

        let receive_thread = thread::spawn(move || {
            let mut connection = PacketConnection::new(stream2, 1024);
            assert!(connection.receive().is_err());
        });

        connection.shutdown(Shutdown::Both).unwrap();
        receive_thread.join().unwrap();
    }

    #[test]
    fn encrypted_connection() {
        let listener = TcpListener::bind("127.0.0.1:5678").unwrap();

        let join_handle = thread::spawn(move || {
            let remote_stream = TcpStream::connect("127.0.0.1:5678").unwrap();
            let remote_con = PacketConnection::new(remote_stream, 1024);
            let mut enc_con =
                EncryptedConnection::<Aes256Crypto, _, _>::with_handshake(remote_con, Curve25519, HandshakeMode::Client).unwrap();
            enc_con.send(b"top secret").unwrap();
        });

        let (local_stream, _) = listener.accept().unwrap();
        let local_con = PacketConnection::new(local_stream, 1024);
        let mut enc_con = EncryptedConnection::<Aes256Crypto, _, _>::with_handshake(local_con, Curve25519, HandshakeMode::Client).unwrap();
        assert_eq!(b"top secret".as_slice(), &enc_con.receive().unwrap());

        join_handle.join().unwrap();
    }
}
