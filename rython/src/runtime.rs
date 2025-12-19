use libc::{size_t, c_void};

// External declarations for Boehm GC
#[link(name = "gc")]
extern "C" {
    pub fn GC_init();
    pub fn GC_malloc(size: size_t) -> *mut c_void;
    pub fn GC_gcollect();
}

/// Initialize the Garbage Collector.
#[no_mangle]
pub unsafe extern "C" fn rython_mem_init() {
    GC_init();
}

/// Allocate memory using the Garbage Collector.
#[no_mangle]
pub unsafe extern "C" fn rython_malloc(size: size_t) -> *mut c_void {
    GC_malloc(size)
}

/// Print a string to the console.
#[no_mangle]
pub unsafe extern "C" fn rython_print_str(s: *const libc::c_char) {
    if !s.is_null() {
        let c_str = std::ffi::CStr::from_ptr(s);
        if let Ok(str_slice) = c_str.to_str() {
            println!("{}", str_slice);
        }
    }
}
