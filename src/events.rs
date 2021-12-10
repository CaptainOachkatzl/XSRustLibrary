pub mod event;
pub mod one_shot_event;

pub trait Invokable<T> {
    fn invoke(&mut self, arg: T);
}

pub trait Subscribable<T> {
    fn subscribe(&mut self, listener: impl Fn(&T) + 'static);
}
