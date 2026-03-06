pub struct Arena {
    memory: Vec<u8>,
    capacity: usize,
    bump: usize,
    pub vm_ptr: *mut crate::vm::VM,
}

impl Arena {
    pub fn new(capacity: usize, vm_ptr: *mut crate::vm::VM) -> Self {
        let mut memory = Vec::with_capacity(capacity);
        unsafe { memory.set_len(capacity); }
        Arena { memory, capacity, bump: 0, vm_ptr }
    }

    pub fn alloc<T>(&mut self, value: T) -> *mut T {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        let start = (self.bump + align - 1) & !(align - 1);
        assert!(start + size <= self.capacity);
        self.bump = start + size;
        let ptr = self.memory.as_mut_ptr().wrapping_add(start) as *mut T;
        unsafe { ptr.write(value); }
        ptr
    }

    pub fn alloc_slice(&mut self, bytes: &[u8]) -> *mut u8 {
        let len = bytes.len();
        let start = self.bump;
        assert!(start + len <= self.capacity);
        self.bump = start + len;
        let ptr = self.memory.as_mut_ptr().wrapping_add(start);
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
        }
        ptr
    }

    pub fn reset(&mut self) { self.bump = 0; }
}