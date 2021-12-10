use std::rc::Rc;

pub mod event;
pub mod one_shot_event;

pub trait Invokable<T> {
    fn invoke(&mut self, arg: T);
}

pub trait Subscribable<T> {
    fn subscribe(&mut self, listener: impl Fn(&T) + 'static) -> Subscription<T>;
}

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
