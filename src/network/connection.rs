use std::net::Shutdown;

pub trait Connection {
    type ErrorType;

    fn send(&mut self, data: &[u8]) -> Result<(), Self::ErrorType>;
    fn receive(&mut self) -> Result<Vec<u8>, Self::ErrorType> {
        let mut buffer = Vec::new();
        self.receive_into(&mut buffer)?;
        Ok(buffer)
    }

    /// receive data directly into the provided buffer, reusing its allocation
    /// to avoid allocating a new buffer on every receive.
    fn receive_into(&mut self, buffer: &mut Vec<u8>) -> Result<usize, Self::ErrorType>;

    fn shutdown(&self, how: Shutdown) -> Result<(), Self::ErrorType>;
    fn try_clone(&self) -> Result<Self, Self::ErrorType>
    where
        Self: Sized;
}
