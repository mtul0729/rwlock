use rand::distributions::{Distribution, Uniform};
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{thread, vec};
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
#[derive(Debug, Clone)]
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

//生成一个随机的布尔序列，其中true至少出现一次，并且如果只出现一次，它会出现在序列的前半部分
//length: 序列的长度
//rate: true出现的概率
//用于生成随机的读者和写者序列，控制读者和写者的数量和比例
fn generate_bool_sequence(length: usize, rate: f64) -> Vec<bool> {
    let mut rng = rand::thread_rng();

    let mut sequence: Vec<bool> = (0..length).map(|_| rng.gen_bool(rate)).collect();

    // 确保至少有一个true
    if !sequence.contains(&true) {
        let position = rng.gen_range(0..length / 2);
        sequence[position] = true;
    } else {
        let true_count = sequence.iter().filter(|&&x| x).count();
        if true_count == 1 {
            let pos = sequence.iter().position(|&x| x).unwrap();
            if pos >= length / 2 {
                let new_position = rng.gen_range(0..length / 2);
                sequence.swap(pos, new_position);
            }
        }
    }

    sequence
}
impl ReaderWriterSequence {
    /// 生成随机的读者和写者序列,读者和写者的数量总共为thread_num
    /// 读者和写者的比例为4:1
    /// 读者和写者的时间间隔为10-25ms
    fn gen_rand(thread_num: usize) -> Self {
        let mut rw_times = Vec::new();
        let between = Uniform::from(10..25); // 用于生成10-25之间的随机数
        let mut rng = rand::thread_rng();

        for bool in generate_bool_sequence(thread_num, 0.8) {
            let rw_time = between.sample(&mut rng); // 随机读/写时间
            let rw_time = Duration::from_millis(rw_time);
            let rw_time = if bool {
                RWTime::ReadTime(rw_time)
            } else {
                RWTime::WriteTime(rw_time)
            };
            let gap_time = between.sample(&mut rng) / 2; // 读者和写者进入的间隔时间
            let gap_time = Duration::from_millis(gap_time);
            rw_times.push(RWEntry {
                rw_time: rw_time,
                enter_gap: gap_time,
            });
        }
        ReaderWriterSequence(rw_times)
    }

    fn dispatch_with_writer_priority(&self) -> Duration {
        println!("写者优先策略下:");
        let lock = Arc::new(WriterPriorityRwLock::new(5));
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
                        println!("{}号线程（写者）等待时间为:\t {:?}", i + 1, waite_time);
                        let mut write_wait_time = writer_wait_time.lock().unwrap();
                        *write_wait_time += waite_time; // 累加写者等待时间

                        thread::sleep(time); // 写操作时间
                        *w += 1;
                        println!("{}号线程（写者）更新临界资源为:\t {}", i + 1, *w);
                        let duration = start.elapsed();
                        println!("{}号线程（写者）完成时间为:\t {:?}", i + 1, duration);
                    }
                    RWTime::ReadTime(time) => {
                        let r = wiriter_lock_clone.read();
                        thread::sleep(time); // 读操作时间
                        println!("{}号线程（读者）读取得临界资源:\t {}", i + 1, *r);
                        let duration = start.elapsed();
                        println!("{}号线程（读者）完成时间为:\t {:?}", i + 1, duration);
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
        println!("读写用时:\t\t {:?}", duration);
        let writer_wait_time = writer_wait_time.lock().unwrap();
        println!("各写者等待时间累计:\t {:?}", writer_wait_time);
        println!("");
        *writer_wait_time
    }

    fn dispatch_with_no_priority(&self) -> Duration {
        println!("无优先策略下:");
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
                        println!("{}号线程（写者）等待时间为:\t {:?}", i + 1, waite_time);
                        let mut write_wait_time = writer_wait_time.lock().unwrap();
                        *write_wait_time += waite_time; // 累加写者等待时间

                        thread::sleep(time); // 写操作时间
                        *w += 1;
                        println!("{}号线程（写者）更新临界资源为:\t {}", i + 1, *w);
                        let duration = start.elapsed();
                        println!("{}号线程（写者）完成时间为:\t {:?}", i + 1, duration);
                    }
                    RWTime::ReadTime(time) => {
                        let r = wiriter_lock_clone.read();
                        thread::sleep(time); // 读操作时间
                        println!("{}号线程（读者）读取得临界资源:\t {}", i + 1, *r);
                        let duration = start.elapsed();
                        println!("{}号线程（读者）完成时间为:\t {:?}", i + 1, duration);
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
        println!("读写用时:\t\t {:?}", duration);
        let writer_wait_time = writer_wait_time.lock().unwrap();
        println!("各写者等待时间累计:\t {:?}", writer_wait_time);
        println!("");
        *writer_wait_time
    }
}

use std::io;
fn main() {
    println!("-------------------------------------------------");
    println!("\t读者写者问题模拟程序");
    println!("");
    let mut input_string = String::new();
    println!("请输入模拟次数（直接回车默认为30次）：");
    // 从标准输入获得模拟次数
    io::stdin()
        .read_line(&mut input_string)
        .expect("无法读取行");
    let tests_num: usize = if input_string.trim().is_empty() {
        30 // 默认值
    } else {
        input_string.trim().parse().expect("请输入一个有效的数字")
    };
    println!("模拟次数已设置为{}次。", tests_num);

    input_string.clear(); // 清空字符串以便再次使用

    println!("请输入单次模拟的读者与写者的总线程数（直接回车默认为20）：");
    io::stdin()
        .read_line(&mut input_string)
        .expect("无法读取行");
    let threads_num: usize = if input_string.trim().is_empty() {
        20 // 默认值
    } else {
        input_string.trim().parse().expect("请输入一个有效的数字")
    };
    println!("单次模拟的总线程数已设置为{}。", threads_num);
    println!("-------------------------------------------------");
    println!("模拟将花费一定时间，是否立即开始？（按回车确认开始）");
    input_string.clear();
    io::stdin()
        .read_line(&mut input_string)
        .expect("无法读取行");

    let mut ratios = vec![];

    for i in 0..tests_num {
        println!("第{}次模拟:", i + 1);
        let sequence = ReaderWriterSequence::gen_rand(threads_num);
        let time_no_priority = sequence.dispatch_with_no_priority();
        let time_with_priority = sequence.dispatch_with_writer_priority();
        let time_ratio = time_no_priority.as_secs_f64() / time_with_priority.as_secs_f64();
        println!(
            "第{}次模拟, 无优先策略时间: {:?}, 有优先策略时间: {:?}, 比例: {:.2}",
            i + 1,
            time_no_priority,
            time_with_priority,
            time_ratio
        );
        println!("");
        if time_ratio.is_nan() {
            println!("有优先策略的时间为0，无法计算比例，不计入平均值计算");
            continue;
        }
        ratios.push(time_ratio);
    }
    println!("-------------------------------------------------");
    println!("每次模拟的比例依次是: {:?}", ratios);
    let sum: f64 = ratios.iter().sum();
    let average = sum / ratios.len() as f64;
    println!("平均比例:\t {}", average);
    //计算标准差
    let mut variance = 0.0;
    for ratio in ratios.iter() {
        variance += (*ratio - average).powi(2);
    }
    let standard_deviation = (variance / ratios.len() as f64).sqrt();
    println!("标准差:\t {}", standard_deviation);

    println!("");
    println!(
        "平均来说，写者优先策略相比于无优先策略，写者等待的时间减少了{:.2}%",
        (1.0 - 1.0 / average) * 100.0
    );
    println!("-------------------------------------------------");
    println!("模拟结束，按回车键退出程序。");
    input_string.clear();
    io::stdin()
        .read_line(&mut input_string)
        .expect("无法读取行");
}
