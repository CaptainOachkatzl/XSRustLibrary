pub trait Connection {
    type ErrorType;

    fn send(&mut self, data: Vec<u8>) -> Result<(), Self::ErrorType>;
    fn receive(&mut self) -> Result<Vec<u8>, Self::ErrorType>;
}
