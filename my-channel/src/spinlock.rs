use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct SpinLock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

struct SpinLockGuard<'a, T> {
    spin_lock: &'a SpinLock<T>,
}

impl<T> SpinLock<T> {
    fn new(v: T) -> Self {
        SpinLock {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(v),
        }
    }

    fn lock(&self) -> SpinLockGuard<T> {
        loop {
            while self.lock.load(Ordering::Relaxed) {}

            if let Ok(_) =
                self.lock
                    .compare_exchange_weak(false, true, Ordering::AcqRel, Ordering::Relaxed)
            {
                break;
            }
        }
        SpinLockGuard { spin_lock: self }
    }
}

unsafe impl<T> Sync for SpinLock<T> {}
unsafe impl<T> Send for SpinLock<T> {}

impl<'a, T> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        self.spin_lock.lock.store(false, Ordering::Release);
    }
}

impl<'a, T> Deref for SpinLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.spin_lock.data.get() }
    }
}

impl<'a, T> DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.spin_lock.data.get() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinlock() {
        const NUM_THREADS: usize = 4;
        const NUM_LOOP: usize = 10000;

        let lock = Arc::new(SpinLock::new(0));
        let mut v = Vec::new();

        for _ in 0..NUM_THREADS {
            let lock0 = lock.clone();
            let t = std::thread::spawn(move || {
                for _ in 0..NUM_LOOP {
                    let mut data = lock0.lock();
                    *data += 1;
                }
            });

            v.push(t);
        }

        for t in v {
            t.join().unwrap();
        }

        println!(
            "COUNT = {} expected={}",
            *lock.lock(),
            NUM_LOOP * NUM_THREADS
        );
    }
}
