//! Process management syscalls
use crate::{
    mm::{read_translated_byte, translated_byte_buffer, write_translated_byte},
    task::{change_program_brk, current_user_token, exit_current_and_run_next, 
        get_current_syscall_count, insert_mmap_area, delete_mmap_area, 
        suspend_current_and_run_next},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
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
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let tv = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let bytes = unsafe {
        core::slice::from_raw_parts(&tv as *const TimeVal as *const u8, core::mem::size_of::<TimeVal>())
    };
    let buffers = translated_byte_buffer(current_user_token(), ts as *mut u8, core::mem::size_of::<TimeVal>());
    let mut bytes_written = 0;
    for buffer in buffers {
        let len = buffer.len().min(bytes.len() - bytes_written);
        buffer[..len].copy_from_slice(&bytes[bytes_written..bytes_written + len]);
        bytes_written += len;
        if bytes_written >= bytes.len() {
            break;
        }
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => read_translated_byte(current_user_token(), id as *const u8),
        1 => write_translated_byte(current_user_token(), id as *mut u8, data as u8),
        2 => get_current_syscall_count(id) as isize,
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    insert_mmap_area(start, len, prot)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    delete_mmap_area(start, len)
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
