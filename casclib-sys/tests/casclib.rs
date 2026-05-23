extern crate casclib_sys;
extern crate libc;

// Hardcodes a Windows path and calls raw FFI directly; LPCTSTR's underlying
// type is platform-dependent (`*const u16` on Windows with UNICODE,
// `*const i8` on linux), so this test only makes sense on Windows. Gated
// at compile time rather than #[ignore]'d so it doesn't blow up on linux.
#[cfg(target_os = "windows")]
#[test]
#[ignore = "requires a local Warcraft III install at the hardcoded path"]
fn test_wc3() {
    use std::ptr;
    // build.rs forces UNICODE on the C build, so CascOpenStorage on Windows
    // takes LPCTSTR = *const u16. ASCII-only path: widen each byte to a
    // u16 and NUL-terminate.
    let path: Vec<u16> = r#"C:\Program Files (x86)\Warcraft III\Data"#
        .bytes()
        .map(u16::from)
        .chain(std::iter::once(0))
        .collect();
    let mut handle: casclib_sys::HANDLE = ptr::null_mut();

    // CascOpenStorage
    unsafe {
        let result =
            casclib_sys::CascOpenStorage(path.as_ptr(), 0, &mut handle as *mut casclib_sys::HANDLE);
        assert!(result);
    }

    let mut info: u32 = 0;

    let result = unsafe {
        casclib_sys::CascGetStorageInfo(
            handle,
            casclib_sys::_CASC_STORAGE_INFO_CLASS_CascStorageTotalFileCount,
            &mut info as *mut u32 as *mut std::os::raw::c_void,
            4,
            ptr::null_mut(),
        )
    };

    assert!(result);
    assert!(info > 0);

    let result = unsafe {
        casclib_sys::CascGetStorageInfo(
            handle,
            casclib_sys::_CASC_STORAGE_INFO_CLASS_CascStorageFeatures,
            &mut info as *mut u32 as *mut std::os::raw::c_void,
            4,
            ptr::null_mut(),
        )
    };

    assert!(result);
}
