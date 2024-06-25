use std::sync::{Arc, Mutex};

mod semaphore;
use semaphore::Semaphore;

pub struct RWLock {
    read_semaphore: Arc<Semaphore>,
    write_semaphore: Arc<Semaphore>,
    readers_count: Mutex<i32>,
}

impl RWLock {
    pub fn new() -> Self {
        Self {
            read_semaphore: Arc::new(Semaphore::new(0)), // 允许一个写操作或多个读操作
            write_semaphore: Arc::new(Semaphore::new(0)), // 确保写操作的独占性
            readers_count: Mutex::new(0),
        }
    }

    //此处与课本的实现有所不同，课本的实现没有请求读信号量，而在读操作时直接请求写信号量
    pub fn read_lock(&self) {
        let mut readers = self.readers_count.lock().unwrap();
        if *readers == 0 {
            self.write_semaphore.is_available(); // 读操作优先级低，等待写操作完成
            self.read_semaphore.wait(); // 第一个读者等待，防止写操作
        }
        *readers += 1;
    }

    pub fn read_unlock(&self) {
        let mut readers = self.readers_count.lock().unwrap();
        *readers -= 1;
        if *readers == 0 {
            self.read_semaphore.signal(); // 最后一个读者完成，允许写操作
        }
    }

    
    pub fn write_lock(&self) {
        self.write_semaphore.wait(); // 等待写操作的独占性
        self.read_semaphore.wait(); // 防止读操作
    }

    pub fn write_unlock(&self) {
        self.read_semaphore.signal(); // 允许读操作
        self.write_semaphore.signal(); // 结束写操作的独占性
    }
}
fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_write_priority() {
        use rand::distributions::{Distribution, Uniform};

        let between = Uniform::from(0..100);
        let mut rng = rand::thread_rng();

        //let time = between.sample(&mut rng); // 随机等待0~99ms

        let rw_lock = Arc::new(RWLock::new());
        let read_threads: Vec<_> = (0..5)
            .map(|_| {
                let rw_lock = Arc::clone(&rw_lock);
                thread::spawn(move || {
                    rw_lock.read_lock();
                    // 执行读操作
                    thread::sleep(Duration::from_secs(1));
                    rw_lock.read_unlock();
                })
            })
            .collect();

        let write_thread = thread::spawn(move || {
            rw_lock.write_lock();
            // 执行写操作
            thread::sleep(Duration::from_secs(2));
            rw_lock.write_unlock();
        });

        for thread in read_threads {
            thread.join().unwrap();
        }

        write_thread.join().unwrap();
    }
}
