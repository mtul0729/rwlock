# Writer-Priority Read-Write Lock

[English] [中文](./README_CN.md)

A writer-priority implementation of [std::sync::RwLock](https://doc.rust-lang.org/std/sync/struct.RwLock.html). By introducing **pseudo-write operations to detect writer contention** and **semaphore-based priority scheduling**, this project implements a writer-priority read-write lock. The core idea is to make readers proactively "yield" to writers when acquiring the lock, significantly reducing writer wait times while balancing fairness between read and write operations.

## Problem Background: Issues with Traditional Read-Write Locks

The Rust standard library's read-write lock [std::sync::RwLock](https://doc.rust-lang.org/std/sync/struct.RwLock.html) allows multiple readers or a single writer to access a resource but provides no priority policy. In high-concurrency read scenarios, writers may experience prolonged waiting (starvation) due to continuously arriving readers. This project aims to **prioritize writer requests** for scenarios where write operations are frequent and require low latency.

## Key Design

The core of this project lies in introducing **pseudo-write operations** and a semaphore mechanism to implement a writer-priority read-write lock, effectively mitigating writer starvation. Below is a detailed breakdown of the key components:

### 1. Role of Pseudo-Write Operations

- **Definition**: Before acquiring a read lock, a reader attempts to briefly acquire and immediately release a write lock (a "pseudo-write" operation).
- **Purpose**: Detect whether writers are contending for the lock:
  - **If the pseudo-write succeeds**: No writers are waiting, allowing the reader to safely acquire the read lock.
  - **If the pseudo-write fails**: Writers are either waiting or active, forcing the reader to block until notified to retry the pseudo-write.

### 2. Writer-Priority Mechanism

- **Semaphore Design**:
  - **Write-Priority Semaphore**: Writers acquire this semaphore first to block new readers.
  - **Read Control Semaphore**: Coordinates actual read/write operations.
- **Workflow Example**:
  1. **Writer Acquiring the Lock**:
     - Acquire the write-priority semaphore to block new readers.
     - Wait for active readers to complete.
     - Execute the write operation and release the semaphore.
  2. **Reader Acquiring the Lock**:
     - First attempt to acquire the write-priority semaphore (pseudo-write operation):
       - **Success**: Release immediately and acquire the read lock.
       - **Failure**: Block and wait for writer completion before retrying the pseudo-write.

### 3. Key Performance Improvements

- **Reduced Writer Wait Time**: Pseudo-write operations force readers to yield when writers are detected, preventing writers from being "starved" by subsequent readers.

### 4. Practical Impact of Pseudo-Write Operations

- **Impact on Readers**:
  - Readers incur minor overhead from pseudo-write operations.
  - Overall system fairness improves by preventing writer starvation.
- **Advantage for Writers**:
  - Writers only need to wait for active readers to finish, not subsequent ones.

## Limitations and Recommendations

The writer-priority strategy significantly increases read operation latency and introduces overhead from pseudo-operations. Thus, it is **only suitable for extreme scenarios where write operations are frequent and low latency is critical**.

For general use cases, consider using the [RwLock](https://docs.rs/parking_lot/latest/parking_lot/type.RwLock.html) provided by [parking_lot](https://lib.rs/crates/parking_lot).