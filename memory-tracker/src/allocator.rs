use arr_macro::arr;
use log::info;
use std::alloc::{GlobalAlloc, Layout};
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};

const COUNTERS_SIZE: usize = 16384;
static JEMALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
static MEM_SIZE: [AtomicUsize; COUNTERS_SIZE as usize] = arr![AtomicUsize::new(0); 16384];
static MEM_CNT: [AtomicUsize; COUNTERS_SIZE as usize] = arr![AtomicUsize::new(0); 16384];
static ENABLED: AtomicUsize = AtomicUsize::new(0);
static GTID: AtomicUsize = AtomicUsize::new(0);
static SANITY: AtomicUsize = AtomicUsize::new(0);

pub struct MyAllocator;

thread_local! {
    pub static TID: RefCell<usize> = RefCell::new(usize::max_value());
    pub static TID2: RefCell<usize> = RefCell::new(usize::max_value());
}

pub fn get_tid() -> usize {
    SANITY.fetch_add(1, Ordering::SeqCst);
    let res = TID.with(|t| {
        if *t.borrow() == usize::max_value() {
            *t.borrow_mut() = GTID.fetch_add(1, Ordering::SeqCst);
        }
        *t.borrow()
    });
    SANITY.fetch_sub(1, Ordering::SeqCst);
    res
}

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        SANITY.fetch_add(1, Ordering::SeqCst);

        let new_layout = Layout::from_size_align(layout.size() + 2, layout.align()).unwrap();

        let tid = get_tid();

        let res = JEMALLOC.alloc(new_layout);
        MEM_SIZE[tid % COUNTERS_SIZE].fetch_add(layout.size(), Ordering::SeqCst);
        MEM_CNT[tid % COUNTERS_SIZE].fetch_add(1, Ordering::SeqCst);

        *res.offset(layout.size() as isize) = (tid % 256) as u8;
        *res.offset((layout.size() + 1) as isize) = ((tid / 256) % 256) as u8;
        SANITY.fetch_sub(1, Ordering::SeqCst);
        res
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        SANITY.fetch_add(1, Ordering::SeqCst);
        let new_layout = Layout::from_size_align(layout.size() + 2, layout.align()).unwrap();

        let tid: usize = *ptr.offset(layout.size() as isize) as usize
            + (*ptr.offset((layout.size() + 1) as isize) as usize) * 256;
        MEM_SIZE[tid % COUNTERS_SIZE].fetch_sub(layout.size(), Ordering::SeqCst);
        MEM_CNT[tid % COUNTERS_SIZE].fetch_sub(1, Ordering::SeqCst);

        JEMALLOC.dealloc(ptr, new_layout);
        SANITY.fetch_sub(1, Ordering::SeqCst);
    }
}

pub fn enable_tracking(name: &str) {
    ENABLED.store(1, Ordering::SeqCst);

    TID2.with(|t| {
        if *t.borrow() == usize::max_value() {
            let tid = get_tid();
            info!("enabling tracking for {}: {}", name, tid);
            *t.borrow_mut() = tid;
        }
    });
}

pub fn print_counters_ary() {
    info!("HMM");
    info!("tid {}", get_tid());
    let mut total_cnt: usize = 0;
    let mut total_size: usize = 0;
    for idx in 0..COUNTERS_SIZE {
        let val: usize = MEM_SIZE.get(idx).unwrap().load(Ordering::SeqCst);
        if val != 0 {
            let cnt = MEM_CNT.get(idx).unwrap().load(Ordering::SeqCst);
            total_cnt += cnt;
            info!("COUNTERS {}: {} {}", idx, cnt, val);
            total_size += val;
        }
    }
    info!("COUNTERS TOTAL {} {}", total_cnt, total_size);
}

pub fn get_sanity_val() -> usize {
    SANITY.load(Ordering::SeqCst)
}
