use std::sync::mpsc::{Receiver, Sender};

use xs_rust_library::connection::Connection;

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
