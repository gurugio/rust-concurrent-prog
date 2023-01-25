use std::sync::{Condvar, Mutex};

pub struct Semaphore {
    mutex: Mutex<isize>,
    cond: Condvar,
    max: isize,
}

impl Semaphore {
    pub fn new(max: isize) -> Self {
        Semaphore {
            mutex: Mutex::new(0),
            cond: Condvar::new(),
            max,
        }
    }

    pub fn wait(&self) {
        let mut cnt = self.mutex.lock().unwrap();
        while *cnt >= self.max {
            cnt = self.cond.wait(cnt).unwrap();
        }
        *cnt += 1;
    }

    pub fn post(&self) {
        let mut cnt = self.mutex.lock().unwrap();
        *cnt -= 1;
        if *cnt <= self.max {
            self.cond.notify_one();
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_sema() {
        const NUM_LOOP: usize = 100000;
        const NUM_THREADS: usize = 8;
        const SEM_NUM: isize = 4;

        static mut CNT: AtomicUsize = AtomicUsize::new(0);

        let mut v = Vec::new();
        let sem = Arc::new(Semaphore::new(SEM_NUM));

        for i in 0..NUM_THREADS {
            let s = sem.clone();
            let t = std::thread::spawn(move || {
                // each thread does
                for _ in 0..NUM_LOOP {
                    s.wait();

                    unsafe { CNT.fetch_add(1, Ordering::SeqCst) };
                    let n = unsafe { CNT.load(Ordering::SeqCst) };
                    println!("semaphore: i = {}, CNT = {}", i, n);
                    assert!((n as isize) <= SEM_NUM);
                    unsafe { CNT.fetch_sub(1, Ordering::SeqCst) };
                    s.post();
                }
            });
            v.push(t);
        }

        for t in v {
            t.join().unwrap();
        }
    }
}
