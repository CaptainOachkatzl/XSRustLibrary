use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
};

use xs_rust_library::cryptography::key_exchange_algos::{
    curve25519::Curve25519,
    key_exchange::{self, KeyExchange},
};

#[test]
fn successful_key_exchange() {
    let (sender_local, receiver_remote) = std::sync::mpsc::channel::<Box<[u8]>>();
    let (sender_remote, receiver_local) = std::sync::mpsc::channel::<Box<[u8]>>();

    let join_handle = thread::spawn(move || remote_thread(sender_remote, receiver_remote));

    let send = |data: &[u8]| -> Result<(), key_exchange::Error> {
        sender_local.send(Box::from(data)).unwrap();
        return Ok(());
    };
    let receive = || -> Result<Box<[u8]>, key_exchange::Error> { Ok(receiver_local.recv().unwrap()) };
    Curve25519::handshake_active(send, receive).unwrap();
    join_handle.join().unwrap();
}

fn remote_thread(sender: Sender<Box<[u8]>>, receiver: Receiver<Box<[u8]>>) {
    let send = |data: &[u8]| -> Result<(), key_exchange::Error> {
        sender.send(Box::from(data)).unwrap();
        return Ok(());
    };
    let receive = || -> Result<Box<[u8]>, key_exchange::Error> { Ok(receiver.recv().unwrap()) };
    Curve25519::handshake_passive(send, receive).unwrap();
}

#[test]
fn bad_handshake() {
    let send = |_: &[u8]| Ok(());
    let receive = || -> Result<Box<[u8]>, key_exchange::Error> { Ok(Box::new([])) };
    Curve25519::handshake_active(send, receive).unwrap_err();
}
