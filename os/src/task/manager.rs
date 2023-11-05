//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }

    /// Take a process having the least stride out of the ready queue
    pub fn fetch_stride_least_task(&mut self) -> Option<Arc<TaskControlBlock>> {
        let mut mi = 1 << 30;
        let mut ma_prio = 0;
        let mut index = 0;
        for (_index, _proc) in self.ready_queue.iter().enumerate() {
            let stride = _proc.inner_exclusive_access().stride;
            let prio = _proc.inner_exclusive_access().priority;
            debug!("now stride {stride} prio: {prio}");
            if stride < mi {
                mi = stride;
                index = _index;
            } else if stride == mi {
                if prio > ma_prio {
                    ma_prio = prio;
                    index = _index;
                }
            }
        }
        debug!("min index: {index}");
        self.ready_queue.remove(index)
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
/// get a process, however we chose the process have the least stride
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    // trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}

/// Take a process out of the ready queue
/// get a process, however we chose the process have the least stride
pub fn fetch_stride_least_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch_stride_least_task()
}
