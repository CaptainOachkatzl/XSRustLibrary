use super::{Invokable, Subscribable};

/// calls all listeners on invoke. all new listeners after the
/// first invoke will get called immediately with args from the first invoke.
/// not threadsafe.
pub struct OneShotEvent<T> {
    _listeners: Vec<Box<dyn Fn(&T)>>,
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
            listener(&arg);
        }

        self._listeners.clear();

        self._args = Some(arg);
    }
}

impl<T> Subscribable<T> for OneShotEvent<T> {
    fn subscribe(&mut self, listener: impl Fn(&T) + 'static) {
        self._listeners.push(Box::new(listener));
    }
}
