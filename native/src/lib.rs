//! Desktop-only C ABI. Firmware links the no_std core directly (RMK) or adds
//! target-specific staticlib/panic glue (QMK), without this std crate.
mod plover;
use mejiro_engine::{State, translate};

use mejiro_engine::ffi::CState as NativeState;
/// ABI version for adapters to reject incompatible binaries.
#[unsafe(no_mangle)]
pub extern "C" fn mejiro_abi_version() -> u32 {
    1
}

/// Pure preview. UTF-8 body is NUL terminated. All output arguments remain
/// unchanged on error. Returns 0 on success, -1 invalid input, -2 capacity,
/// -3 control request (use Session in the core for runtime controls).
///
/// # Safety
/// Pointers must reference valid, non-overlapping memory of stated lengths;
/// output parameters must be writable. `stroke` is not NUL terminated.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mejiro_translate(
    stroke: *const u8,
    stroke_len: usize,
    state: *const NativeState,
    include_commands: bool,
    output: *mut u8,
    capacity: usize,
    next_state: *mut NativeState,
    output_context: *mut u8,
) -> i32 {
    if stroke.is_null()
        || state.is_null()
        || output.is_null()
        || next_state.is_null()
        || output_context.is_null()
    {
        return -1;
    }
    let raw = match core::str::from_utf8(unsafe { core::slice::from_raw_parts(stroke, stroke_len) })
    {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let Ok(state) = State::try_from(unsafe { *state }) else {
        return -1;
    };
    let t = match translate(raw, state, include_commands) {
        Ok(t) => t,
        Err(mejiro_engine::Error::Capacity) => return -2,
        Err(_) => return -1,
    };
    if t.request != mejiro_engine::Request::Output {
        return -3;
    }
    let body = match plover::plover_body(&t.output) {
        Ok(s) => s,
        Err(_) => return -2,
    };
    let text = body.as_str().as_bytes();
    if text.len() + 1 > capacity {
        return -2;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(text.as_ptr(), output, text.len());
        *output.add(text.len()) = 0;
        *next_state = t.next.into();
        *output_context = t.output.context as u8;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_capacity_errors_do_not_write_partial_state() {
        let raw = b"K-T";
        let state = NativeState::default();
        let mut next = NativeState { vowel: 14, ..state };
        let mut context = 255;
        let mut bytes = [0x7f; 2];
        let rc = unsafe {
            mejiro_translate(
                raw.as_ptr(),
                raw.len(),
                &state,
                false,
                bytes.as_mut_ptr(),
                bytes.len(),
                &mut next,
                &mut context,
            )
        };
        assert_eq!(rc, -2);
        assert_eq!(bytes, [0x7f; 2]);
        assert_eq!(next.vowel, 14);
        assert_eq!(context, 255);
    }
}
