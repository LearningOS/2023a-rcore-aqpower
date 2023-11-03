//! Process management syscalls

use core::mem::size_of;

use crate::{
    config::MAX_SYSCALL_NUM,
    mm::{translated_byte_buffer, MapPermission, VirtAddr},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_current_task,
        get_task_bucket, get_task_first_dispatch_time, insert_area, suspend_current_and_run_next,
        TaskStatus,
    },
    timer::{get_time_ms, get_time_us},
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let buffers =
        translated_byte_buffer(current_user_token(), _ts as *const u8, size_of::<TimeVal>());
    let us = get_time_us();
    for buffer in buffers {
        let time_val_ptr: *mut TimeVal = buffer.as_mut_ptr() as *mut TimeVal;
        unsafe {
            let time_val = &mut *time_val_ptr;
            time_val.sec = us / 1_000_000;
            time_val.usec = us % 1_000_000;
        }
        break;
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let current_task = get_current_task();
    let bucket = get_task_bucket(current_task);
    let first_time = get_task_first_dispatch_time(current_task);
    let time_distance = get_time_ms() - first_time;
    unsafe {
        *_ti = TaskInfo {
            status: TaskStatus::Running,
            syscall_times: bucket,
            time: time_distance,
        };
    }
    let buffers = translated_byte_buffer(
        current_user_token(),
        _ti as *const u8,
        size_of::<TaskInfo>(),
    );
    for buffer in buffers {
        let task_info_ptr: *mut TaskInfo = buffer.as_mut_ptr() as *mut TaskInfo;
        unsafe {
            let task_info = &mut *task_info_ptr;
            task_info.status = TaskStatus::Running;
            task_info.syscall_times = bucket;
            task_info.time = time_distance;
        }
        break;
    }
    0
}

// YOUR JOB: Implement mmap.
// 申请长度为 len 字节的物理内存（不要求实际物理内存位置，可以随便找一块）
// 将其映射到 start 开始的虚存，内存页属性为 port
/**
 * 参数：
 * start 需要映射的虚存起始地址，要求按页对齐
 * len 映射字节长度，可以为 0
 * port：第 0 位表示是否可读，第 1 位表示是否可写，第 2 位表示是否可执行。其他位无效且必须为 0
 */
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");
    debug!("start: {:#b}, len: {}", _start, _len);
    let virt_add = VirtAddr::from(_start);
    if virt_add.aligned() == false || _port & !0x7 != 0 || _port & 0x7 == 0 || _len != 4096{
        debug!("error1 port");
        return -1;
    }
    let mut map_p = MapPermission::U;
    if (_port & 0b0001) != 0 {
        map_p = map_p | MapPermission::R;
    }
    if (_port & 0b0010) != 0 {
        map_p = map_p | MapPermission::W;
    }
    if (_port & 0b0100) != 0 {
        map_p = map_p | MapPermission::X;
    }

    insert_area(virt_add, VirtAddr::from(_start + _len), map_p);
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    
    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
