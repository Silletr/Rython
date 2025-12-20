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

/// Print an integer to the console.
#[no_mangle]
pub unsafe extern "C" fn rython_print_int(i: i64) {
    println!("{}", i);
}

/// Print a float to the console.
#[no_mangle]
pub unsafe extern "C" fn rython_print_float(f: f64) {
    println!("{}", f);
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

// Math built-ins
#[no_mangle]
pub extern "C" fn rython_add(a: i64, b: i64) -> i64 { a + b }

#[no_mangle]
pub extern "C" fn rython_minus(a: i64, b: i64) -> i64 { a - b }

#[no_mangle]
pub extern "C" fn rython_multiply(a: i64, b: i64) -> i64 { a * b }

#[no_mangle]
pub extern "C" fn rython_divide(a: i64, b: i64) -> i64 { if b != 0 { a / b } else { 0 } }

#[no_mangle]
pub extern "C" fn rython_fibonacci(n: i64) -> i64 {
    if n <= 1 { return n; }
    let mut a = 0i64;
    let mut b = 1i64;
    for _ in 2..=n {
        let temp = a.wrapping_add(b);
        a = b;
        b = temp;
    }
    b
}
