//use core::{num, time};
use rand::distributions::{Distribution, Uniform};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::time::{sleep, Duration};
//use std::time::{Duration, Instant};

mod semaphore;

mod no_priority_rw_lock;
use no_priority_rw_lock::NoPriorityRwLock;

mod writer_priority_rw_lock;
use writer_priority_rw_lock::WriterPriorityRwLock;
#[tokio::main]
async fn main() {
    let between = Uniform::from(10..20); // 随机等待时间
    let mut rng = rand::thread_rng();
    let rw_time = std::time::Duration::from_millis(between.sample(&mut rng)); // 随机等待时间
    let thread_num = 5; //写者线程数，读者线程数为thread_num*2
    let spawn_gap = tokio::time::Duration::from_millis(15); // 生成写者线程间隔时间，读者线程间隔时间为spawn_gap/2

    let time1 = run_writer_priority_async(thread_num, rw_time, spawn_gap).await;
    //let time2 = run_no_priority_async(thread_num, rw_time, spawn_gap).await;
    println!("写者优先算法的写者周转时间比例为: {:.2}%", time1 * 100.0);
    //println!("无优先级算法的写者周转时间比例为: {:.2}%", time2 * 100.0);
}

use std::time::Instant;

async fn run_writer_priority_async(
    thread_num: u32,
    rw_time: std::time::Duration,
    spawn_gap: tokio::time::Duration,
) -> f64 {
    //TODO: 存在系统级线程不被阻塞的问题，会影响总结束时间duration的计算
    let lock = Arc::new(WriterPriorityRwLock::new(5));
    let write_done_time = Arc::new(Mutex::new(Duration::from_millis(0)));
    let start = Instant::now();

    let writer_task = async {
        for i in 0..thread_num {
            let wiriter_lock_clone = Arc::clone(&lock);
            let writer_done_time = Arc::clone(&write_done_time);
            tokio::spawn(async move {
                sleep(spawn_gap).await;
                thread::spawn(move || {
                    let mut w = wiriter_lock_clone.write();
                    thread::sleep(rw_time); // 写操作时间
                    *w += 1;
                    println!("{}号写者将临界资源更新为:\t {}", i + 1, *w);
                    let duration = start.elapsed();
                    println!("{}号写者完成时间为:\t {:?}", i + 1, duration);

                    // 记录写者完成时间,将得到最后一个写者完成时间
                    let mut write_done_time = writer_done_time.lock().unwrap();
                    *write_done_time = duration;
                });
            })
            .await
            .unwrap();
        }
    };

    let reader_task = async {
        for i in 0..thread_num * 2 {
            // 读者数目比写者多
            let reader_lock_clone = Arc::clone(&lock);
            tokio::spawn(async move {
                sleep(spawn_gap / 2).await;
                thread::spawn(move || {
                    let r = reader_lock_clone.read();
                    thread::sleep(rw_time); // 读操作时间
                    println!("{}号读者的读取临界资源为:\t {}", i + 1, *r);
                    let duration = start.elapsed();
                    println!("{}号读者完成时间为:\t {:?}", i + 1, duration);
                });
            })
            .await
            .unwrap();
        }
    };

    tokio::join!(writer_task, reader_task);

    let duration = start.elapsed();
    println!("所有进程用时: {:?}", duration);
    let write_time = *write_done_time.lock().unwrap();
    println!("最后一个写者完成时间为: {:?}", write_time);
    let write_ratio = write_time.as_secs_f64() / duration.as_secs_f64();
    println!("写者周转时间比例为: {:.2}%", write_ratio * 100.0);
    write_ratio
}

async fn run_no_priority_async(
    thread_num: u32,
    rw_time: std::time::Duration,
    spawn_gap: tokio::time::Duration,
) -> f64 {
    let lock = Arc::new(NoPriorityRwLock::new(5));
    let write_done_time = Arc::new(Mutex::new(Duration::from_millis(0)));
    let start = Instant::now();

    let writer_task = async {
        for i in 0..thread_num {
            let wiriter_lock_clone = Arc::clone(&lock);
            let writer_done_time = Arc::clone(&write_done_time);
            tokio::spawn(async move {
                sleep(spawn_gap).await;
                thread::spawn(move || {
                    let mut w = wiriter_lock_clone.write();
                    thread::sleep(rw_time); // 写操作时间
                    *w += 1;
                    println!("{}号写者将临界资源更新为:\t {}", i + 1, *w);
                    let duration = start.elapsed();
                    println!("{}号写者完成时间为:\t {:?}", i + 1, duration);

                    // 记录写者完成时间,将得到最后一个写者完成时间
                    let mut write_done_time = writer_done_time.lock().unwrap();
                    *write_done_time = duration;
                });
            })
            .await
            .unwrap();
        }
    };

    let reader_task = async {
        for i in 0..thread_num * 2 {
            // 读者数目比写者多
            let reader_lock_clone = Arc::clone(&lock);
            tokio::spawn(async move {
                sleep(spawn_gap / 2).await;
                thread::spawn(move || {
                    let r = reader_lock_clone.read();
                    thread::sleep(rw_time); // 读操作时间
                    println!("{}号读者的读取临界资源为:\t {}", i + 1, *r);
                    let duration = start.elapsed();
                    println!("{}号读者完成时间为:\t {:?}", i + 1, duration);
                });
            })
            .await
            .unwrap();
        }
    };

    tokio::join!(writer_task, reader_task);

    let duration = start.elapsed();
    println!("所有进程用时: {:?}", duration);
    let write_time = *write_done_time.lock().unwrap();
    println!("最后一个写者完成时间为: {:?}", write_time);
    let write_ratio = write_time.as_secs_f64() / duration.as_secs_f64();
    println!("写者周转时间比例为: {:.2}%", write_ratio * 100.0);
    write_ratio
}