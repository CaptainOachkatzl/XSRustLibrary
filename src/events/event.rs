use std::rc::{Rc, Weak};

use super::{Invokable, Subscribable, Subscription};

/// calls all subscribers on invoke. not threadsafe.
pub struct Event<T> {
  _subscribers: Vec<Weak<dyn Fn(&T)>>,
}

impl<T> Event<T> {
  pub fn new() -> Self {
    Self {
      _subscribers: Vec::new(),
    }
  }
}

impl<T> Invokable<T> for Event<T> {
  fn invoke(&mut self, arg: T) {
    self._subscribers.retain(|subscriber| match subscriber.upgrade() {
      Some(v) => {
        v(&arg);
        return true;
      }
      None => return false,
    });
  }
}

impl<T: 'static> Subscribable<T> for Event<T> {
  fn subscribe<'r>(&mut self, subscriber: Box<dyn Fn(&T)>) -> Subscription<T> {
    let ref_subscriber = Rc::new(subscriber);
    let weak_subscriber = Rc::downgrade(&ref_subscriber.clone());
    self._subscribers.push(weak_subscriber);
    return Subscription::new(ref_subscriber);
  }
}
