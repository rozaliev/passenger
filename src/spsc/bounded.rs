use std::cmp;
use std::ptr;
use std::mem;
use std::cell::UnsafeCell;

use alloc::heap;
use alloc::oom::oom;

use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SendError<T>(pub T);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ReceiveError;


#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TrySendError<T> {
    Full(T),
    Disconnected(T),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TryReceiveError {
    Empty,
    Disconnected,
}


pub struct BoundedSpscQueue;

pub struct Sender<T> {
    core: Arc<Core<T>>,
    head: usize,
    tail: usize,
}

pub struct Receiver<T> {
    core: Arc<Core<T>>,
    head: usize,
    tail: usize,
}


pub struct Core<T> {
    ptr: UnsafeCell<*mut T>,

    len: usize,

    tail: AtomicUsize,
    head: AtomicUsize,

    dropped: AtomicBool,
}

impl<T> ::std::fmt::Debug for TrySendError<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            TrySendError::Full(..) => "Full(..)".fmt(f),
            TrySendError::Disconnected(..) => "Disconnected(..)".fmt(f),
        }
    }
}


impl BoundedSpscQueue {
    pub fn new<T>(bound: usize) -> (Sender<T>, Receiver<T>) {
        let core = Arc::new(Core::new(bound));
        (Sender::new(core.clone()), Receiver::new(core))
    }
}

impl<T> Sender<T> {
    fn new(core: Arc<Core<T>>) -> Sender<T> {
        Sender {
            core: core,
            head: 0,
            tail: 0,
        }
    }

    pub fn send(&mut self, el: T) -> Result<(), SendError<T>> {
        if self.core.dropped.load(Ordering::Relaxed) {
            return Err(SendError(el));
        }

        let next_head = self.core.wrap_add(self.head, 1);

        if next_head == self.tail {
            loop {
                self.tail = self.core.tail.load(Ordering::Relaxed);

                if next_head != self.tail {
                    break;
                } else {
                    if self.core.dropped.load(Ordering::Relaxed) {
                        return Err(SendError(el));
                    }
                }
            }

        }

        unsafe {
            let p: *mut T = (*self.core.ptr.get()).offset(self.head as isize);
            ptr::write(p, el)
        };


        self.head = next_head;
        self.core.head.store(next_head, Ordering::Relaxed);

        Ok(())

    }

    pub fn try_send(&mut self, el: T) -> Result<(), TrySendError<T>> {
        if self.core.dropped.load(Ordering::Relaxed) {
            return Err(TrySendError::Disconnected(el));
        }

        let next_head = self.core.wrap_add(self.head, 1);

        if next_head == self.tail {
            self.tail = self.core.tail.load(Ordering::Relaxed);

            if next_head == self.tail {
                return Err(TrySendError::Full(el));
            }

        }

        unsafe {
            let p: *mut T = (*self.core.ptr.get()).offset(self.head as isize);
            ptr::write(p, el)
        };


        self.head = next_head;
        self.core.head.store(next_head, Ordering::Relaxed);

        Ok(())
    }
}

impl<T> Receiver<T> {
    fn new(core: Arc<Core<T>>) -> Receiver<T> {
        Receiver {
            core: core,
            head: 0,
            tail: 0,
        }
    }

    pub fn recv(&mut self) -> Result<T, ReceiveError> {

        if self.head == self.tail {
            loop {
                self.head = self.core.head.load(Ordering::Relaxed);

                if self.head != self.tail {
                    break;
                } else {
                    if self.core.dropped.load(Ordering::Relaxed) {
                        return Err(ReceiveError);
                    }
                }
            }
        }



        let data = unsafe {
            let p: *mut T = (*self.core.ptr.get()).offset(self.tail as isize);
            ptr::read(p)
        };


        self.tail = self.core.wrap_add(self.tail, 1);
        self.core.tail.store(self.tail, Ordering::Relaxed);

        Ok(data)
    }



    pub fn try_recv(&mut self) -> Result<T, TryReceiveError> {
        if self.head == self.tail {
            self.head = self.core.head.load(Ordering::Relaxed);

            if self.head == self.tail {
                if self.core.dropped.load(Ordering::Relaxed) {
                    return Err(TryReceiveError::Disconnected);
                }

                return Err(TryReceiveError::Empty);
            }

        }



        let data = unsafe {
            let p: *mut T = (*self.core.ptr.get()).offset(self.tail as isize);
            ptr::read(p)
        };


        self.tail = self.core.wrap_add(self.tail, 1);
        self.core.tail.store(self.tail, Ordering::Relaxed);

        Ok(data)
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.core.set_dropped();
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.core.set_dropped();
    }
}

unsafe impl<T: Send> Send for Sender<T> {}
unsafe impl<T: Send> Send for Receiver<T> {}


impl<T> Core<T> {
    fn new(bound: usize) -> Core<T> {
        assert!(mem::size_of::<T>() != 0, "no ZST support");

        let len = cmp::max(bound + 1, 1 + 1).next_power_of_two();
        let align = mem::align_of::<T>();
        let elem_size = mem::size_of::<T>();
        let ptr = unsafe { heap::allocate(len * elem_size, align) };

        if ptr.is_null() {
            oom()
        }

        Core {
            tail: AtomicUsize::new(0),
            head: AtomicUsize::new(0),
            len: len,
            ptr: UnsafeCell::new(ptr as *mut _),

            dropped: AtomicBool::new(false),
        }
    }



    fn set_dropped(&self) {
        self.dropped.store(true, Ordering::Relaxed);
    }

    #[inline]
    fn wrap_add(&self, idx: usize, addend: usize) -> usize {
        wrap_index(idx + addend, self.len)
    }
}

impl<T> Drop for Core<T> {
    fn drop(&mut self) {
        let head = self.head.load(Ordering::Relaxed);
        let mut tail = self.tail.load(Ordering::Relaxed);

        while head != tail {
            let _ = unsafe {
                let p: *mut T = (*self.ptr.get()).offset(tail as isize);
                ptr::read(p)
            };


            tail = self.wrap_add(tail, 1);
        }

        let align = mem::align_of::<T>();
        let elem_size = mem::size_of::<T>();
        let num_bytes = elem_size * self.len;

        unsafe {
            let ptr = *self.ptr.get();
            heap::deallocate(ptr as *mut _, num_bytes, align);
        }


    }
}


#[inline]
fn wrap_index(index: usize, size: usize) -> usize {
    // size is always a power of 2
    debug_assert!(size.is_power_of_two());
    index & (size - 1)
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn smoke() {
        let (mut sender, mut receiver) = BoundedSpscQueue::new(2);
        sender.send(1).unwrap();
        sender.send(2).unwrap();

        assert_eq!(receiver.recv(), Ok(1));
        assert_eq!(receiver.recv(), Ok(2));

        sender.send(3).unwrap();
        assert_eq!(receiver.recv(), Ok(3));

        sender.send(1).unwrap();
        sender.send(2).unwrap();

        assert_eq!(receiver.recv(), Ok(1));
        assert_eq!(receiver.recv(), Ok(2));

    }

    #[test]
    fn try_smoke() {
        let (mut sender, mut receiver) = BoundedSpscQueue::new(2);
        sender.try_send(1).unwrap();
        sender.try_send(2).unwrap();

        assert_eq!(receiver.try_recv(), Ok(1));
        assert_eq!(receiver.try_recv(), Ok(2));

        sender.try_send(3).unwrap();
        assert_eq!(receiver.try_recv(), Ok(3));

        sender.try_send(1).unwrap();
        sender.try_send(2).unwrap();

        assert_eq!(receiver.try_recv(), Ok(1));
        assert_eq!(receiver.try_recv(), Ok(2));
    }
}
