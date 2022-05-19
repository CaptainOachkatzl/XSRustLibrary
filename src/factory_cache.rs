use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use std::{
  collections::{BTreeMap, HashMap},
  hash::Hash,
  ops::Index,
  sync::Arc,
};

pub trait Cache<K, V>: for<'a> Index<&'a K, Output = V> {
  fn get(&self, key: &K) -> Option<&V>;
  fn insert(&mut self, key: K, value: V);
}

impl<K, V> Cache<K, V> for BTreeMap<K, V>
where
  K: Ord,
{
  fn get(&self, key: &K) -> Option<&V> {
    self.get(key)
  }

  fn insert(&mut self, key: K, value: V) {
    self.insert(key, value);
  }
}

impl<K, V> Cache<K, V> for HashMap<K, V>
where
  K: Hash + Eq,
{
  fn get(&self, key: &K) -> Option<&V> {
    self.get(key)
  }

  fn insert(&mut self, key: K, value: V) {
    self.insert(key, value);
  }
}

/// # FactoryCache
///
/// Caches the output of a factory/generator function for the given parameter(s).
///
/// Repeated calls to "get" are served directly from the cache instead.
///
/// # Examples
///
/// ```
/// fn intensive_calculation(as_f64: usize) -> f64 {
///   thread::sleep(Duration::from_secs(2));
///   as_f64 as f64
/// }
///
/// let factory_cache = FactoryCache::new(Box::new(BTreeMap::new()), Box::new(|x| intensive_calculation(*x)));
/// println!("{}", factory_cache.get(3)); // takes 2 seconds to write "3"
/// println!("{}", factory_cache.get(5)); // takes 2 seconds to write "5"
/// println!("{}", factory_cache.get(3)); // instantly writes "3"
/// ```
///
pub struct FactoryCache<K, V, C>
where
  C: Cache<K, Arc<V>>,
{
  cache: RwLock<C>,
  factory_fn: Box<dyn Fn(K) -> V + Send + Sync>,
}

impl<K, V, C> FactoryCache<K, V, C>
where
  K: Clone,
  C: Cache<K, Arc<V>>,
{
  pub fn new(cache: C, factory_fn: Box<dyn Fn(K) -> V + Send + Sync>) -> FactoryCache<K, V, C> {
    FactoryCache {
      cache: RwLock::new(cache),
      factory_fn,
    }
  }

  pub fn get(&self, key: K) -> Arc<V> {
    let read_lock = self.cache.upgradable_read();
    if let Some(val) = (*read_lock).get(&key) {
      return val.clone();
    }

    let mut write_lock = RwLockUpgradableReadGuard::upgrade(read_lock);
    let unlocked_cache = &mut *write_lock;
    let val = Arc::new((self.factory_fn)(key.clone()));
    unlocked_cache.insert(key, val.clone());
    return val;
  }
}
