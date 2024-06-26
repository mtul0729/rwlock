//use core::{num, time};
use rand::distributions::{Distribution, Uniform};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use rand::Rng;

mod semaphore;

mod no_priority_rw_lock;
use no_priority_rw_lock::NoPriorityRwLock;

mod writer_priority_rw_lock;
use writer_priority_rw_lock::WriterPriorityRwLock;

#[derive(Debug, Clone)]
enum RWTime {
    ReadTime(Duration),
    WriteTime(Duration),
}

use std::ops::Deref;
#[derive(Debug,Clone)]
struct RWEntry {
    rw_time: RWTime,
    enter_gap: Duration,
}

struct ReaderWriterSequence(Vec<RWEntry>);

impl Deref for ReaderWriterSequence {
    type Target = Vec<RWEntry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ReaderWriterSequence {

    /// 生成随机的读者和写者序列,读者和写者的数量总共为thread_num
    /// 读者和写者的比例为4:1
    /// 读者和写者的时间间隔为10-25ms
    fn gen_rand(thread_num: u32) -> Self {
        let mut rw_times = Vec::new();
        let between = Uniform::from(10..25); // 用于生成10-25之间的随机数
        let mut rng = rand::thread_rng();

        for _ in 0..thread_num {
            let rw_time = between.sample(&mut rng); // 随机等待时间
            let rw_time = Duration::from_millis(rw_time);
            let rw_time = if rng.gen_bool(0.8) { // 生成80%的读者,20%的写者
                RWTime::ReadTime(rw_time)
            } else {
                RWTime::WriteTime(rw_time)
            };
            let gap_time = between.sample(&mut rng) / 2;
            let gap_time = Duration::from_millis(gap_time);
            rw_times.push(RWEntry {
                rw_time: rw_time,
                enter_gap: gap_time,
            });
        }
        ReaderWriterSequence(rw_times)
    }

    fn dispatch_with_no_priority(&self) -> Duration{
        let lock = Arc::new(NoPriorityRwLock::new(5));
        let mut handles = vec![];

        let start = Instant::now();
        let writer_wait_time = Arc::new(Mutex::new(Duration::from_millis(0)));

        // 创建写者与读者线程
        for (i, rw_entry) in self.iter().enumerate() {
            let rw_time = rw_entry.rw_time.clone();
            let enter_gap = rw_entry.enter_gap.clone();
            // 创建写者线程
            let wiriter_lock_clone = Arc::clone(&lock);
            let writer_wait_time = Arc::clone(&writer_wait_time);
            thread::sleep(enter_gap); // 读者和写者进入的间隔时间
            let writer_handle = thread::spawn(move || {
                match rw_time {
                    RWTime::WriteTime(time) => {
                        let start_waite = Instant::now();
                        let mut w = wiriter_lock_clone.write();
                        let waite_time = start_waite.elapsed();
                        println!("{}号写者等待时间为:\t {:?}", i + 1, waite_time);
                        let mut write_wait_time = writer_wait_time.lock().unwrap();
                        *write_wait_time += waite_time; // 累加写者等待时间

                        thread::sleep(time); // 写操作时间
                        *w += 1;
                        println!("{}号写者将临界资源更新为:\t {}", i + 1, *w);
                        let duration = start.elapsed();
                        println!("{}号写者完成时间为:\t {:?}", i + 1, duration);
                    }
                    RWTime::ReadTime(time) => {
                        let r = wiriter_lock_clone.read();
                        thread::sleep(time); // 读操作时间
                        println!("{}号读者读取得临界资源:\t {}", i + 1, *r);
                        let duration = start.elapsed();
                        println!("{}号读者完成时间为:\t {:?}", i + 1, duration);
                    }
                }
            });
            handles.push(writer_handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }

        let duration = start.elapsed();
        println!("所有进程用时: {:?}", duration);
        let writer_wait_time = writer_wait_time.lock().unwrap();
        println!("所有写者等待时间累计: {:?}",writer_wait_time);
        *writer_wait_time
    }
}


fn main() {
    //run_writer_priority(5, 1000);
    //run_no_priority(5, 1000);
    let sequence = ReaderWriterSequence::gen_rand(50);
    sequence.dispatch_with_no_priority();
}

