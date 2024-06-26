use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;
mod semaphore;
use semaphore::Semaphore;

pub struct WriterPriorityRwLock<T: ?Sized> {
    read_semaphore: Semaphore,
    write_semaphore: Semaphore,
    readers_count: Mutex<i32>,
    data: UnsafeCell<T>,
}
unsafe impl<T: ?Sized + Send> Send for WriterPriorityRwLock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for WriterPriorityRwLock<T> {}

impl<T> WriterPriorityRwLock<T> {
    pub fn new(data: T) -> Self {
        Self {
            read_semaphore: Semaphore::new(1),
            write_semaphore: Semaphore::new(1),
            readers_count: Mutex::new(0),
            data: UnsafeCell::new(data),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<T> {
        self.write_semaphore.pseudowait(); // 读者等待写者
        let mut readers = self.readers_count.lock().unwrap();
        // 如果在这里调用write_semaphore.pseudowait()，会导致死锁
        if *readers == 0 {
            self.read_semaphore.wait();
        }
        *readers += 1;
        //自动释放readers_count的锁
        RwLockReadGuard { lock: self }
    }

    pub fn write(&self) -> RwLockWriteGuard<T> {
        self.write_semaphore.wait();
        self.read_semaphore.wait();
        RwLockWriteGuard { lock: self }
    }
}

pub struct RwLockReadGuard<'a, T> {
    lock: &'a WriterPriorityRwLock<T>,
}

impl<'a, T> Deref for RwLockReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> Drop for RwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        let mut readers = self.lock.readers_count.lock().unwrap();
        *readers -= 1;
        if *readers == 0 {
            self.lock.read_semaphore.signal();
        }
    }
}

pub struct RwLockWriteGuard<'a, T> {
    lock: &'a WriterPriorityRwLock<T>,
}

impl<'a, T> Deref for RwLockWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T> Drop for RwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.read_semaphore.signal();
        self.lock.write_semaphore.signal();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::{Distribution, Uniform};
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn test0() {
        use std::sync::Arc;

        // 创建一个 `RwLock` 包裹的共享数据
        let lock = Arc::new(WriterPriorityRwLock::new(5));

        // 读线程
        let lock_clone = Arc::clone(&lock);
        let read_thread = thread::spawn(move || {
            let r = lock_clone.read();
            println!("Read lock value: {}", *r);
        });

        // 写线程
        let lock_clone = Arc::clone(&lock);
        let write_thread = thread::spawn(move || {
            let mut w = lock_clone.write();
            *w += 1;
            println!("Write lock value: {}", *w);
        });

        // 等待线程完成
        read_thread.join().unwrap();
        write_thread.join().unwrap();

        // 最终值
        let r = lock.read();
        println!("Final value: {}", *r);
    }

    #[test]
    fn basic_test() {
        let between = Uniform::from(50..100); // 随机等待时间
        let mut rng = rand::thread_rng();

        let reader_count = 10;
        let writer_count = 3;
        let lock = Arc::new(WriterPriorityRwLock::new(5));
        let mut handles = vec![];

        let start = Instant::now();

        // 创建读者线程
        for i in 0..reader_count {
            let time = between.sample(&mut rng); // 随机等待时间
            let lock_clone = Arc::clone(&lock);
            let handle = thread::spawn(move || {
                let r = lock_clone.read();
                thread::sleep(Duration::from_millis(time)); // 读操作时间
                println!("Read lock value: {}", *r);
                let duration = start.elapsed();
                println!("Reader{i} time: {:?}", duration);
            });
            handles.push(handle);
        }

        // 创建写者线程
        for i in 0..writer_count {
            let time = between.sample(&mut rng); // 随机等待时间
            let lock_clone = Arc::clone(&lock);
            let handle = thread::spawn(move || {
                let mut w = lock_clone.write();
                thread::sleep(Duration::from_millis(time)); // 写操作时间
                *w += 1;
                println!("Write lock value: {}", *w);
                let duration = start.elapsed();
                println!("Writer{i} time: {:?}", duration);
            });
            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }

        let duration = start.elapsed();
        println!("Total time: {:?}", duration);
    }

    #[test]
    fn test_writer_priority() {
        use rand::distributions::{Distribution, Uniform};

        let between = Uniform::from(50..100); // 随机等待时间
        let mut rng = rand::thread_rng();

        let reader_count = 10;
        let writer_count = 1;
        let lock = Arc::new(WriterPriorityRwLock::new(5));
        let mut handles = vec![];

        let start = Instant::now();

        // 创建写者线程
        for i in 0..writer_count {
            let time = between.sample(&mut rng); // 随机等待时间
            let lock_clone = Arc::clone(&lock);
            let handle = thread::spawn(move || {
                let mut w = lock_clone.write();
                thread::sleep(Duration::from_millis(time)); // 写操作时间
                *w += 1;
                println!("Write lock value: {}", *w);
                let duration = start.elapsed();
                println!("Writer{i} time: {:?}", duration);
            });
            handles.push(handle);
        }
        // 创建读者线程
        for i in 0..reader_count {
            let time = between.sample(&mut rng); // 随机等待时间
            let lock_clone = Arc::clone(&lock);
            let handle = thread::spawn(move || {
                let r = lock_clone.read();
                thread::sleep(Duration::from_millis(time)); // 读操作时间
                println!("Read lock value: {}", *r);
                let duration = start.elapsed();
                println!("Reader{i} time: {:?}", duration);
            });
            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }

        let duration = start.elapsed();
        println!("Total time: {:?}", duration);
    }
}
