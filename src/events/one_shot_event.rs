use std::sync::{Arc, Weak};

use super::{subscription::create_registered_subscription, Subscribable, Subscription, InvokableOnce};

/// calls all subscribers on invoke. all new subscribers after the
/// first invoke will get called immediately with args from the first invoke.
/// not threadsafe.
pub struct OneShotEvent<T> {
  _subscribers: Vec<Weak<fn(&T)>>,
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

impl<T> InvokableOnce<T> for OneShotEvent<T> {
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
  fn subscribe(&mut self, subscriber: fn(&T)) -> Subscription<T> {
    match &self._args {
      Some(v) => {
        subscriber(&v);
        return Subscription::new(Arc::new(subscriber));
      }
      None => return create_registered_subscription(&mut self._subscribers, subscriber),
    }
  }
}
