mod util;

use std::thread;

use util::test_connections::{ChannelConnection, FaultyConnection};
use xs_rust_library::cryptography::key_exchange::{curve25519::Curve25519, HandshakeMode, KeyExchange};

#[test]
fn successful_key_exchange() {
    let (mut con_local, mut con_remote) = ChannelConnection::new_test_pair();

    let join_handle = thread::spawn(move || {
        Curve25519.handshake(&mut con_remote, HandshakeMode::Client).unwrap();
    });

    Curve25519.handshake(&mut con_local, HandshakeMode::Server).unwrap();
    join_handle.join().unwrap();
}

#[test]
fn bad_handshake() {
    Curve25519.handshake(&mut FaultyConnection, HandshakeMode::Client).unwrap_err();
}
