//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.
const SYSCALL_WRITE: usize = 64;
/// exit syscall
const SYSCALL_EXIT: usize = 93;
/// yield syscall
const SYSCALL_YIELD: usize = 124;
/// gettime syscall
const SYSCALL_GET_TIME: usize = 169;
/// sbrk syscall
const SYSCALL_SBRK: usize = 214;
/// munmap syscall
const SYSCALL_MUNMAP: usize = 215;
/// mmap syscall
const SYSCALL_MMAP: usize = 222;
/// trace syscall
const SYSCALL_TRACE: usize = 410;

mod fs;
mod process;

use fs::*;
use process::*;

use crate::task::trace_current_syscall_count;

#[derive(Copy, Clone, Debug)]
/// enum for different syscall kinds
pub enum SyscallKind {
    /// sys_write syscall
    Write,
    /// sys_exit syscall
    Exit,
    /// sys_yield syscall
    Yield,
    /// sys_get_time syscall
    GetTime,
    /// sys_sbrk syscall
    Sbrk,
    /// sys_munmap syscall
    Munmap,
    /// sys_mmap syscall
    Mmap,
    /// sys_trace syscall
    Trace,
}

impl SyscallKind {
    /// the number of syscall kinds
    pub const COUNT: usize = 8;
    /// get syscall kind from syscall number
    pub fn from_syscall_number(n: usize) -> Option<Self> {
        match n {
            SYSCALL_WRITE => Some(Self::Write),
            SYSCALL_EXIT => Some(Self::Exit),
            SYSCALL_YIELD => Some(Self::Yield),
            SYSCALL_GET_TIME => Some(Self::GetTime),
            SYSCALL_SBRK => Some(Self::Sbrk),
            SYSCALL_MUNMAP => Some(Self::Munmap),
            SYSCALL_MMAP => Some(Self::Mmap),
            SYSCALL_TRACE => Some(Self::Trace),
            _ => None,
        }
    }
    /// get index of syscall kind for syscall count array
    pub fn as_index(self) -> usize {
        match self {
            Self::Write => 0,
            Self::Exit => 1,
            Self::Yield => 2,
            Self::GetTime => 3,
            Self::Sbrk => 4,
            Self::Munmap => 5,
            Self::Mmap => 6,
            Self::Trace => 7,
        }
    }
}

/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    trace_current_syscall_count(syscall_id);
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TRACE => sys_trace(args[0], args[1], args[2]),
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2]),
        SYSCALL_MUNMAP => sys_munmap(args[0], args[1]),
        SYSCALL_SBRK => sys_sbrk(args[0] as i32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
