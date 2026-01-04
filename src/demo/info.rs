mod info_struct;
pub mod ops;
use super::{Byte, NativeInt};


// the specific static variable storing the info at a specific place in memory
static mut INFO_STATE: info_struct::Info = info_struct::Info::new();

// the unsafe "swiss army knife function"
#[allow(static_mut_refs)]
pub(crate) fn process_info<F, K>(f: F) -> K
where
    F: FnOnce(&mut info_struct::Info) -> K,
{
    unsafe { f(&mut INFO_STATE) }
}

// returns pointer to and byte length of a vector
pub(crate) fn ptr_and_len<T>(v: &'static T) -> (NativeInt, NativeInt)
where
    Vec<Byte>: From<&'static T>,
{
    let bytes: Vec<Byte> = v.into();
    (bytes.as_ptr() as NativeInt, bytes.len() as NativeInt)
}
