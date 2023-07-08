pub mod event;
pub mod one_shot_event;
pub mod subscription;

use self::subscription::Subscription;

pub type EventHandler<T> = dyn Fn(&T) + Sync + Send;

pub trait Invokable<T> {
    fn invoke(&mut self, arg: &T);
}

// can only be invoked once. additional invokes have no effect.
pub trait InvokableOnce<T> {
    fn invoke(&mut self, arg: T);
}

pub trait Subscribable<T> {
    fn subscribe(&mut self, event_handler: Box<EventHandler<T>>) -> Subscription<T>;
}
