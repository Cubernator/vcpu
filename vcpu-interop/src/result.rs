use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::os::raw::c_char;
use util::InteropGetName;
use util_derive::InteropGetName;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, FromPrimitive, InteropGetName)]
pub enum VcpuResult {
    UnknownError = -1,
    Ok = 0,
    InvalidType = 1,
    UTF8Error = 2,
    AssemblerError = 3,
    MemoryInUse = 4,
    FragmentIntersection = 5,
    KeyAlreadyExists = 6,
    OutOfRange = 7,
    ExecutableLoadFailed = 8,
    ExecutableSaveFailed = 9,
}

#[no_mangle]
pub unsafe extern "C" fn vcpu_result_get_description(
    result: i32,
    desc: *mut *const c_char,
) -> VcpuResult {
    if let Some(result) = VcpuResult::from_i32(result) {
        *desc = result.interop_name().as_ptr() as *const c_char;
        VcpuResult::OutOfRange
    } else {
        VcpuResult::OutOfRange
    }
}
