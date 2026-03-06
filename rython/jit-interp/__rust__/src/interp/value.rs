use std::fmt;
use std::ops::{Add, Sub, Mul, Div};
use crate::arena::Arena;

const QNAN: u64 = 0x7ff8_0000_0000_0000;
const TAG_MASK: u64 = 0xffff_0000_0000_0000;
const PAYLOAD_MASK: u64 = 0x0000_ffff_ffff_ffff;
const TAG_BOOL: u64 = 0x0001_0000_0000_0000;
const TAG_INT:  u64 = 0x0002_0000_0000_0000;
const TAG_OBJ:  u64 = 0x0003_0000_0000_0000;

#[derive(Clone, Copy)]
pub struct Value(u64);

impl Value {
    pub fn from_bool(b: bool) -> Self { Value(QNAN | TAG_BOOL | (b as u64)) }
    pub fn to_bool(self) -> Option<bool> {
        if (self.0 & QNAN) == QNAN && (self.0 & TAG_MASK) == (QNAN | TAG_BOOL) {
            Some((self.0 & 1) != 0)
        } else { None }
    }
    pub fn from_int(i: i64) -> Self {
        Value(QNAN | TAG_INT | ((i as u64) & PAYLOAD_MASK))
    }
    pub fn to_int(self) -> Option<i64> {
        if (self.0 & QNAN) == QNAN && (self.0 & TAG_MASK) == (QNAN | TAG_INT) {
            let i = (self.0 & PAYLOAD_MASK) as i64;
            Some((i << 16) >> 16)
        } else { None }
    }
    pub fn from_f64(f: f64) -> Self { Value(f.to_bits()) }
    pub fn to_f64(self) -> Option<f64> {
        if (self.0 & QNAN) != QNAN { Some(f64::from_bits(self.0)) } else { None }
    }
    pub fn from_ptr<T>(ptr: *mut T) -> Self {
        let ptr_bits = ptr as u64;
        assert!(ptr_bits & !PAYLOAD_MASK == 0);
        Value(QNAN | TAG_OBJ | ptr_bits)
    }
    pub fn to_ptr<T>(self) -> Option<*mut T> {
        if (self.0 & QNAN) == QNAN && (self.0 & TAG_MASK) == (QNAN | TAG_OBJ) {
            Some((self.0 & PAYLOAD_MASK) as *mut T)
        } else { None }
    }
    pub fn from_string_in_arena(s: &str, arena: &mut Arena) -> Self {
        let len = s.len();
        let _total_size = std::mem::size_of::<usize>() + len;
        let ptr = arena.alloc_slice(&[]);
        let ptr = ptr as *mut u8;
        unsafe {
            *(ptr as *mut usize) = len;
            let data_ptr = ptr.add(std::mem::size_of::<usize>());
            std::ptr::copy_nonoverlapping(s.as_ptr(), data_ptr, len);
        }
        Self::from_ptr(ptr as *mut ())
    }
    pub fn to_string_from_arena(self) -> Option<String> {
        if let Some(ptr) = self.to_ptr::<()>() {
            unsafe {
                let len_ptr = ptr as *const usize;
                let len = *len_ptr;
                let data_ptr = (ptr as *const u8).add(std::mem::size_of::<usize>());
                let slice = std::slice::from_raw_parts(data_ptr, len);
                Some(String::from_utf8_lossy(slice).to_string())
            }
        } else { None }
    }
    pub fn nil() -> Self { Value(0) }
    pub fn is_nil(self) -> bool { self.0 == 0 }
    pub fn lt(self, rhs: Self) -> Self {
        if let (Some(a), Some(b)) = (self.to_int(), rhs.to_int()) {
            Value::from_bool(a < b)
        } else { panic!("Type error in <"); }
    }
}

impl Add for Value {
    type Output = Value;
    fn add(self, rhs: Self) -> Self::Output {
        match (self.to_int(), rhs.to_int()) {
            (Some(a), Some(b)) => Value::from_int(a + b),
            _ => panic!("Type error in +"),
        }
    }
}
impl Sub for Value {
    type Output = Value;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self.to_int(), rhs.to_int()) {
            (Some(a), Some(b)) => Value::from_int(a - b),
            _ => panic!("Type error in -"),
        }
    }
}
impl Mul for Value {
    type Output = Value;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self.to_int(), rhs.to_int()) {
            (Some(a), Some(b)) => Value::from_int(a * b),
            _ => panic!("Type error in *"),
        }
    }
}
impl Div for Value {
    type Output = Value;
    fn div(self, rhs: Self) -> Self::Output {
        match (self.to_int(), rhs.to_int()) {
            (Some(a), Some(b)) => {
                if b == 0 { panic!("Division by zero"); }
                Value::from_int(a / b)
            }
            _ => panic!("Type error in /"),
        }
    }
}
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_nil() { write!(f, "nil") }
        else if let Some(b) = self.to_bool() { write!(f, "{}", b) }
        else if let Some(i) = self.to_int() { write!(f, "{}", i) }
        else if let Some(x) = self.to_f64() { write!(f, "{}", x) }
        else if let Some(s) = self.to_string_from_arena() { write!(f, "{}", s) }
        else { write!(f, "Value({:#x})", self.0) }
    }
}