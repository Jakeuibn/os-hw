//! Semaphore

use crate::sync::UPSafeCell;
use crate::sync::deadlock::{DEADLOCK_DETECTOR_MANAGER, ResourceType};
use crate::task::{block_current_and_run_next, current_task, wakeup_task, TaskControlBlock};
use alloc::{collections::VecDeque, sync::Arc};

/// semaphore structure
pub struct Semaphore {
    /// resource id of the semaphore
    pub rid: usize,
    /// semaphore inner
    pub inner: UPSafeCell<SemaphoreInner>,
}

pub struct SemaphoreInner {
    pub count: isize,
    pub wait_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl Semaphore {
    /// Create a new semaphore
    pub fn new(rid:usize, res_count: usize) -> Self {
        trace!("kernel: Semaphore::new");
        Self {
            rid,
            inner: unsafe {
                UPSafeCell::new(SemaphoreInner {
                    count: res_count as isize,
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }

    /// up operation of semaphore
    pub fn up(&self) {
        trace!("kernel: Semaphore::up");
        DEADLOCK_DETECTOR_MANAGER.exclusive_access().set_res_type(ResourceType::Semaphore);
        DEADLOCK_DETECTOR_MANAGER.exclusive_access().unlock(self.get_rid());
        let mut inner = self.inner.exclusive_access();
        inner.count += 1;
        if inner.count <= 0 {
            if let Some(task) = inner.wait_queue.pop_front() {
                wakeup_task(task);
            }
        }
    }

    /// down operation of semaphore
    pub fn down(&self) -> bool {
        trace!("kernel: Semaphore::down");
        DEADLOCK_DETECTOR_MANAGER.exclusive_access().set_res_type(ResourceType::Semaphore);
        DEADLOCK_DETECTOR_MANAGER.exclusive_access().need(self.get_rid());
        if DEADLOCK_DETECTOR_MANAGER.exclusive_access().check_deadlock() {
            DEADLOCK_DETECTOR_MANAGER.exclusive_access().noneed(self.get_rid());
            return false;
        }
        let mut inner = self.inner.exclusive_access();
        inner.count -= 1;
        if inner.count < 0 {
            inner.wait_queue.push_back(current_task().unwrap());
            drop(inner);
            block_current_and_run_next();
        }
        DEADLOCK_DETECTOR_MANAGER.exclusive_access().set_res_type(ResourceType::Semaphore);
        DEADLOCK_DETECTOR_MANAGER.exclusive_access().lock(self.get_rid());
        if DEADLOCK_DETECTOR_MANAGER.exclusive_access().check_deadlock() {
            DEADLOCK_DETECTOR_MANAGER.exclusive_access().unlock(self.get_rid());
            let mut inner = self.inner.exclusive_access();
            inner.count += 1;
            return false;
        }
        true
    }
    /// get resource id of the semaphore
    pub fn get_rid(&self) -> usize {
        self.rid
    }
}
