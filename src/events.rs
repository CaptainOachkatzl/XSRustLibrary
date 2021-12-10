pub mod event;
pub mod one_shot_event;
pub mod subscription;

use self::subscription::Subscription;

pub trait Invokable<T> {
  fn invoke(&mut self, arg: T);
}

pub trait Subscribable<T> {
  fn subscribe(&mut self, listener: Box<dyn Fn(&T)>) -> Subscription<T>;
}
