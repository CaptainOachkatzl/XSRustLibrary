use std::sync::{Arc, Weak};

pub struct Subscription<T> {
  shared_ptr: Option<Arc<dyn Fn(&T) + Sync + Send + 'static>>,
}

impl<T> Subscription<T> {
  pub fn new(shared: Arc<dyn Fn(&T) + Sync + Send + 'static>) -> Subscription<T> {
    Subscription::<T> {
      shared_ptr: Some(shared),
    }
  }

  pub fn unsubscribe(&mut self) {
    self.shared_ptr = None;
  }
}

pub fn create_registered_subscription<T: 'static>(
  subscriber_register: &mut Vec<Weak<dyn Fn(&T) + Sync + Send + 'static>>,
  subscriber: Box<dyn Fn(&T) + Sync + Send + 'static>,
) -> Subscription<T> {
  let ref_subscriber = Arc::new(subscriber);
  let weak_subscriber = Arc::downgrade(&ref_subscriber.clone());
  subscriber_register.push(weak_subscriber);
  return Subscription::new(ref_subscriber);
}
