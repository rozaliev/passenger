#![feature(alloc, oom, heap_api)]

// TODO
// mpsc
// mpmc

extern crate alloc;

mod spsc;


pub use spsc::bounded::{
    BoundedSpscQueue,
    Receiver as BoundedSpscReceiver,
    Sender as BoundedSpscSender,
    TrySendError as SpscTrySendError,
    TryReceiveError as SpscTryReceiveError,
};
