#![feature(alloc, oom, heap_api, optin_builtin_traits)]


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
