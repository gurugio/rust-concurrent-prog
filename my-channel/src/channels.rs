use crate::semaphore::Semaphore;
use std::collections::LinkedList;
use std::sync::{Arc, Condvar, Mutex};

#[derive(Clone)]
pub struct Sender<T> {
    sem: Arc<Semaphore>,
    buf: Arc<Mutex<LinkedList<T>>>,
    cond: Arc<Condvar>,
}

impl<T: Send> Sender<T> {
    pub fn send(&self, data: T) {
        self.sem.wait();
        let mut buf = self.buf.lock().unwrap();
        buf.push_back(data);
        self.cond.notify_one();
    }
}

pub struct Receiver<T> {
    sem: Arc<Semaphore>,
    buf: Arc<Mutex<LinkedList<T>>>,
    cond: Arc<Condvar>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> T {
        let mut buf = self.buf.lock().unwrap();
        loop {
            if let Some(data) = buf.pop_front() {
                self.sem.post();
                return data;
            }

            buf = self.cond.wait(buf).unwrap();
        }
    }
}

pub fn channel<T>(max: isize) -> (Sender<T>, Receiver<T>) {
    assert!(max > 0);
    let sem = Arc::new(Semaphore::new(max));
    let buf = Arc::new(Mutex::new(LinkedList::new()));
    let cond = Arc::new(Condvar::new());
    let tx = Sender {
        sem: sem.clone(),
        buf: buf.clone(),
        cond: cond.clone(),
    };
    let rx = Receiver { sem, buf, cond };
    (tx, rx)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_channel() {
        const NUM_LOOP: usize = 10000;
        const NUM_THREADS: usize = 8;

        let (tx, rx) = channel(4);
        let mut v = Vec::new();

        let t = std::thread::spawn(move || {
            let mut cnt = 0;
            while cnt < NUM_THREADS * NUM_LOOP {
                let n = rx.recv();
                println!("recv: n = {:?}", n);
                cnt += 1;
            }
        });

        v.push(t);

        for i in 0..NUM_THREADS {
            let tx0 = tx.clone();
            let t = std::thread::spawn(move || {
                for j in 0..NUM_LOOP {
                    tx0.send((i, j));
                }
            });
            v.push(t);
        }

        for t in v {
            t.join().unwrap();
        }
    }
}
