﻿use buddy_allocator::{BuddyAllocator, LinkedListBuddy, UsizeBuddy};
use std::{
    alloc::Layout,
    ptr::{null_mut, NonNull},
    time::Instant,
};

type Allocator<const N: usize> = BuddyAllocator<N, UsizeBuddy, LinkedListBuddy>;

#[repr(C, align(4096))]
struct Page([u8; 4096]);

impl Page {
    const ZERO: Self = Self([0; 4096]);
}

/// 8 MiB
static mut MEMORY: [Page; 2048] = [Page::ZERO; 2048];

fn main() {
    let mut allocator = Allocator::<5>::new();
    let ptr = NonNull::new(unsafe { MEMORY.as_mut_ptr() }).unwrap();
    let len = core::mem::size_of_val(unsafe { &MEMORY });
    allocator.init(12, ptr);
    println!(
        "MEMORY: {:#x}..{:#x}",
        ptr.as_ptr() as usize,
        ptr.as_ptr() as usize + len
    );
    let t = Instant::now();
    unsafe { allocator.transfer(ptr, len) };
    println!("transfer {:?}", t.elapsed());

    assert_eq!(len, allocator.capacity());
    assert_eq!(len, allocator.free());

    println!(
        "
BEFORE
{allocator:#x?}"
    );

    let mut blocks = [null_mut::<Page>(); 1024];
    let layout = Layout::new::<Page>();
    let t = Instant::now();
    for block in blocks.iter_mut() {
        let (ptr, size) = allocator.allocate(layout).unwrap();
        debug_assert_eq!(layout.size(), size);
        *block = ptr.as_ptr();
    }
    let t = t.elapsed();
    println!("allocate   {t:?} ({} times)", blocks.len());

    assert_eq!(len, allocator.capacity());
    assert_eq!(len - blocks.len() * layout.size(), allocator.free());

    let t = Instant::now();
    for block in blocks.iter_mut() {
        allocator.deallocate(NonNull::new(*block).unwrap(), layout.size());
        *block = null_mut();
    }
    let t = t.elapsed();
    println!("deallocate {t:?} ({} times)", blocks.len());

    assert_eq!(len, allocator.capacity());
    assert_eq!(len, allocator.free());

    println!(
        "
AFTER
{allocator:#x?}"
    );
}
