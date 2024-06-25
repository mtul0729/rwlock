use std::sync::{Condvar, Mutex};

pub struct Semaphore {
    count: Mutex<i32>, //临界资源的数目，相当于(同时访问该资源的最大线程数-1)。Mutex是互斥锁，用于保护临界资源的访问
    condvar: Condvar,  //条件变量，用于线程间的通信
}

impl Semaphore {
    pub fn new(count: i32) -> Self {
        Self {
            count: Mutex::new(count),
            condvar: Condvar::new(),
        }
    }

    pub fn wait(&self) {
        let mut count = self.count.lock().unwrap();
        *count -= 1;
        // 阻塞，等待条件变量的通知，同时释放锁，遵循“让权等待”原则。得到通知后，重新获取锁，继续执行
        // 使用 闭包(|count| *count < 0) 来防止虚假唤醒，不需要显式的 while 循环
        let _guard = self.condvar.wait_while(count, |count| *count < 0); 
    }

    pub fn signal(&self) {
        let mut count = self.count.lock().unwrap();
        *count += 1;
        self.condvar.notify_one(); // 通知一个等待的线程
    }

    pub fn is_available(&self) {
        let mut _count = self.count.lock().unwrap();
        let _guard = self.condvar.wait_while(_count, |_count| *_count < 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    #[test]
    fn test_semaphore() {
        use rand::distributions::{Distribution, Uniform};

        let between = Uniform::from(0..100);
        let mut rng = rand::thread_rng();

        let semaphore = Arc::new(Semaphore::new(3)); // 初始计数为3的信号量
        let mut handles = vec![];

        for i in 0..10 {
            let time = between.sample(&mut rng); // 随机等待0~99ms
            let semaphore = Arc::clone(&semaphore);
            handles.push(thread::spawn(move || {
                semaphore.wait();
                println!("Thread {} is running", i);
                thread::sleep(std::time::Duration::from_millis(time));
                println!("Thread {} is done", i);
                semaphore.signal();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
