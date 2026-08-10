use std::net::Shutdown;

pub trait Connection {
    type ErrorType;

    fn send(&mut self, data: &[u8]) -> Result<(), Self::ErrorType>;
    fn receive(&mut self) -> Result<Vec<u8>, Self::ErrorType>;
    fn shutdown(&self, how: Shutdown) -> Result<(), Self::ErrorType>;
    fn try_clone(&self) -> Result<Self, Self::ErrorType>
    where
        Self: Sized;
}
