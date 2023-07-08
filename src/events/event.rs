use crate::{subscription::SubscriptionStorage, EventHandler};

use super::{Invokable, Subscribable, Subscription};

/// calls all subscribed handlers on invoke.
pub struct Event<T> {
    subscribers: SubscriptionStorage<T>,
}

impl<T: 'static> Event<T> {
    pub fn new() -> Self {
        Self {
            subscribers: SubscriptionStorage::new(),
        }
    }
}

impl<T: 'static> Default for Event<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> Invokable<T> for Event<T> {
    fn invoke(&mut self, arg: &T) {
        self.subscribers.inner_mut().retain(|subscriber| match subscriber.upgrade() {
            Some(v) => {
                v(arg);
                true
            }
            None => false,
        });
    }
}

impl<T: 'static> Subscribable<T> for Event<T> {
    fn subscribe<'r>(&mut self, event_handler: Box<EventHandler<T>>) -> Subscription<T> {
        self.subscribers.add_event_handler(event_handler)
    }
}
