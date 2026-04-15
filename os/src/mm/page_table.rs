//! Implementation of [`PageTableEntry`] and [`PageTable`].

use super::{frame_alloc, FrameTracker, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;

bitflags! {
    /// page table entry flags
    pub struct PTEFlags: u8 {
        /// Valid
        const V = 1 << 0;
        /// Readable
        const R = 1 << 1;
        /// Writable
        const W = 1 << 2;
        /// eXecutable
        const X = 1 << 3;
        /// User
        const U = 1 << 4;
        /// Global
        const G = 1 << 5;
        /// Accessed
        const A = 1 << 6;
        /// Dirty
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
/// page table entry structure
pub struct PageTableEntry {
    /// bits of page table entry
    pub bits: usize,
}

impl PageTableEntry {
    /// Create a new page table entry
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
    /// Create an empty page table entry
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    /// Get the physical page number from the page table entry
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    /// Get the flags from the page table entry
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    /// The page pointered by page table entry is valid?
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    /// The page pointered by page table entry is readable?
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    /// The page pointered by page table entry is writable?
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    /// The page pointered by page table entry is executable?
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
    /// The page pointered by page table entry is user accessible?
    pub fn user(&self) -> bool {
        (self.flags() & PTEFlags::U) != PTEFlags::empty()
    }
}

/// page table structure
pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

/// Assume that it won't oom when creating/mapping.
impl PageTable {
    /// Create a new page table
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }
    /// Temporarily used to get arguments from user space.
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }
    /// Find PageTableEntry by VirtPageNum, create a frame for a 4KB page table if not exist
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }
    /// Find PageTableEntry by VirtPageNum
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }
    /// set the map between virtual page number and physical page number
    #[allow(unused)]
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    /// remove the map between virtual page number and physical page number
    #[allow(unused)]
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }
    /// get the page table entry from the virtual page number
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }
    /// get the token from the page table
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }

    /// Dump current Sv39 page table for debugging.
    pub fn dump(&self) {
        println!(
            "[pt] ===== Sv39 page table dump start, root ppn = {:#x} =====",
            self.root_ppn.0
        );
        dump_page_table_level(self.root_ppn, 2, 0);
        println!("[pt] ===== Sv39 page table dump end =====");
    }

    /// Dump an existing page table by SATP token.
    pub fn dump_from_token(token: usize) {
        Self::from_token(token).dump();
    }
}

fn flag_char(cond: bool, c: char) -> char {
    if cond {
        c
    } else {
        '-'
    }
}

fn dump_flags(flags: PTEFlags) {
    println!(
        "{}{}{}{}{}{}{}{}",
        flag_char(flags.contains(PTEFlags::V), 'V'),
        flag_char(flags.contains(PTEFlags::R), 'R'),
        flag_char(flags.contains(PTEFlags::W), 'W'),
        flag_char(flags.contains(PTEFlags::X), 'X'),
        flag_char(flags.contains(PTEFlags::U), 'U'),
        flag_char(flags.contains(PTEFlags::G), 'G'),
        flag_char(flags.contains(PTEFlags::A), 'A'),
        flag_char(flags.contains(PTEFlags::D), 'D')
    );
}

fn dump_page_table_level(table_ppn: PhysPageNum, level: usize, vpn_prefix: usize) {
    for (idx, pte) in table_ppn.get_pte_array().iter().enumerate() {
        if !pte.is_valid() {
            continue;
        }

        let flags = pte.flags();
        let child_ppn = pte.ppn();
        let next_vpn_prefix = vpn_prefix | (idx << (level * 9));
        let is_leaf = pte.readable() || pte.writable() || pte.executable();

        print!(
            "[pt] L{} idx={:#03x} pte={:#018x} ppn={:#x} flags=",
            level, idx, pte.bits, child_ppn.0
        );
        dump_flags(flags);

        if is_leaf {
            let va_start = next_vpn_prefix << 12;
            let size = 1usize << (12 + level * 9);
            let va_end = va_start + size;
            let pa_start = child_ppn.0 << 12;
            let pa_end = pa_start + size;
            println!(
                "[pt]   -> VA [{:#x}, {:#x}) -> PA [{:#x}, {:#x})",
                va_start, va_end, pa_start, pa_end
            );
        } else if level > 0 {
            dump_page_table_level(child_ppn, level - 1, next_vpn_prefix);
        } else {
            println!("[pt]   -> warning: non-leaf entry at level 0");
        }
    }
}

/// Translate&Copy a ptr[u8] array with LENGTH len to a mutable u8 Vec through page table
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}

/// Translate&Read a single ptr to a mutable u8 through page table
/// The user must be able to access and read it, otherwise -1 is returned.
pub fn read_translated_byte(token: usize, ptr: *const u8) -> isize {
    let page_table = PageTable::from_token(token);
    let va = VirtAddr::from(ptr as usize);
    let vpn = va.floor();
    if let Some(pte) = page_table.translate(vpn) {
        if !pte.user() || !pte.readable() {
            return -1;
        }
        let ppn = pte.ppn();
        ppn.get_bytes_array()[va.page_offset()] as isize
    } else {
        return -1;
    }
}

/// Translate&Write data to a mutable u8 through page table
/// The user must be able to access and write it, otherwise -1 is returned.
pub fn write_translated_byte(token: usize, ptr: *mut u8, value: u8) -> isize {
    let page_table = PageTable::from_token(token);
    let va = VirtAddr::from(ptr as usize);
    let vpn = va.floor();
    if let Some(pte) = page_table.translate(vpn) {
        if !pte.user() || !pte.writable() {
            return -1;
        }
        let ppn = pte.ppn();
        ppn.get_bytes_array()[va.page_offset()] = value;
        0
    } else {
        return -1;
    }
}

/// check that no page table entry in the range [start_va, end_va) is valid
pub fn check_vpn_range_no_entry(token: usize, start_va: VirtAddr, end_va: VirtAddr) -> bool {
    let page_table = PageTable::from_token(token);
    let mut vpn = start_va.floor();
    while VirtAddr::from(vpn) < end_va {
        if let Some(pte) = page_table.translate(vpn) {
            if pte.is_valid() {
                return false;
            }
        }
        vpn.step();
    }
    true
}

/// check that all page table entries in the range [start_va, end_va) are valid
pub fn check_vpn_range_all_entry(token: usize, start_va: VirtAddr, end_va: VirtAddr) -> bool {
    let page_table = PageTable::from_token(token);
    let mut vpn = start_va.floor();
    while VirtAddr::from(vpn) < end_va {
        if let Some(pte) = page_table.translate(vpn) {
            if !pte.is_valid() {
                return false;
            }
        } else {
            return false;
        }
        vpn.step();
    }
    true
}