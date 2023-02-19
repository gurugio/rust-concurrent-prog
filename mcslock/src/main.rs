use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::ptr::null_mut;
use std::sync::atomic::{fence, AtomicBool, AtomicPtr, Ordering};

pub struct MCSLock<T> {
    last: AtomicPtr<MCSNode<T>>,
    data: UnsafeCell<T>,
}

pub struct MCSNode<T> {
    next: AtomicPtr<MCSNode<T>>,
    locked: AtomicBool,
}

pub struct MCSLockGuard<'a, T> {
    node: &'a mut MCSNode<T>,
    mcs_lock: &'a MCSLock<T>,
}

unsafe impl<T> Sync for MCSLock<T> {}
unsafe impl<T> Send for MCSLock<T> {}

impl<T> MCSNode<T> {
    pub fn new() -> Self {
        MCSNode {
            next: AtomicPtr::new(null_mut()),
            locked: AtomicBool::new(false),
        }
    }
}

impl<'a, T> Deref for MCSLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mcs_lock.data.get() }
    }
}

impl<'a, T> DerefMut for MCSLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mcs_lock.data.get() }
    }
}

impl<T> MCSLock<T> {
    pub fn new(v: T) -> Self {
        MCSLock {
            last: AtomicPtr::new(null_mut()),
            data: UnsafeCell::new(v),
        }
    }

    pub fn lock<'a>(&'a self, node: &'a mut MCSNode<T>) -> MCSLockGuard<T> {
        node.next = AtomicPtr::new(null_mut());
        node.locked = AtomicBool::new(false);

        let guard = MCSLockGuard {
            node,
            mcs_lock: self,
        };

        let ptr = guard.node as *mut MCSNode<T>; // "as *mut MCSNode<T>" prevents move of guard.node
        let prev = self.last.swap(ptr, Ordering::Relaxed);

        if prev != null_mut() {
            guard.node.locked.store(true, Ordering::Relaxed);

            let prev = unsafe { &*prev };
            prev.next.store(ptr, Ordering::Relaxed);

            while guard.node.locked.load(Ordering::Relaxed) {}
        }
        fence(Ordering::Acquire);

        guard
    }
}

impl<'a, T> Drop for MCSLockGuard<'a, T> {
    fn drop(&mut self) {
        if self.node.next.load(Ordering::Relaxed) == null_mut() {
            let ptr = self.node as *mut MCSNode<T>;
            if let Ok(_) = self.mcs_lock.last.compare_exchange(
                ptr,
                null_mut(),
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                return;
            }
        }

        while self.node.next.load(Ordering::Relaxed) == null_mut() {}

        let next = unsafe { &mut *self.node.next.load(Ordering::Relaxed) };
        next.locked.store(false, Ordering::Release);
    }
}

use std::sync::Arc;
const NUM_LOOP: usize = 100000;
const NUM_THREADS: usize = 16;

fn main() {
    println!("Hello, world!");

    let n = Arc::new(MCSLock::new(0));
    let mut v = Vec::new();

    for _ in 0..NUM_THREADS {
        let n0 = n.clone();

        let t = std::thread::spawn(move || {
            let mut node = MCSNode::new();
            for _ in 0..NUM_LOOP {
                let mut r = n0.lock(&mut node);
                *r += 1;
            }
        });
        v.push(t);
    }

    for t in v {
        t.join().unwrap();
    }

    let mut node = MCSNode::new();
    let r = n.lock(&mut node);
    println!("count={} expected={}", *r, NUM_LOOP * NUM_THREADS);
}
