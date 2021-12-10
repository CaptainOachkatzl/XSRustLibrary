use std::rc::{Rc, Weak};

use super::{Invokable, Subscribable, Subscription};

/// calls all listeners on invoke. not threadsafe.
pub struct Event<T> {
    _listeners: Vec<Weak<dyn Fn(&T)>>,
}

impl<T> Event<T> {
    pub fn new() -> Self {
        Self {
            _listeners: Vec::new(),
        }
    }
}

impl<T> Invokable<T> for Event<T> {
    fn invoke(&mut self, arg: T) {
        self._listeners.retain(|listener| match listener.upgrade() {
            Some(v) => {
                v(&arg);
                return true;
            }
            None => return false,
        });
    }
}

impl<T> Subscribable<T> for Event<T> {
    fn subscribe(&mut self, listener: impl Fn(&T) + 'static) -> Subscription<T> {
        let ref_listener = Rc::new(listener);
        let weak_listener = Rc::downgrade(&ref_listener.clone());
        self._listeners.push(weak_listener);
        return Subscription::new(ref_listener);
    }
}
