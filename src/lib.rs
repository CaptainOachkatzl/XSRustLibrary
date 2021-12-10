pub mod events;

#[cfg(test)]
mod tests {
  use std::{cell::RefCell, rc::Rc};

  use crate::events::{event::Event, one_shot_event::OneShotEvent, Invokable, Subscribable};

  #[test]
  fn event_test() {
    let counter = Rc::new(RefCell::new(0));
    let counter_result = counter.clone();
    let mut event = Event::<i32>::new();

    let callback = move |x: &i32| {
      let mut counter_value = counter.borrow_mut();
      *counter_value += 1;
      assert_eq!(*x, 3);
    };

    let _sub = event.subscribe(Box::new(callback.clone()));
    event.invoke(3);
    let _sub = event.subscribe(Box::new(callback));

    assert_eq!(*counter_result.borrow(), 1);
  }

  #[test]
  fn one_shot_test() {
    let counter = Rc::new(RefCell::new(0));
    let counter_result = counter.clone();
    let mut event = OneShotEvent::<i32>::new();

    let callback = move |x: &i32| {
      let mut counter_value = counter.borrow_mut();
      *counter_value += 1;
      assert_eq!(*x, 3);
    };

    let _sub = event.subscribe(Box::new(callback.clone()));
    event.invoke(3);
    let _sub = event.subscribe(Box::new(callback));

    assert_eq!(*counter_result.borrow(), 2);
  }
}
