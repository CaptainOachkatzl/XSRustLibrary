use super::{Invokable, Subscribable};

/// calls all listeners on invoke. not threadsafe.
pub struct Event<T> {
    _listeners: Vec<Box<dyn Fn(&T)>>,
}

impl<T> Event<T> {
    pub fn new() -> Self {
        Self {
            _listeners: Vec::new(),
        }
    }
}

impl<T> Invokable<T> for Event<T> {
    fn invoke(&mut self, value: T) {
        for listener in &self._listeners {
            listener(&value);
        }
    }
}

impl<T> Subscribable<T> for Event<T> {
    fn subscribe(&mut self, listener: impl Fn(&T) + 'static) {
        self._listeners.push(Box::new(listener));
    }
}
