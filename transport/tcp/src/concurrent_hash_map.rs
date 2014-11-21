use std::borrow::BorrowFrom;
use std::collections::HashMap;
use std::collections::hash_map::{Entry, Occupied, Vacant};
use std::default::Default;
use std::hash::{
  Hash,
  Hasher,
  RandomSipHasher
};
use std::sync::{
  Arc,
  Weak,
  RWLock
};

/// This is a very shitty concurrent hash map that will do for now
pub struct ConcurrentHashMap<K, V, H = RandomSipHasher>(RWLock<HashMap<K, Arc<V>, H>>);

impl<K, V> ConcurrentHashMap<K, V, RandomSipHasher>
  where K: Send + Sync + Hash + Eq,
        V: Send + Sync,
{
  pub fn new() -> ConcurrentHashMap<K, V, RandomSipHasher>
  {
    ConcurrentHashMap(RWLock::new(HashMap::new()))
  }
}

impl<K, V, S, H> ConcurrentHashMap<K, V, H>
  where K: Send + Sync + Eq + Hash<S>,
        V: Send + Sync,
        H: Send + Sync + Hasher<S>,
{
  pub fn get<Sized? Q>(&self, k: &Q) -> Option<Arc<V>>
    where Q: Hash<S> + Eq + BorrowFrom<K>
  {
    self.0.read().get(k).map(|v| v.clone())
  }

  //fn entry<'a>(&'a mut self, key: K) -> Entry<'a, K, V>
  //{
  //
  //}

  pub fn get_or_init(&self,
                     k: K,
                     init: || -> V)
                     -> Arc<V>
  {
    // Upside, might avoid write lock. Downside, first come first SUCKER
    match self.get(&k) {
      Some(v) => return v.clone(),
      _       => (),
    }

    match self.0.write().entry(k) {
      Vacant(entry)   => {
        let arc = entry.set(Arc::new(init()));
        arc.clone()
      }
      Occupied(entry) => entry.into_mut().clone(),
    }
  }
}
