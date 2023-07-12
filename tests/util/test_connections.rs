#![allow(unused)]

use std::{
    fmt::Display,
    net::{TcpListener, TcpStream},
    sync::mpsc::{Receiver, Sender},
    thread,
};

use xs_rust_library::{
    connection::Connection,
    encrypted_connection::EncryptedConnection,
    encryption::aes256_crypto::Aes256Crypto,
    key_exchange::{curve25519::Curve25519, HandshakeMode},
    packet_connection::PacketConnection,
};

pub struct ChannelConnection {
    pub sender: Sender<Box<[u8]>>,
    pub receiver: Receiver<Box<[u8]>>,
}

impl Connection for ChannelConnection {
    type ErrorType = String;

    fn send(&mut self, data: &[u8]) -> Result<(), String> {
        Ok(self.sender.send(Box::from(data.clone())).unwrap())
    }

    fn receive(&mut self) -> Result<Vec<u8>, String> {
        Ok(self.receiver.recv().unwrap().into_vec())
    }
}

impl ChannelConnection {
    pub fn new_test_pair() -> (ChannelConnection, ChannelConnection) {
        let (sender_local, receiver_remote) = std::sync::mpsc::channel::<Box<[u8]>>();
        let (sender_remote, receiver_local) = std::sync::mpsc::channel::<Box<[u8]>>();

        let con_local = ChannelConnection {
            sender: sender_local,
            receiver: receiver_local,
        };

        let con_remote = ChannelConnection {
            sender: sender_remote,
            receiver: receiver_remote,
        };

        (con_local, con_remote)
    }
}

pub struct FaultyConnection;

impl Connection for FaultyConnection {
    type ErrorType = String;

    fn send(&mut self, _data: &[u8]) -> Result<(), String> {
        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>, String> {
        Ok(Vec::new())
    }
}

pub fn new_packet_connection_test_pair() -> (PacketConnection, PacketConnection) {
    let listener = TcpListener::bind("127.0.0.1:1234").unwrap();

    let join_handle = thread::spawn(move || {
        let remote_stream = TcpStream::connect("127.0.0.1:1234").unwrap();
        PacketConnection::new(remote_stream, 1024)
    });

    let (local_stream, _) = listener.accept().unwrap();
    let local_con = PacketConnection::new(local_stream, 1024);

    let remote_con = join_handle.join().unwrap();
    (local_con, remote_con)
}

pub fn new_aes_encrypted_connection_test_pair<E: Display, Con: Connection<ErrorType = E> + Send + Sync + 'static>(
    local: Con,
    remote: Con,
) -> (EncryptedConnection<Aes256Crypto, Con>, EncryptedConnection<Aes256Crypto, Con>) {
    let join_handle = thread::spawn(move || EncryptedConnection::with_handshake(remote, Curve25519, HandshakeMode::Client).unwrap());

    let local_con = EncryptedConnection::with_handshake(local, Curve25519, HandshakeMode::Server).unwrap();

    let remote_con = join_handle.join().unwrap();
    (local_con, remote_con)
}
