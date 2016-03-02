#![feature(test)]


extern crate test;
extern crate passenger;
use test::{Bencher, black_box};

use std::sync::mpsc::sync_channel;
use std::thread;

use passenger::BoundedSpscQueue;


#[bench]
fn simple_std_thread(b: &mut Bencher) {
    b.bytes = 8;
    let (sender, receiver) = sync_channel::<i32>(1000);
    thread::spawn(move || {
        loop {
            match sender.send(0) {
                Ok(_) => {}
                Err(_) => return,
            }
        }
    });

    b.iter(|| black_box(receiver.recv()));

}

#[bench]
fn simple_bounded_spsc_thread(b: &mut Bencher) {
    b.bytes = 8;
    let (mut sender, mut receiver) = BoundedSpscQueue::new(1000);


    thread::spawn(move || {
        loop {
            match sender.send(0) {
                Ok(_) => {}
                Err(_) => return,

            }
        }
    });


    b.iter(|| black_box(receiver.recv()));

}

#[bench]
fn simple_std(b: &mut Bencher) {
    b.bytes = 8;
    let (sender, receiver) = sync_channel::<i32>(1000);
    b.iter(|| {
        sender.send(0).unwrap();
        receiver.recv().unwrap();
    });
}






#[bench]
fn simple_bounded_spsc(b: &mut Bencher) {
    b.bytes = 8;
    let (mut sender, mut receiver) = BoundedSpscQueue::new(1000);

    b.iter(|| {
        sender.send(0).unwrap();
        receiver.recv().unwrap();
    });


}
