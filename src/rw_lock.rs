use crate::semaphore::Semaphore;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;

/// 读写锁的优先策略
pub trait RwLockPolicy {
    fn before_read(semaphore: &Semaphore);
}

/// 无优先策略
pub struct NoPriorityPolicy;

impl RwLockPolicy for NoPriorityPolicy {
    fn before_read(_: &Semaphore) {
        // No-op for no priority
    }
}

/// 写者优先策略
pub struct WriterPriorityPolicy;

impl RwLockPolicy for WriterPriorityPolicy {
    fn before_read(semaphore: &Semaphore) {
        semaphore.pseudowait(); // 读者等待写者
    }
}

pub struct RwLock<T: ?Sized, P: RwLockPolicy> {
    read_semaphore: Semaphore,
    write_semaphore: Semaphore,
    readers_count: Mutex<i32>,
    _policy: std::marker::PhantomData<P>,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send, P: RwLockPolicy> Send for RwLock<T, P> {}
unsafe impl<T: ?Sized + Send + Sync, P: RwLockPolicy> Sync for RwLock<T, P> {}

impl<T, P: RwLockPolicy> RwLock<T, P> {
    /// 创建0-1信号量
    pub fn new(data: T) -> Self {
        Self {
            read_semaphore: Semaphore::new(1),
            write_semaphore: Semaphore::new(1),
            readers_count: Mutex::new(0),
            _policy: std::marker::PhantomData,
            data: UnsafeCell::new(data),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<T, P> {
        P::before_read(&self.write_semaphore);
        let mut readers = self.readers_count.lock().unwrap();
        if *readers == 0 {
            self.read_semaphore.wait();
        }
        *readers += 1;
        RwLockReadGuard { lock: self }
    }

    pub fn write(&self) -> RwLockWriteGuard<T, P> {
        self.write_semaphore.wait();
        self.read_semaphore.wait();
        RwLockWriteGuard { lock: self }
    }
}

pub struct RwLockReadGuard<'a, T, P: RwLockPolicy> {
    lock: &'a RwLock<T, P>,
}

impl<T, P: RwLockPolicy> Deref for RwLockReadGuard<'_, T, P> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T, P: RwLockPolicy> Drop for RwLockReadGuard<'_, T, P> {
    fn drop(&mut self) {
        let mut readers = self.lock.readers_count.lock().unwrap();
        *readers -= 1;
        if *readers == 0 {
            self.lock.read_semaphore.signal();
        }
    }
}

pub struct RwLockWriteGuard<'a, T, P: RwLockPolicy> {
    lock: &'a RwLock<T, P>,
}

impl<T, P: RwLockPolicy> Deref for RwLockWriteGuard<'_, T, P> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T, P: RwLockPolicy> DerefMut for RwLockWriteGuard<'_, T, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T, P: RwLockPolicy> Drop for RwLockWriteGuard<'_, T, P> {
    fn drop(&mut self) {
        self.lock.read_semaphore.signal();
        self.lock.write_semaphore.signal();
    }
}

pub type NoPriorityRwLock<T> = RwLock<T, NoPriorityPolicy>;
pub type WriterPriorityRwLock<T> = RwLock<T, WriterPriorityPolicy>;

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
        let lock = Arc::new(NoPriorityRwLock::new(5));

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
        let lock = Arc::new(NoPriorityRwLock::new(5));
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
        let lock = Arc::new(NoPriorityRwLock::new(5));
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
