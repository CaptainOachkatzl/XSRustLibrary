use std::fmt::Display;

pub trait Connection<E>
where
    E: Display,
{
    fn send(&mut self, data: &[u8]) -> Result<(), E>;
    fn receive(&mut self) -> Result<Vec<u8>, E>;
}
