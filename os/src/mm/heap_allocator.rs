//! The global allocator
use crate::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::LockedHeap;

#[global_allocator]
/// heap allocator instance
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
/// panic when heap allocation error occurs
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}
/// heap space ([u8; KERNEL_HEAP_SIZE])
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
/// initiate heap allocator
pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

#[cfg(feature = "heap-bench")]
/// benchmark for heap allocator, including fixed-size, mixed-size and batch allocation patterns
pub fn heap_benchmark() {
    use crate::timer::get_time;
    use alloc::alloc::{alloc, dealloc, Layout};
    use core::ptr::null_mut;

    const FIXED_ROUNDS: usize = 8_192;
    const MIXED_ROUNDS: usize = 16_384;
    const LIVE_SLOTS: usize = 128;
    const BATCH_LIVE: usize = 64;
    const BATCH_ROUNDS: usize = 256;
    const MIXED_SIZES: [usize; 6] = [16, 32, 64, 128, 512, 2048];

    fn next_seed(seed: usize) -> usize {
        seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223)
    }

    println!("[heap-bench] start");

    let fixed_layout = Layout::from_size_align(64, 8).unwrap();
    let fixed_start = get_time();
    for round in 0..FIXED_ROUNDS {
        unsafe {
            let ptr = alloc(fixed_layout);
            assert!(!ptr.is_null(), "fixed-size allocation failed");
            ptr.write_bytes((round as u8).wrapping_add(1), fixed_layout.size());
            dealloc(ptr, fixed_layout);
        }
    }
    let fixed_cost = get_time() - fixed_start;

    let mut slots: [*mut u8; LIVE_SLOTS] = [null_mut(); LIVE_SLOTS];
    let mut layouts = [Layout::new::<u8>(); LIVE_SLOTS];
    let mut seed = 0x1234_5678usize;
    let mixed_start = get_time();
    for _ in 0..MIXED_ROUNDS {
        seed = next_seed(seed);
        let slot = seed % LIVE_SLOTS;

        if !slots[slot].is_null() {
            unsafe {
                dealloc(slots[slot], layouts[slot]);
            }
            slots[slot] = null_mut();
        }

        let size = MIXED_SIZES[(seed >> 8) % MIXED_SIZES.len()] + (seed & 0x0f);
        let layout = Layout::from_size_align(size, 8).unwrap();
        unsafe {
            let ptr = alloc(layout);
            assert!(!ptr.is_null(), "mixed allocation failed");
            ptr.write_bytes(seed as u8, size);
            slots[slot] = ptr;
            layouts[slot] = layout;
        }
    }
    for slot in 0..LIVE_SLOTS {
        if !slots[slot].is_null() {
            unsafe {
                dealloc(slots[slot], layouts[slot]);
            }
        }
    }
    let mixed_cost = get_time() - mixed_start;

    let batch_layout = Layout::from_size_align(1024, 8).unwrap();
    let mut batch_slots: [*mut u8; BATCH_LIVE] = [null_mut(); BATCH_LIVE];
    let batch_start = get_time();
    for _ in 0..BATCH_ROUNDS {
        for slot in 0..BATCH_LIVE {
            unsafe {
                let ptr = alloc(batch_layout);
                assert!(!ptr.is_null(), "batch allocation failed");
                batch_slots[slot] = ptr;
            }
        }
        for slot in 0..BATCH_LIVE {
            unsafe {
                dealloc(batch_slots[slot], batch_layout);
            }
            batch_slots[slot] = null_mut();
        }
    }
    let batch_cost = get_time() - batch_start;

    println!(
        "[heap-bench] fixed 64B x {} = {} ticks",
        FIXED_ROUNDS, fixed_cost
    );
    println!(
        "[heap-bench] mixed {} rounds across {} live slots = {} ticks",
        MIXED_ROUNDS, LIVE_SLOTS, mixed_cost
    );
    println!(
        "[heap-bench] batch 1024B x {}*{} = {} ticks",
        BATCH_ROUNDS, BATCH_LIVE, batch_cost
    );
}

#[allow(unused)]
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    extern "C" {
        fn sbss();
        fn ebss();
    }
    let bss_range = sbss as usize..ebss as usize;
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    for (i, val) in v.iter().take(500).enumerate() {
        assert_eq!(*val, i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    println!("heap_test passed!");
}
