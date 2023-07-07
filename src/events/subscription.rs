use std::sync::{Arc, Weak};

pub struct Subscription<T> {
    shared_ptr: Option<Arc<dyn Fn(&T) + Sync + Send>>,
}

impl<T> Subscription<T> {
    pub fn new(shared: Arc<dyn Fn(&T) + Sync + Send>) -> Subscription<T> {
        Subscription::<T> { shared_ptr: Some(shared) }
    }

    pub fn unsubscribe(&mut self) {
        self.shared_ptr = None;
    }
}

pub struct SubscriptionStorage<T> {
    subscribers: Vec<Weak<dyn Fn(&T) + Sync + Send>>,
}

impl<T: 'static> SubscriptionStorage<T> {
    pub fn new() -> Self {
        Self { subscribers: vec![] }
    }

    pub fn add_event_handler(&mut self, event_handler: Box<dyn Fn(&T) + Sync + Send>) -> Subscription<T> {
        let ref_subscriber = Arc::new(event_handler);
        let weak_subscriber = Arc::downgrade(&ref_subscriber.clone());
        self.subscribers.push(weak_subscriber);
        return Subscription::new(ref_subscriber);
    }

    pub fn inner_mut(&mut self) -> &mut Vec<Weak<dyn Fn(&T) + Sync + Send>> {
        &mut self.subscribers
    }
}
