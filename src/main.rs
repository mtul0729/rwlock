use rand::distributions::{Distribution, Uniform};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
mod writer_priority_rw_lock;
use writer_priority_rw_lock::WriterPriorityRwLock;

fn main() {
    let between = Uniform::from(10..20); // 随机等待时间
    let mut rng = rand::thread_rng();

    let count = 5;
    let lock = Arc::new(WriterPriorityRwLock::new(5));
    let mut handles = vec![];

    let start = Instant::now();

    // 创建写者与读者线程
    for i in 0..count {
        // 创建写者线程
        let time = between.sample(&mut rng); // 随机等待时间
        let wiriter_lock_clone = Arc::clone(&lock);
        let writer_handle = thread::spawn(move || {
            let mut w = wiriter_lock_clone.write();
            thread::sleep(Duration::from_millis(time)); // 写操作时间
            *w += 1;
            println!("Write lock value: {}", *w);
            let duration = start.elapsed();
            println!("Writer{i} time: {:?}", duration);
        });
        handles.push(writer_handle);

        // 创建读者线程
        let time = between.sample(&mut rng); // 随机等待时间
        let reader_lock_clone = Arc::clone(&lock);
        let reader_handle = thread::spawn(move || {
            let r = reader_lock_clone.read();
            thread::sleep(Duration::from_millis(time)); // 读操作时间
            println!("Read lock value: {}", *r);
            let duration = start.elapsed();
            println!("Reader{i} time: {:?}", duration);
        });
        handles.push(reader_handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    println!("Total time: {:?}", duration);
}
