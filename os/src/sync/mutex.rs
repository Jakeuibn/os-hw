//! Mutex (spin-like and blocking(sleep))

use super::UPSafeCell;
use crate::sync::deadlock::{DEADLOCK_DETECTOR_MANAGER, ResourceType};
use crate::task::TaskControlBlock;
use crate::task::{block_current_and_run_next, suspend_current_and_run_next, current_task};
use crate::task::{wakeup_task};
use alloc::{collections::VecDeque, sync::Arc};

/// Mutex trait
pub trait Mutex: Sync + Send {
    /// Lock the mutex
    fn lock(&self) -> bool;
    /// Unlock the mutex
    fn unlock(&self);
    /// Get the resource id of the mutex
    fn get_rid(&self) -> usize;
}

/// Spinlock Mutex struct
pub struct MutexSpin {
    rid: usize,
    locked: UPSafeCell<bool>,
}

impl MutexSpin {
    /// Create a new spinlock mutex
    pub fn new(rid: usize) -> Self {
        Self {
            rid,
            locked: unsafe { UPSafeCell::new(false) },
        }
    }
}

impl Mutex for MutexSpin {
    /// Lock the spinlock mutex
    fn lock(&self) -> bool {
        trace!("kernel: MutexSpin::lock");
        DEADLOCK_DETECTOR_MANAGER.exclusive_access().set_res_type(ResourceType::Mutex);
        DEADLOCK_DETECTOR_MANAGER.exclusive_access().need(self.get_rid());
        if DEADLOCK_DETECTOR_MANAGER.exclusive_access().check_deadlock() {
            DEADLOCK_DETECTOR_MANAGER.exclusive_access().noneed(self.get_rid());
            return false;
        }
        loop {
            let mut locked = self.locked.exclusive_access();
            if *locked {
                drop(locked);
                suspend_current_and_run_next();
                continue;
            } else {
                DEADLOCK_DETECTOR_MANAGER.exclusive_access().set_res_type(ResourceType::Mutex);
                DEADLOCK_DETECTOR_MANAGER.exclusive_access().lock(self.get_rid());
                if DEADLOCK_DETECTOR_MANAGER.exclusive_access().check_deadlock() {
                    DEADLOCK_DETECTOR_MANAGER.exclusive_access().unlock(self.get_rid());
                    return false;
                }
                *locked = true;
                return true;
            }
        }
    }

    fn unlock(&self) {
        trace!("kernel: MutexSpin::unlock");
        DEADLOCK_DETECTOR_MANAGER.exclusive_access().set_res_type(ResourceType::Mutex);
        DEADLOCK_DETECTOR_MANAGER.exclusive_access().unlock(self.get_rid());
        let mut locked = self.locked.exclusive_access();
        *locked = false;
    }

    fn get_rid(&self) -> usize {
        self.rid
    }
}

/// Blocking Mutex struct
pub struct MutexBlocking {
    rid: usize,
    inner: UPSafeCell<MutexBlockingInner>,
}

pub struct MutexBlockingInner {
    locked: bool,
    wait_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl MutexBlocking {
    /// Create a new blocking mutex
    pub fn new(rid: usize) -> Self {
        trace!("kernel: MutexBlocking::new");
        Self {
            rid,
            inner: unsafe {
                UPSafeCell::new(MutexBlockingInner {
                    locked: false,
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }
}

impl Mutex for MutexBlocking {
    /// lock the blocking mutex
    fn lock(&self) -> bool {
        trace!("kernel: MutexBlocking::lock");
        DEADLOCK_DETECTOR_MANAGER.exclusive_access().set_res_type(ResourceType::Mutex);
        DEADLOCK_DETECTOR_MANAGER.exclusive_access().need(self.get_rid());
        if DEADLOCK_DETECTOR_MANAGER.exclusive_access().check_deadlock() {
            DEADLOCK_DETECTOR_MANAGER.exclusive_access().noneed(self.get_rid());
            return false;
        }
        let mut mutex_inner = self.inner.exclusive_access();
        if mutex_inner.locked {
            mutex_inner.wait_queue.push_back(current_task().unwrap());
            drop(mutex_inner);
            block_current_and_run_next();
            DEADLOCK_DETECTOR_MANAGER.exclusive_access().set_res_type(ResourceType::Mutex);
            DEADLOCK_DETECTOR_MANAGER.exclusive_access().lock(self.get_rid());
            if DEADLOCK_DETECTOR_MANAGER.exclusive_access().check_deadlock() {
                DEADLOCK_DETECTOR_MANAGER.exclusive_access().unlock(self.get_rid());
                let mut mutex_inner = self.inner.exclusive_access();
                mutex_inner.locked = false;
                return false;
            }
        } else {
            DEADLOCK_DETECTOR_MANAGER.exclusive_access().set_res_type(ResourceType::Mutex);
            DEADLOCK_DETECTOR_MANAGER.exclusive_access().lock(self.get_rid());
            if DEADLOCK_DETECTOR_MANAGER.exclusive_access().check_deadlock() {
                DEADLOCK_DETECTOR_MANAGER.exclusive_access().unlock(self.get_rid());
                return false;
            }
            mutex_inner.locked = true;
        }
        true
    }

    /// unlock the blocking mutex
    fn unlock(&self) {
        trace!("kernel: MutexBlocking::unlock");
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        DEADLOCK_DETECTOR_MANAGER.exclusive_access().set_res_type(ResourceType::Mutex);
        DEADLOCK_DETECTOR_MANAGER.exclusive_access().unlock(self.get_rid());
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            wakeup_task(waking_task);
        } else {
            mutex_inner.locked = false;
        }
    }

    fn get_rid(&self) -> usize {
        self.rid
    }
}
