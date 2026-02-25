# lab5
## 简单总结你实现的功能（200字以内，不要贴代码）及你完成本次实验所用的时间。
银行家算法，实现了死锁检测。完成本次实验大约花费了四天。


## 问答作业
### 1. 在我们的多线程实现中，当主线程 (即 0 号线程) 退出时，视为整个进程退出， 此时需要结束该进程管理的所有线程并回收其资源。 - 需要回收的资源有哪些？ - 其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？
需要回收：给每个线程分配的user resource，包括tid，用户栈，trap context区域，删除内核空间中给每个线程分配的内核栈，以及整个进程的资源，包括各种数据结构，与内存中分配的页面。

其他线程的 TaskControlBlock 可能在以下位置被引用：主线程的暂时放到stopping task中，延迟回收，其他线程可能会存在于进程的线程列表tasks中，随着进程被回收一起被回收，还可能存在于TASK_MANAGER的调度队列中，需要删掉，还可能存在于timer计数器队列的TimerCondVar中，需要删掉。

### 2. 对比以下两种 Mutex 中的实现，二者有什么区别？这些区别可能会导致什么问题？
区别：Mutex1的lock有一个loop，unlock无论如何会设置lock为false，Mutex2的lock没有loop，unlock只有在没有等待线程时才会设置lock为false。我觉得都没有问题，无非是Mutex1重新循环了一次，Mutex2直接退出lock函数了，lock和unlock都是原子操作。

```rust
 1impl Mutex for Mutex1 {
 2    fn lock(&self) {
 3        loop {
 4            let mut mutex_inner = self.inner.exclusive_access();
 5            if mutex_inner.locked {
 6                mutex_inner.wait_queue.push_back(current_task().unwrap());
 7                drop(mutex_inner);
 8                block_current_and_run_next();
 9            } else {
10                mutex_inner.locked = true;
11                break;
12            }
13        }
14    }
15
16    fn unlock(&self) {
17        let mut mutex_inner = self.inner.exclusive_access();
18        assert!(mutex_inner.locked);
19        mutex_inner.locked = false;
20        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
21            add_task(waking_task);
22        }
23    }
24}
25
26impl Mutex for Mutex2 {
27    fn lock(&self) {
28        let mut mutex_inner = self.inner.exclusive_access();
29        if mutex_inner.locked {
30            mutex_inner.wait_queue.push_back(current_task().unwrap());
31            drop(mutex_inner);
32            block_current_and_run_next();
33        } else {
34            mutex_inner.locked = true;
35        }
36    }
37
38    fn unlock(&self) {
39        let mut mutex_inner = self.inner.exclusive_access();
40        assert!(mutex_inner.locked);
41        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
42            add_task(waking_task);
43        } else {
44            mutex_inner.locked = false;
45        }
46    }
47}
```
## HONOR CODE
无