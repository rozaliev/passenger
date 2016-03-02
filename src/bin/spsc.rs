extern crate passenger;

use passenger::BoundedSpscQueue;
use std::thread;

fn main() {
    let (mut sender, mut receiver) = BoundedSpscQueue::new(1000);
    thread::spawn(move || {
        loop {
            match sender.send(0) {
                Ok(_) => {}
                Err(_) => return,

            }
        }

    });

    loop {
        let _ = receiver.recv().unwrap();
    }

}
