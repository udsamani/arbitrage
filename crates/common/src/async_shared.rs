use std::sync::Arc;

use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};

/// A shared reference to a value that can be read and written to asynchronously.
///
/// This type of lock does not block the thread when the lock is already held by another task.
/// Rather it returns the yield back to the thread and allows other tasks to run.
pub struct SharedAsyncRef<T> {
    inner: Arc<RwLock<T>>
}

impl<T> Clone for SharedAsyncRef<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<T: Default> Default for SharedAsyncRef<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> SharedAsyncRef<T> {
    pub fn new(value: T) -> Self {
        Self { inner: Arc::new(RwLock::new(value)) }
    }

    pub async fn read(&self) -> RwLockReadGuard<T> {
        self.inner.read().await
    }

    pub async fn write(&self) -> RwLockWriteGuard<T> {
        self.inner.write().await
    }

    pub fn try_write(&self) -> Result<RwLockWriteGuard<T>, TryLockError> {
        self.inner.try_write()
    }
}
