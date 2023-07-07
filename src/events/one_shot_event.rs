use std::sync::Arc;

use crate::subscription::SubscriptionStorage;

use super::{InvokableOnce, Subscribable, Subscription};

/// calls all subscribers on invoke. all new subscribers after the
/// first invoke will get called immediately with args from the first invoke.
/// not threadsafe.
pub struct OneShotEvent<T> {
    subscribers: SubscriptionStorage<T>,
    args: Option<T>,
}

impl<T: 'static> OneShotEvent<T> {
    pub fn new() -> OneShotEvent<T> {
        OneShotEvent::<T> {
            subscribers: SubscriptionStorage::new(),
            args: Option::None,
        }
    }
}

impl<T: 'static> InvokableOnce<T> for OneShotEvent<T> {
    fn invoke(&mut self, arg: T) {
        if self.args.is_some() {
            return;
        }

        for subscriber in self.subscribers.inner_mut() {
            match subscriber.upgrade() {
                Some(v) => v(&arg),
                None => (),
            }
        }

        self.subscribers.inner_mut().clear();

        self.args = Some(arg);
    }
}

impl<T: 'static> Subscribable<T> for OneShotEvent<T> {
    fn subscribe(&mut self, event_handler: Box<dyn Fn(&T) + Sync + Send + 'static>) -> Subscription<T> {
        match &self.args {
            Some(v) => {
                event_handler(&v);
                return Subscription::new(Arc::new(event_handler));
            }
            None => return self.subscribers.add_event_handler(event_handler),
        }
    }
}
