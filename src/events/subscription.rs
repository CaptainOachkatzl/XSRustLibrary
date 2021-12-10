use std::rc::{Rc, Weak};

pub struct Subscription<T> {
  shared_ptr: Option<Rc<dyn Fn(&T)>>,
}

impl<T> Subscription<T> {
  pub fn new(shared: Rc<dyn Fn(&T)>) -> Subscription<T> {
    Subscription::<T> {
      shared_ptr: Some(shared),
    }
  }

  pub fn unsubscribe(&mut self) {
    self.shared_ptr = None;
  }
}

pub fn create_registered_subscription<T: 'static>(
  subscriber_register: &mut Vec<Weak<dyn Fn(&T)>>,
  subscriber: Box<dyn Fn(&T)>,
) -> Subscription<T> {
  let ref_subscriber = Rc::new(subscriber);
  let weak_subscriber = Rc::downgrade(&ref_subscriber.clone());
  subscriber_register.push(weak_subscriber);
  return Subscription::new(ref_subscriber);
}
