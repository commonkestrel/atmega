use core::mem::drop;
use core::hint::spin_loop;
use core::cell::UnsafeCell;
use core::ops::{ Deref, DerefMut };
use core::sync::atomic::{ AtomicBool, Ordering };

#[derive(Debug)]
pub enum TryLockError {
    Locked,
}

impl core::fmt::Display for TryLockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use TryLockError::*;
        let err = match self {
            Locked => "Mutex is already locked",
        };

        write!(f, "{}", err)
    }
}

impl core::error::Error for TryLockError {
    fn description(&self) -> &str {
        use TryLockError::*;
        match self {
            Locked => "Mutex is already locked",
        }
    }
}

pub type TryLockResult<T> = Result<T, TryLockError>;

pub struct Mutex<T> {
    data: UnsafeCell<T>,
    lock: AtomicBool,
}

impl<T> Mutex<T> {
    #[inline]
    pub const fn new(data: T) -> Self {
        Mutex {
            data: UnsafeCell::new(data),
            lock: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        while self.lock.swap(true, Ordering::AcqRel) {
            spin_loop();
        }
        MutexGuard{ mutex: &self }
    }

    pub fn try_lock(&self) -> TryLockResult<MutexGuard<'_, T>> {
        if self.lock.load(Ordering::SeqCst) {
            return TryLockResult::Err(TryLockError::Locked);
        }
        Ok(MutexGuard{ mutex: &self })
    }

    fn unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }
}

impl<T: Default> Default for Mutex<T> {
    fn default() -> Self {
        Mutex::new(T::default())
    }
}

impl<T> From<T> for Mutex<T> {
    fn from(value: T) -> Self {
        Mutex::new(value)
    }
}

unsafe impl<T> Send for Mutex<T> where T: Send {}
unsafe impl<T> Sync for Mutex<T> where T: Send {}


pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<T> MutexGuard<'_, T> {
    pub fn unlock(self) {
        drop(self)
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.unlock();
    }
}

unsafe impl<T> Send for MutexGuard<'_, T> where T: Send {}
unsafe impl<T> Sync for MutexGuard<'_, T> where T: Send + Sync {}
