use std::rc::{Rc, Weak};

use super::{Invokable, Subscribable, Subscription};

/// calls all subscribers on invoke. all new subscribers after the
/// first invoke will get called immediately with args from the first invoke.
/// not threadsafe.
pub struct OneShotEvent<T> {
  _subscribers: Vec<Weak<dyn Fn(&T)>>,
  _args: Option<T>,
}

impl<T> OneShotEvent<T> {
  pub fn new() -> OneShotEvent<T> {
    OneShotEvent::<T> {
      _subscribers: Vec::new(),
      _args: Option::None,
    }
  }
}

impl<T> Invokable<T> for OneShotEvent<T> {
  fn invoke(&mut self, arg: T) {
    if self._args.is_some() {
      return;
    }

    for subscriber in &self._subscribers {
      match subscriber.upgrade() {
        Some(v) => v(&arg),
        None => (),
      }
    }

    self._subscribers.clear();

    self._args = Some(arg);
  }
}

impl<T: 'static> Subscribable<T> for OneShotEvent<T> {
  fn subscribe(&mut self, subscriber: Box<dyn Fn(&T)>) -> Subscription<T> {
    match &self._args {
      Some(v) => {
        subscriber(&v);
        return Subscription::new(Rc::new(subscriber));
      }
      None => {
        let ref_subscriber = Rc::new(subscriber);
        let weak_subscriber = Rc::downgrade(&ref_subscriber.clone());
        self._subscribers.push(weak_subscriber);
        return Subscription::new(ref_subscriber);
      }
    }
  }
}
