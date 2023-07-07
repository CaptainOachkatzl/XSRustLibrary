use std::{cell::RefCell, rc::Rc, sync::*, thread};

use xs_rust_library::{event::*, one_shot_event::OneShotEvent, *};

#[test]
fn event_test() {
    let mut event = Event::<Arc<Mutex<i32>>>::new();
    let counter = Arc::new(Mutex::new(0));
    let counter2 = counter.clone();

    let handler = |i: &Arc<Mutex<i32>>| {
        *i.lock().unwrap() += 1;
    };

    let _subscription = event.subscribe(Box::from(handler));

    let thread = thread::spawn(move || {
        event.invoke(&counter2);
    });

    thread.join().unwrap();
    assert_eq!(*counter.lock().unwrap(), 1);
}

#[test]
fn one_shot_test() {
    let mut event = OneShotEvent::<Rc<RefCell<i32>>>::new();
    let counter = Rc::new(RefCell::new(0));

    let callback = Box::new(|x: &Rc<RefCell<i32>>| {
        *x.borrow_mut() += 1;
    });

    let _sub = event.subscribe(callback.clone());
    event.invoke(counter.clone());
    let _sub = event.subscribe(callback);

    assert_eq!(*counter.as_ref().borrow(), 2);
}
