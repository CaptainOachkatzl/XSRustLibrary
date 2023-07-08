use std::sync::{Arc, Weak};

use crate::EventHandler;

pub struct Subscription<T> {
    shared_ptr: Option<Arc<EventHandler<T>>>,
}

impl<T> Subscription<T> {
    pub fn new(shared: Arc<EventHandler<T>>) -> Subscription<T> {
        Subscription::<T> { shared_ptr: Some(shared) }
    }

    pub fn unsubscribe(&mut self) {
        self.shared_ptr = None;
    }
}

pub struct SubscriptionStorage<T> {
    subscribers: Vec<Weak<EventHandler<T>>>,
}

impl<T: 'static> Default for SubscriptionStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> SubscriptionStorage<T> {
    pub fn new() -> Self {
        Self { subscribers: vec![] }
    }

    pub fn add_event_handler(&mut self, event_handler: Box<EventHandler<T>>) -> Subscription<T> {
        let ref_subscriber = Arc::new(event_handler);
        let weak_subscriber = Arc::downgrade(&ref_subscriber.clone());
        self.subscribers.push(weak_subscriber);
        Subscription::new(ref_subscriber)
    }

    pub fn inner_mut(&mut self) -> &mut Vec<Weak<EventHandler<T>>> {
        &mut self.subscribers
    }
}
