#![allow(unused)]

use std::{fmt::Debug, time::Instant};

use xs_rust_library::connection::Connection;

pub fn measure_connection_throughput<E: Debug, Con: Connection<ErrorType = E>>(local_con: &mut Con, remote_con: &mut Con, test_name: &str) {
    for &load in TEST_LOADS {
        let data = load.to_vec();
        let start = Instant::now();
        local_con.send(data).unwrap();
        remote_con.receive().unwrap();
        let duration = start.elapsed();
        let load_size = load.len();
        let bytes_per_sec = load.len() as f64 / duration.as_secs_f64();
        println!("{test_name}: {bytes_per_sec} B/s, size = {load_size}, duration = {duration:?}")
    }
}

static TEST_LOADS: &[&[u8]] = &[&[0_u8; 1024], &[5_u8; 1024 * 1024], &[10_u8; 100 * 1024 * 1024]];
