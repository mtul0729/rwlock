//use core::{num, time};
use rand::distributions::{Distribution, Uniform};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

mod semaphore;

mod no_priority_rw_lock;
use no_priority_rw_lock::NoPriorityRwLock;

mod writer_priority_rw_lock;
use writer_priority_rw_lock::WriterPriorityRwLock;

fn main() {
    let between = Uniform::from(10..20); // 随机等待时间
    let mut rng = rand::thread_rng();
    let time = between.sample(&mut rng); // 随机等待时间
    let tread_num = 5;

    let time1 = run_writer_priority(tread_num, time);
    let time2 = run_no_priority(tread_num, time);
    println!("写者优先算法的写者周转时间比例为: {:.2}%", time1 * 100.0);
    println!("无优先级算法的写者周转时间比例为: {:.2}%", time2 * 100.0);
}

fn run_writer_priority(count: u32, time: u64) -> f64 {
    //TODO: 读者和写者一次性全部进入无法验证写者优先算法
    //TODO: 设置两个线程分别定时产生读者和写者
    let lock = Arc::new(WriterPriorityRwLock::new(5));
    let mut handles = vec![];

    let start = Instant::now();
    let write_done_time = Arc::new(Mutex::new(Duration::from_millis(0)));

    // 创建写者与读者线程
    for i in 0..count {
        // 创建写者线程
        let wiriter_lock_clone = Arc::clone(&lock);
        let writer_done_time = Arc::clone(&write_done_time);
        let writer_handle = thread::spawn(move || {
            let mut w = wiriter_lock_clone.write();
            thread::sleep(Duration::from_millis(time)); // 写操作时间
            *w += 1;
            println!("{}号写者将临界资源更新为:\t {}", i + 1, *w);
            let duration = start.elapsed();
            println!("{}号写者完成时间为:\t {:?}", i + 1, duration);

            // 记录写者完成时间,将得到最后一个写者完成时间
            let mut write_done_time = writer_done_time.lock().unwrap();
            *write_done_time = duration;
        });
        handles.push(writer_handle);

        // 创建读者线程
        let reader_lock_clone = Arc::clone(&lock);
        let reader_handle = thread::spawn(move || {
            let r = reader_lock_clone.read();
            thread::sleep(Duration::from_millis(time)); // 读操作时间
            println!("{}号读者的读取临界资源为:\t {}", i + 1, *r);
            let duration = start.elapsed();
            println!("{}号读者完成时间为:\t {:?}", i + 1, duration);
        });
        handles.push(reader_handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    println!("所有进程用时: {:?}", duration);
    let write_time = *write_done_time.lock().unwrap();
    println!("最后一个写者完成时间为: {:?}", write_time);
    let write_ratio = write_time.as_secs_f64() / duration.as_secs_f64();
    println!("写者周转时间比例为: {:.2}%", write_ratio * 100.0);
    write_ratio
}

fn run_no_priority(count: u32, time: u64) -> f64 {
    let lock = Arc::new(NoPriorityRwLock::new(5));
    let mut handles = vec![];

    let start = Instant::now();
    let write_done_time = Arc::new(Mutex::new(Duration::from_millis(0)));

    // 创建写者与读者线程
    for i in 0..count {
        // 创建写者线程
        let wiriter_lock_clone = Arc::clone(&lock);
        let writer_done_time = Arc::clone(&write_done_time);
        let writer_handle = thread::spawn(move || {
            let mut w = wiriter_lock_clone.write();
            thread::sleep(Duration::from_millis(time)); // 写操作时间
            *w += 1;
            println!("{}号写者将临界资源更新为:\t {}", i + 1, *w);
            let duration = start.elapsed();
            println!("{}号写者完成时间为:\t {:?}", i + 1, duration);

            // 记录写者完成时间,将得到最后一个写者完成时间
            let mut write_done_time = writer_done_time.lock().unwrap();
            *write_done_time = duration;
        });
        handles.push(writer_handle);

        // 创建读者线程
        let reader_lock_clone = Arc::clone(&lock);
        let reader_handle = thread::spawn(move || {
            let r = reader_lock_clone.read();
            thread::sleep(Duration::from_millis(time)); // 读操作时间
            println!("{}号读者的读取临界资源为:\t {}", i + 1, *r);
            let duration = start.elapsed();
            println!("{}号读者完成时间为:\t {:?}", i + 1, duration);
        });
        handles.push(reader_handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    println!("所有进程用时: {:?}", duration);
    let write_time = *write_done_time.lock().unwrap();
    println!("最后一个写者完成时间为: {:?}", write_time);
    let write_ratio = write_time.as_secs_f64() / duration.as_secs_f64();
    println!("写者周转时间比例为: {:.2}%", write_ratio * 100.0);
    write_ratio
}
