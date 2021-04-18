use volatile::Volatile;
use core::cell::{Cell, RefCell};
use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use alloc::vec;
use crate::asm;
use spin::Mutex;

const EFLAGS_AC_BIT: u32 = 0x00040000;
const CR0_CACHE_DISABLE: u32 = 0x60000000;

pub fn memtest(start: u32, end: u32) -> u32 {
    let mut flg486 = false;
    asm::store_eflags((asm::load_eflags() as u32 | EFLAGS_AC_BIT) as i32);
    let mut eflags = asm::load_eflags() as u32;
    // 386ではAC=1にしても自動で0に戻ってしまう
    if eflags & EFLAGS_AC_BIT != 0 {
        flg486 = true;
    }
    eflags &= !EFLAGS_AC_BIT;
    asm::store_eflags(eflags as i32);

    if flg486 {
        // キャッシュ禁止
        let cr0 = asm::load_cr0() | CR0_CACHE_DISABLE;
        asm::store_cr0(cr0);
    }

    let memory = memtest_main(start, end);

    if flg486 {
        // キャッシュ許可
        let mut cr0 = asm::load_cr0();
        cr0 &= !CR0_CACHE_DISABLE;
        asm::store_cr0(cr0);
    }

    memory
}

fn memtest_main(start: u32, end: u32) -> u32 {
    let pat0: u32 = 0xaa55aa55;
    let pat1: u32 = 0x55aa55aa;
    let mut r = start;
    for i in (start..end).step_by(0x1000) {
        r = i;
        let mp = (i + 0xffc) as *mut u32;
        let p = unsafe { &mut *(mp as *mut Volatile<u32>) };
        let old = p.read();
        p.write(pat0);
        p.write(!p.read());
        if p.read() != pat1 {
            p.write(old);
            break;
        }
        p.write(!p.read());
        if p.read() != pat0 {
            p.write(old);
            break;
        }
        p.write(old);
    }
    r
}

const MEMMAN_FREES: u32 = 4090; // 約32KB
pub const MEMMAN_ADDR: u32 = 0x003c0000;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C, packed)]
struct FreeInfo {
    addr: u32,
    size: u32,
}



// #[derive(Clone, Copy)]
// #[repr(C, packed)]
pub struct MemMan {
    notifier: Mutex<()>,
    frees: Cell<u32>,
    maxfrees: Cell<u32>,
    lostsize: Cell<u32>,
    losts: Cell<u32>,
    free: RefCell<[FreeInfo; MEMMAN_FREES as usize]>,
}

unsafe impl GlobalAlloc for MemMan {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // pub fn alloc(&self, size: u32) -> Result<u32, &'static str>
        // 1. Compare alloc interfaces between MemMan and GlobalAlloc
        // layout has size info: layout.size <-> size in MemMan.alloc's size arg
        // MamMan.alloc: address of allocated memory. If failed, return string.
        // GlobalAlloc: address of allocated memory. If failed, just panic.
        // todo: cast u32 to *mut u8
        let addr = self.alloc_by_size(layout.size() as u32);
        // todo: cast u32 to *mut u8
        addr.unwrap() as *mut u8 // mutable なu8 へのポインタ
    }

    unsafe fn dealloc(&self, addr: *mut u8, layout: Layout) {
        self.free(addr as u32, layout.size() as u32).unwrap();
    }
}

#[global_allocator]
pub static memman: MemMan = MemMan::new(); // static 関数みたいなもん

unsafe impl Sync for MemMan {} // コンパイラにスレッドセーフだよと教えてあげる（嘘でもよい）。MarkerTraitという。

impl MemMan {
    pub const fn new() -> MemMan {
        MemMan {
            notifier: Mutex::new(()), // 中身が空でいい時
            frees: Cell::new(0),
            maxfrees: Cell::new(0),
            lostsize: Cell::new(0),
            losts: Cell::new(0),
            free: RefCell::new([FreeInfo { addr: 0, size: 0 }; MEMMAN_FREES as usize]),
        }
    }

    pub fn total(&self) -> u32 {
        let res = self.notifier.lock();
        let mut t = 0;
        let free = self.free.borrow();
        for i in 0..self.frees.get() {
            t += free[i as usize].size;
        }
        t
    }

    pub fn alloc_by_size(&self, size: u32) -> Result<u32, &'static str> {
        let res = self.notifier.lock();
        let mut free = self.free.borrow_mut();
        for i in 0..self.frees.get() {
            let i = i as usize;
            if free[i].size >= size {
                let a = free[i].addr; // address
                free[i].addr += size;
                free[i].size -= size;
                // self.free[i].addr += size;
                // self.free[i].size -= size;
                if free[i].size == 0 {
                    // self.frees -= 1;
                    // self.free[i] = self.free[i + 1]
                    self.frees.set(self.frees.get() - 1);
                    free[i] = free[i + 1];
                }
                return Ok(a);
            }
        }
        Err("CANNOT ALLOCATE MEMORY")
    }

    pub fn free(&self, addr: u32, size: u32) -> Result<(), &'static str> {
        let res = self.notifier.lock();
        let mut idx: usize = 0;
        let mut free = self.free.borrow_mut();
        // addrの順に並ぶように、insertすべきindexを決める
        for i in 0..self.frees.get() {
            let i = i as usize;
            if free[i].addr > addr {
                idx = i;
                break;
            }
        }
        if idx > 0 {
            if free[idx - 1].addr + free[idx - 1].size == addr {
                free[idx - 1].size += size;
                if idx < self.frees.get() as usize {
                    if addr + size == free[idx].addr {
                        free[idx - 1].size += free[idx].size;
                    }
                    self.frees.set(self.frees.get() - 1);
                    for i in idx..(self.frees.get() as usize) {
                        free[i] = free[i + 1];
                    }
                }
                return Ok(());
            }
        }
        if idx < self.frees.get() as usize {
            if addr + size == free[idx].addr {
                free[idx].addr = addr;
                free[idx].size += size;
                return Ok(());
            }
        }
        if self.frees.get() < MEMMAN_FREES {
            let mut j = self.frees.get() as usize;
            while j > idx {
                free[j] = free[j - 1];
                j -= 1;
            }
            self.frees.set(self.frees.get() + 1);
            if self.maxfrees.get() < self.frees.get() {
                self.maxfrees.set(self.frees.get());
            }
            free[idx].addr = addr;
            free[idx].size = size;
            return Ok(());
        }
        self.losts.set(self.losts.get() + 1);
        self.lostsize.set(self.lostsize.get() + size);
        Err("CANNOT FREE MEMORY")
    }

    pub fn alloc_4k(&self, size: u32) -> Result<u32, &'static str> {
        let size = (size + 0xfff) & 0xfffff000;
        self.alloc_by_size(size)
    }

    pub fn free_4k(&self, addr: u32, size: u32) -> Result<(), &'static str> {
        let size = (size + 0xfff) & 0xfffff000;
        self.free(addr, size)
    }
}