use std::{collections::BTreeMap, sync::{Barrier, Arc}, thread};

use xs_bevy_core_2d::{Counter, FactoryCache};

#[test]
fn validate_results() {
  const TEST_CYCLES: i64 = 100;
  let cache = FactoryCache::new(BTreeMap::new(), Box::new(generate_square));

  // check original results
  for i in 0..TEST_CYCLES {
    assert!(*cache.get(i) == generate_square(i));
  }

  // check cached results
  for i in 0..TEST_CYCLES {
    assert!(*cache.get(i) == generate_square(i));
  }
}

fn generate_square(x: i64) -> i64 {
  x * x
}

#[test]
fn confirm_cached() {
  static COUNTER: Counter = Counter::new();

  let call_counter = |_: usize| {
    COUNTER.tick();
    return 1;
  };

  let cache = FactoryCache::new(BTreeMap::new(), Box::new(call_counter));

  assert_eq!(COUNTER.count(), 0);
  cache.get(1);
  assert_eq!(COUNTER.count(), 1);
  cache.get(1);
  assert_eq!(COUNTER.count(), 1);
}

#[test]
fn confirm_stable_cache() {
  let factory_cache = FactoryCache::new(BTreeMap::new(), Box::new(generate_square));
  let x: std::sync::Arc<i64> = factory_cache.get(42);
  for i in 0..10 {
    factory_cache.get(i);
  }
  assert_eq!(*x, generate_square(42))
}

#[test]
fn parallel_test() {
  const THREAD_COUNT: usize = 100;

  let factory_cache = Arc::new(FactoryCache::new(BTreeMap::new(), Box::new(generate_square)));

  let barrier = Arc::new(Barrier::new(THREAD_COUNT));
  let mut thread_handles = vec!();
  for i in 0..THREAD_COUNT {
    let thread_id = i;
    let bar = Arc::clone(&barrier);
    let factory = Arc::clone(&factory_cache);
    thread_handles.push(thread::spawn(move || {
      bar.wait();
      execute_workload(thread_id, factory);
    }));
  }

  for handle in thread_handles {
    handle.join().unwrap();
  }
}

fn execute_workload(thread_id: usize, factory_cache: Arc<FactoryCache<i64, i64, BTreeMap<i64, Arc<i64>>>>) {
  const WORKLOAD: i64 = 1000;
  const BATCH_SIZE: i64 = 10;
  
  for i in 0..WORKLOAD {
    let load_balanced = (i + thread_id as i64) % BATCH_SIZE;

    factory_cache.get(load_balanced);
    //generate_square(&load_balanced);
  }
}
