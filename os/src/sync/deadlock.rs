use alloc::vec::Vec;
use alloc::vec;
use lazy_static::*;

use crate::task::{current_process, current_tid};

use super::UPSafeCell;
/// DeadlockDetector struct, used to detect deadlock in the system
pub struct DeadlockDetector {
    /// Condition variable inner
    pub inner: UPSafeCell<DeadlockDetectorInner>,
}
pub struct DeadlockDetectorInner {
    pub available: Vec<isize>,
    pub allocation: Vec<Vec<isize>>,
    pub need: Vec<Vec<isize>>,
}

impl DeadlockDetector {
    /// create a new DeadlockDetector
    pub fn new() -> Self {
        trace!("kernel: DeadlockDetector::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(DeadlockDetectorInner {
                    available: Vec::new(),
                    allocation: Vec::new(),
                    need: Vec::new(),
                })
            },
        }
    }
    /// add task, lazy add
    pub fn add_task(&self, tid: usize) {
        let mut inner = self.inner.exclusive_access();
        // let task_num = inner.allocation.len();
        let res_num = inner.available.len();
        while tid >= inner.allocation.len() {
            inner.allocation.push(vec![0; res_num]);
            inner.need.push(vec![0; res_num]);
        }
    }
    /// add resource
    pub fn add_res(&self, rid: usize, res_count: usize) {
        let mut inner = self.inner.exclusive_access();
        let res_num = inner.available.len();
        if rid >= res_num {
            inner.available.push(res_count as isize);
            for alloc in inner.allocation.iter_mut() {
                alloc.push(0);
            }
            for need in inner.need.iter_mut() {
                need.push(0);
            }
        } else {
            inner.available[rid] = res_count as isize;
            for alloc in inner.allocation.iter_mut() {
                alloc[rid] = 0;
            }
            for need in inner.need.iter_mut() {
                need[rid] = 0;
            }
        }
    }
    /// need resource for a task
    pub fn need(&self, tid: usize, rid: usize) {
        self.add_task(tid);
        let mut inner = self.inner.exclusive_access();
        inner.need[tid][rid] += 1;
    }
    /// noneed resource for a task
    pub fn noneed(&self, tid: usize, rid: usize) {
        self.add_task(tid);
        let mut inner = self.inner.exclusive_access();
        inner.need[tid][rid] -= 1;
    }
    /// lock resource for a task
    pub fn lock(&self, tid: usize, rid: usize) {
        self.add_task(tid);
        let mut inner = self.inner.exclusive_access();
        inner.allocation[tid][rid] += 1;
        inner.need[tid][rid] -= 1;
        inner.available[rid] -= 1;
    }
    /// unlock resource for a task
    pub fn unlock(&self, tid: usize, rid: usize) {
        self.add_task(tid);
        let mut inner = self.inner.exclusive_access();
        inner.allocation[tid][rid] -= 1;
        inner.available[rid] += 1;
    }
    /// check if the system is in deadlock
    pub fn check_deadlock(&self) -> bool {
        let inner = self.inner.exclusive_access();
        let task_num = inner.allocation.len();
        let res_num = inner.available.len();
        let mut work = inner.available.clone();
        let mut finish = vec![false; task_num];
        loop {
            let mut found = false;
            for tid in 0..task_num {
                if !finish[tid] && inner.need[tid].iter().enumerate().all(|(rid, &need)| need <= work[rid]) {
                    for rid in 0..res_num {
                        work[rid] += inner.allocation[tid][rid];
                    }
                    finish[tid] = true;
                    found = true;
                }
            }
            if !found {
                break;
            }
        }
        if finish.iter().all(|&x| x) {
            false
        } else {
            true
        }
    }
}

/// Resource type enum
#[derive(PartialEq, Eq)]
pub enum ResourceType {
    /// Mutex resource type
    Mutex,
    /// Semaphore resource type
    Semaphore,
}

pub struct DeadlockDetectorManager {
    res_type: ResourceType,
}
impl DeadlockDetectorManager {
    /// create a new DeadlockDetectorManager
    pub fn new() -> Self {
        trace!("kernel: DeadlockDetectorManager::new");
        Self {
            res_type: ResourceType::Mutex,
        }
    }
    /// set resource type for deadlock detector manager
    pub fn set_res_type(&mut self, res_type: ResourceType) {
        self.res_type = res_type;
    }
    /// need resource for a task
    pub fn need(&self, rid: usize) {
        let process = current_process();
        let process_inner = process.inner_exclusive_access();
        if self.res_type == ResourceType::Mutex {
            process_inner.mutex_deadlock_detector.need(
                current_tid(),
                rid,
            );
        } else if self.res_type == ResourceType::Semaphore {
            process_inner.semaphore_deadlock_detector.need(
                current_tid(),
                rid,
            );
        }
    }
    /// noneed resource for a task    
    pub fn noneed(&self, rid: usize) {
        let process = current_process();
        let process_inner = process.inner_exclusive_access();
        if self.res_type == ResourceType::Mutex {
            process_inner.mutex_deadlock_detector.noneed(
                current_tid(),
                rid,
            );
        } else if self.res_type == ResourceType::Semaphore {
            process_inner.semaphore_deadlock_detector.noneed(
                current_tid(),
                rid,
            );
        }
    }
    /// lock resource for a task
    pub fn lock(&self, rid: usize) {
        let process = current_process();
        let process_inner = process.inner_exclusive_access();
        if self.res_type == ResourceType::Mutex {
            process_inner.mutex_deadlock_detector.lock(
                current_tid(),
                rid,
            );
        } else if self.res_type == ResourceType::Semaphore {
            process_inner.semaphore_deadlock_detector.lock(
                current_tid(),
                rid,
            );
        }
    }
    /// unlock resource for a task
    pub fn unlock(&self, rid: usize) {
        let process = current_process();
        let process_inner = process.inner_exclusive_access();
        if self.res_type == ResourceType::Mutex {
            process_inner.mutex_deadlock_detector.unlock(
                current_tid(),
                rid,
            );
        } else if self.res_type == ResourceType::Semaphore {
            process_inner.semaphore_deadlock_detector.unlock(
                current_tid(),
                rid,
            );
        }
    }
    /// check if the system is in deadlock
    pub fn check_deadlock(&self) -> bool {
        let process = current_process();
        let process_inner = process.inner_exclusive_access();
        if process_inner.enable_deadlock_detect {
            if self.res_type == ResourceType::Mutex {
                process_inner.mutex_deadlock_detector.check_deadlock()
            } else if self.res_type == ResourceType::Semaphore {
                process_inner.semaphore_deadlock_detector.check_deadlock()
            } else {
                false
            }
        } else {
            false
        }
    }
}

lazy_static! {
    /// DEADLOCK_DETECTOR_MANAGER instance through lazy_static!
    pub static ref DEADLOCK_DETECTOR_MANAGER: UPSafeCell<DeadlockDetectorManager> =
        unsafe { UPSafeCell::new(DeadlockDetectorManager::new()) };
}