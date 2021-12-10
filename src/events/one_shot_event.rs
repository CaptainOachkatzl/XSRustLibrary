use std::rc::{Rc, Weak};

use super::{Invokable, Subscribable, Subscription};

/// calls all listeners on invoke. all new listeners after the
/// first invoke will get called immediately with args from the first invoke.
/// not threadsafe.
pub struct OneShotEvent<T> {
    _listeners: Vec<Weak<dyn Fn(&T)>>,
    _args: Option<T>,
}

impl<T> OneShotEvent<T> {
    pub fn new() -> OneShotEvent<T> {
        OneShotEvent::<T> {
            _listeners: Vec::new(),
            _args: Option::None,
        }
    }
}

impl<T> Invokable<T> for OneShotEvent<T> {
    fn invoke(&mut self, arg: T) {
        if self._args.is_some() {
            return;
        }

        for listener in &self._listeners {
            match listener.upgrade() {
                Some(v) => v(&arg),
                None => (),
            }
        }

        self._listeners.clear();

        self._args = Some(arg);
    }
}

impl<T> Subscribable<T> for OneShotEvent<T> {
    fn subscribe(&mut self, listener: impl Fn(&T) + 'static) -> Subscription<T> {
        match &self._args {
            Some(v) => {
                listener(&v);
                return Subscription::new(Rc::new(listener));
            }
            None => {
                let ref_listener = Rc::new(listener);
                let weak_listener = Rc::downgrade(&ref_listener.clone());
                self._listeners.push(weak_listener);
                return Subscription::new(ref_listener);
            }
        }
    }
}
