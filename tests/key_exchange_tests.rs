use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
};

use xs_rust_library::{
    connection::Connection,
    cryptography::key_exchange::{curve25519::Curve25519, HandshakeMode, KeyExchange},
};

struct ChannelConnection {
    sender: Sender<Box<[u8]>>,
    receiver: Receiver<Box<[u8]>>,
}

impl Connection<String> for ChannelConnection {
    fn send(&mut self, data: &[u8]) -> Result<(), String> {
        Ok(self.sender.send(Box::from(data.clone())).unwrap())
    }

    fn receive(&mut self) -> Result<Vec<u8>, String> {
        Ok(self.receiver.recv().unwrap().into_vec())
    }
}

#[test]
fn successful_key_exchange() {
    let (sender_local, receiver_remote) = std::sync::mpsc::channel::<Box<[u8]>>();
    let (sender_remote, receiver_local) = std::sync::mpsc::channel::<Box<[u8]>>();

    let mut con_local = ChannelConnection {
        sender: sender_local,
        receiver: receiver_local,
    };

    let con_remote = ChannelConnection {
        sender: sender_remote,
        receiver: receiver_remote,
    };

    let join_handle = thread::spawn(move || remote_thread(con_remote));

    Curve25519::handshake(&mut con_local, HandshakeMode::Server).unwrap();
    join_handle.join().unwrap();
}

fn remote_thread(mut connection: ChannelConnection) {
    Curve25519::handshake(&mut connection, HandshakeMode::Client).unwrap();
}

struct FaultyConnection;

impl Connection<String> for FaultyConnection {
    fn send(&mut self, _data: &[u8]) -> Result<(), String> {
        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>, String> {
        Ok(Vec::new())
    }
}

#[test]
fn bad_handshake() {
    Curve25519::handshake(&mut FaultyConnection, HandshakeMode::Client).unwrap_err();
}
