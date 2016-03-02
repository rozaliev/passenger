#![feature(stmt_expr_attributes)]
#![cfg_attr(feature="benchmark", feature(test))]

#[cfg(feature = "benchmark")]
extern crate criterion;
#[cfg(feature = "benchmark")]
extern crate test;
extern crate passenger;



#[cfg(feature = "benchmark")]
use criterion::Criterion;

#[cfg(feature = "benchmark")]
mod benches {
    use std::sync::mpsc::sync_channel;
    use std::thread;
    use test::black_box;

    use passenger::BoundedSpscQueue;
    use criterion::Bencher;


    pub fn simple_sync_std(b: &mut Bencher) {
        let (sender, receiver) = sync_channel::<i32>(1000);
        thread::spawn(move || {
            loop {
                match sender.send(0) {
                    Ok(_) => {}
                    Err(_) => return,
                }
            }
        });

        let _ = receiver.recv();

        b.iter(|| black_box(receiver.recv()));

    }


    pub fn simple_bounded_spsc(b: &mut Bencher) {
        let (mut sender, mut receiver) = BoundedSpscQueue::new(1000);


        thread::spawn(move || {
            loop {
                match sender.send(0) {
                    Ok(_) => {}
                    Err(_) => return,

                }
            }
        });

        let _ = receiver.recv();

        b.iter(|| black_box(receiver.recv()));

    }




    pub fn simple_single_thread_sync_std(b: &mut Bencher) {
        let (sender, receiver) = sync_channel::<i32>(1000);
        b.iter(|| {
            sender.send(0).unwrap();
            receiver.recv().unwrap();
        });
    }



    pub fn simple_single_thread_bounded_spsc(b: &mut Bencher) {
        let (mut sender, mut receiver) = BoundedSpscQueue::new(1000);

        b.iter(|| {
            sender.send(0).unwrap();
            receiver.recv().unwrap();
        });
    }

}
#[cfg(feature = "benchmark")]
use benches::*;


fn main() {
    #[cfg(feature = "benchmark")]
    {
        let mut b = Criterion::default();

        b.bench_function("std single thread", simple_single_thread_sync_std);
        b.bench_function("bounded spsc single thread",
                         simple_single_thread_bounded_spsc);
        b.bench_function("std", simple_sync_std);
        b.bench_function("bounded spsc", simple_bounded_spsc);

    }
}
