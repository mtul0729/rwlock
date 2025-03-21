# 写者优先的读写锁
[English](./README.md) [中文]

一个写者优先的[std::sync::RwLock](https://doc.rust-lang.org/std/sync/struct.RwLock.html)。通过**伪写操作检测写者竞争**和**信号量优先级调度**，实现了写者优先的读写锁。核心思想是让读者在申请锁时主动“让步”给写者，从而显著减少写者等待时间，平衡读写操作的公平性。

## 问题背景：读写锁的问题

Rust标准库的读写锁[std::sync::RwLock](https://doc.rust-lang.org/std/sync/struct.RwLock.html)允许多读者或单写者访问资源，但不提供任何优先策略。而在高并发读场景下，写者可能因持续到来的读者而长时间等待（饥饿）。本项目旨在**优先处理写者请求**，以适应写操作频繁且需低延迟的场景。

## 关键设计

该项目的核心是通过引入**伪写操作**和信号量机制，实现写者优先的读写锁，有效减少写者饥饿问题。以下是关键点的分步解释：

### 1. 伪写操作的核心作用

- **伪写操作定义**：读者在申请读锁前，先尝试获取写锁并立即释放。
- **目的**：通过伪写操作检测当前是否存在写者竞争：
  - **若伪写成功**：表明无写者等待，读者可安全获取读锁。
  - **若伪写失败**：表明有写者正在等待或执行，读者需阻塞，直到被唤醒后重新尝试伪写。

### 2. 写者优先的实现机制

- **信号量设计**：
  - **写优先信号量**：写者请求锁时优先占用此信号量，阻止后续读者进入。
  - **读控制信号量**：协调实际的读写操作。
- **流程示例**：
  1. **写者申请锁**：
     - 获取写优先信号量，阻止新读者。
     - 等待当前活跃读者完成。
     - 执行写操作后释放信号量。
  2. **读者申请锁**：
     - 先尝试获取写优先信号量（伪写操作）：
       - **成功**：立即释放，获取读锁。
       - **失败**：阻塞，等待写者完成后重试伪写。

### 3. 性能提升的关键

- **减少写者等待时间**：伪写操作使读者在检测到写者等待时主动让步，避免写者被后续读者“插队”。

### 4. 伪写操作的实际效果

- **对读者的影响**：
  - 读者需额外执行伪写操作，略微增加单次读操作的开销。
  - 但整体系统公平性提升，避免写者饥饿。
- **对写者的优势**：
  - 写者无需等待所有后续读者完成，仅需等待当前活跃读者退出。

## 缺点与延伸

写者优先的策略会造成读操作的延迟极大增加，且伪操作会带来整体的开销，因此只适用于写操作频繁且需低延迟的极端场景。

对于一般的场景，建议使用[parking_lot](https://lib.rs/crates/parking_lot)提供的[RwLock](https://docs.rs/parking_lot/latest/parking_lot/type.RwLock.html).
