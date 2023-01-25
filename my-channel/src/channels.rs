use crate::semaphore::semaphore;
use std::collections::LinkedList;
use std::sync::{Arc, Condvar, Mutex};

#[derive(Clone)]
pub struct 