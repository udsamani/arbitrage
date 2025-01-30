use std::sync::{Arc, Mutex, MutexGuard};

pub struct SharedRef<T> {
    inner: Arc<Mutex<T>>,
}

impl<T: Default> Default for SharedRef<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> Clone for SharedRef<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> SharedRef<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(value)),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        self.inner.lock().unwrap()
    }
}
