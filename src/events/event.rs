use std::sync::Weak;

use super::{subscription::create_registered_subscription, Invokable, Subscribable, Subscription};

/// calls all subscribed handlers on invoke.
pub struct Event<T> {
  _subscribers: Vec<Weak<dyn Fn(&T) + Sync + Send + 'static>>,
}

impl<T> Event<T> {
  pub fn new() -> Self {
    Self {
      _subscribers: Vec::new(),
    }
  }
}

impl<T> Invokable<T> for Event<T> {
  fn invoke(&mut self, arg: &T) {
    self
      ._subscribers
      .retain(|subscriber| match subscriber.upgrade() {
        Some(v) => {
          v(&arg);
          return true;
        }
        None => return false,
      });
  }
}

impl<T: 'static> Subscribable<T> for Event<T> {
  fn subscribe<'r>(&mut self, subscriber: Box<dyn Fn(&T) + Sync + Send + 'static>) -> Subscription<T> {
    return create_registered_subscription(&mut self._subscribers, subscriber);
  }
}
