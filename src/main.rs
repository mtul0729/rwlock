use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;
mod semaphore;
use semaphore::Semaphore;

pub struct RWLock<T: ?Sized> {
    read_semaphore: Semaphore,
    write_semaphore: Semaphore,
    readers_count: Mutex<i32>,
    data: UnsafeCell<T>,
}
unsafe impl<T: ?Sized + Send> Send for RWLock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for RWLock<T> {}

impl<T> RWLock<T> {
    pub fn new(data: T) -> Self {
        Self {
            read_semaphore: Semaphore::new(1),
            write_semaphore: Semaphore::new(1),
            readers_count: Mutex::new(0),
            data: UnsafeCell::new(data),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<T> {
        let mut readers = self.readers_count.lock().unwrap();
        if *readers == 0 {
            self.write_semaphore.is_available();
            self.read_semaphore.wait();
        }
        *readers += 1;
        drop(readers);
        RwLockReadGuard { lock: self }
    }

    pub fn write(&self) -> RwLockWriteGuard<T> {
        self.write_semaphore.wait();
        self.read_semaphore.wait();
        RwLockWriteGuard { lock: self }
    }
}

pub struct RwLockReadGuard<'a, T> {
    lock: &'a RWLock<T>,
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
    lock: &'a RWLock<T>,
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

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};
    //use rand::distributions::{Distribution, Uniform};

    #[test]
    fn test0() {
        use std::sync::Arc;

        // 创建一个 `RwLock` 包裹的共享数据
        let lock = Arc::new(RWLock::new(5));

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
    fn test1() {
        let reader_count = 10;
        let writer_count = 3;
        let lock = Arc::new(RWLock::new(5));
        let mut handles = vec![];

        let start = Instant::now();

        // 创建读者线程
        for i in 0..reader_count {
            // 假设有5个读者
            let lock_clone = Arc::clone(&lock);
            let handle = thread::spawn(move || {
                let r = lock_clone.read();
                thread::sleep(Duration::from_millis(100)); // 读操作时间
                println!("Read lock value: {}", *r);
                let duration = start.elapsed();
                println!("Reader{i} time: {:?}", duration);
            });
            handles.push(handle);
        }

        // 创建写者线程
        for i in 0..writer_count {
            // 假设有3个写者
            let lock_clone = Arc::clone(&lock);
            let handle = thread::spawn(move || {
                let mut w = lock_clone.write();
                thread::sleep(Duration::from_millis(200)); // 写操作时间
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
    fn test_writer() {
        let reader_count = 10;
        let writer_count = 2;
        let lock = Arc::new(RWLock::new(5));
        let mut handles = vec![];

        let start = Instant::now();
        // 创建写者线程
        for i in 0..writer_count {
            // 假设有3个写者
            let lock_clone = Arc::clone(&lock);
            let handle = thread::spawn(move || {
                let mut w = lock_clone.write();
                thread::sleep(Duration::from_millis(200)); // 写操作时间
                *w += 1;
                println!("Write lock value: {}", *w);
                let duration = start.elapsed();
                println!("Writer{i} time: {:?}", duration);
            });
            handles.push(handle);
        }
        // 创建读者线程
        for i in 0..reader_count {
            // 假设有5个读者
            let lock_clone = Arc::clone(&lock);
            let handle = thread::spawn(move || {
                let r = lock_clone.read();
                thread::sleep(Duration::from_millis(100)); // 读操作时间
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
