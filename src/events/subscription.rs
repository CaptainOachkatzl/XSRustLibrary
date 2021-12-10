use std::rc::Rc;

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
