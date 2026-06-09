use std::ffi::c_char;

const NUL: u8 = 0;

#[inline(always)]
fn ascii_isalpha(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

#[inline(always)]
#[allow(dead_code)]
fn vim_ispathsep_nocolon(c: u8) -> bool {
    #[cfg(unix)]
    {
        c == b'/'
    }
    #[cfg(not(unix))]
    {
        c == b'/' || c == b'\\'
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_is_absolute(fname: *const c_char) -> bool {
    if fname.is_null() {
        return false;
    }
    let first = unsafe { *fname as u8 };
    if first == NUL {
        return false;
    }

    #[cfg(not(unix))]
    {
        let second = unsafe { *fname.add(1) as u8 };
        if second != NUL {
            let third = unsafe { *fname.add(2) as u8 };
            if ascii_isalpha(first) && second == b':' && vim_ispathsep_nocolon(third) {
                return true;
            }
        }
        vim_ispathsep_nocolon(first)
    }

    #[cfg(unix)]
    {
        first == b'/' || first == b'~'
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_has_drive_letter(p: *const c_char, path_len: usize) -> bool {
    if p.is_null() || path_len < 2 {
        return false;
    }
    let p0 = unsafe { *p as u8 };
    let p1 = unsafe { *p.add(1) as u8 };
    if !ascii_isalpha(p0) || (p1 != b':' && p1 != b'|') {
        return false;
    }
    if path_len == 2 {
        return true;
    }
    let p2 = unsafe { *p.add(2) as u8 };
    p2 == b'/' || p2 == b'\\' || p2 == b'?' || p2 == b'#'
}

const URL_SLASH: i32 = 1;
const URL_BACKSLASH: i32 = 2;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_is_url(p: *const c_char) -> i32 {
    if p.is_null() {
        return 0;
    }
    let first = unsafe { *p as u8 };
    if first == NUL {
        return 0;
    }
    let second = unsafe { *p.add(1) as u8 };
    if second == NUL {
        return 0;
    }

    if first == b':' && second == b'/' {
        return URL_SLASH;
    }

    let third = unsafe { *p.add(2) as u8 };
    if third == NUL {
        return 0;
    }
    if first == b':' && second == b'\\' && third == b'\\' {
        return URL_BACKSLASH;
    }

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_with_url(fname: *const c_char) -> i32 {
    if fname.is_null() {
        return 0;
    }
    let first = unsafe { *fname as u8 };
    if !ascii_isalpha(first) {
        return 0;
    }

    let mut len = 0;
    let mut curr = fname;
    unsafe {
        while *curr != 0 {
            len += 1;
            curr = curr.add(1);
        }
    }

    if unsafe { path_has_drive_letter(fname, len) } {
        return 0;
    }

    let mut p = unsafe { fname.add(1) };
    loop {
        let val = unsafe { *p as u8 };
        if val == 0 {
            break;
        }
        if val.is_ascii_alphanumeric() || val == b'+' || val == b'-' || val == b'.' {
            p = unsafe { p.add(1) };
        } else {
            break;
        }
    }

    let last_body_char = unsafe { *p.offset(-1) as u8 };
    if last_body_char == b'+' || last_body_char == b'-' || last_body_char == b'.' {
        return 0;
    }

    unsafe { path_is_url(p) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_isAbsName(name: *const c_char) -> bool {
    unsafe { path_with_url(name) != 0 || path_is_absolute(name) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_path_is_absolute_unix() {
        #[cfg(unix)]
        {
            let p1 = CString::new("/usr/bin/local").unwrap();
            let p2 = CString::new("~/docs").unwrap();
            let p3 = CString::new("relative/path").unwrap();
            unsafe {
                assert!(path_is_absolute(p1.as_ptr()));
                assert!(path_is_absolute(p2.as_ptr()));
                assert!(!path_is_absolute(p3.as_ptr()));
            }
        }
    }

    #[test]
    fn test_path_is_url() {
        let u1 = CString::new(":/").unwrap();
        let u2 = CString::new(":\\\\").unwrap();
        let u3 = CString::new("invalid").unwrap();
        unsafe {
            assert_eq!(path_is_url(u1.as_ptr()), URL_SLASH);
            assert_eq!(path_is_url(u2.as_ptr()), URL_BACKSLASH);
            assert_eq!(path_is_url(u3.as_ptr()), 0);
        }
    }

    #[test]
    fn test_path_with_url() {
        let u1 = CString::new("http://example.com").unwrap();
        let u2 = CString::new("ftp://files").unwrap();
        let u3 = CString::new("invalid-url").unwrap();
        unsafe {
            assert_eq!(path_with_url(u1.as_ptr()), URL_SLASH);
            assert_eq!(path_with_url(u2.as_ptr()), URL_SLASH);
            assert_eq!(path_with_url(u3.as_ptr()), 0);
        }
    }

    #[test]
    fn test_vim_is_abs_name() {
        let p1 = CString::new("/absolute/path").unwrap();
        let p2 = CString::new("http://google.com").unwrap();
        let p3 = CString::new("relative/file").unwrap();
        unsafe {
            assert!(vim_isAbsName(p1.as_ptr()));
            assert!(vim_isAbsName(p2.as_ptr()));
            assert!(!vim_isAbsName(p3.as_ptr()));
        }
    }
}
