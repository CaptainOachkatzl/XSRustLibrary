use std::sync::Arc;

use crate::{subscription::SubscriptionStorage, EventHandler};

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

impl<T: 'static> Default for OneShotEvent<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> InvokableOnce<T> for OneShotEvent<T> {
    fn invoke(&mut self, arg: T) {
        if self.args.is_some() {
            return;
        }

        for subscriber in self.subscribers.inner_mut() {
            if let Some(event_handler) = subscriber.upgrade() {
                event_handler(&arg);
            }
        }

        self.subscribers.inner_mut().clear();

        self.args = Some(arg);
    }
}

impl<T: 'static> Subscribable<T> for OneShotEvent<T> {
    fn subscribe(&mut self, event_handler: Box<EventHandler<T>>) -> Subscription<T> {
        match &self.args {
            Some(v) => {
                event_handler(v);
                Subscription::new(Arc::new(event_handler))
            }
            None => self.subscribers.add_event_handler(event_handler),
        }
    }
}
