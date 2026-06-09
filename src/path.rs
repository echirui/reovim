use std::ffi::{c_char, c_int};

const NUL: u8 = 0;

unsafe extern "C" {
    // Declared in Neovim's mbyte.c
    pub fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;

    // Declared in Neovim's option_vars.h / globals.h
    #[allow(dead_code)]
    pub static p_fic: c_int;

    // Declared in Neovim's mbyte.c
    pub fn mb_strcmp_ic(ic: bool, s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn mb_strnicmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;

    // libc functions
    pub fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;

    // Windows compatibility helpers from mbyte.c
    pub fn utf_ptr2char(p: *const c_char) -> c_int;
    pub fn utfc_ptr2len(p: *const c_char) -> c_int;
    pub fn utf_fold(c: c_int) -> c_int;
}

#[inline(always)]
fn ascii_isalpha(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

unsafe fn mb_ptr_adv(p: &mut *const c_char) {
    let b = unsafe { **p as u8 };
    let len = if b < 0x80 {
        1
    } else if (b & 0xE0) == 0xC0 {
        2
    } else if (b & 0xF0) == 0xE0 {
        3
    } else if (b & 0xF8) == 0xF0 {
        4
    } else {
        1
    };
    *p = unsafe { p.add(len) };
}

#[unsafe(no_mangle)]
pub extern "C" fn vim_ispathsep(c: c_int) -> bool {
    let c = c as u8;
    #[cfg(unix)]
    {
        c == b'/'
    }
    #[cfg(not(unix))]
    {
        c == b':' || c == b'/' || c == b'\\'
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vim_ispathsep_nocolon(c: c_int) -> bool {
    let c = c as u8;
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
pub extern "C" fn vim_ispathlistsep(c: c_int) -> bool {
    let c = c as u8;
    #[cfg(unix)]
    {
        c == b':'
    }
    #[cfg(not(unix))]
    {
        c == b';'
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn is_path_head(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let first = unsafe { *path as u8 };
    if first == 0 {
        return false;
    }
    #[cfg(not(unix))]
    {
        let second = unsafe { *path.add(1) as u8 };
        ascii_isalpha(first) && second == b':'
    }
    #[cfg(unix)]
    {
        vim_ispathsep(first as c_int)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_past_head(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let mut retval = path;
    #[cfg(not(unix))]
    {
        if unsafe { is_path_head(path) } {
            retval = unsafe { path.add(2) };
        }
    }
    while unsafe { *retval } != 0 && vim_ispathsep(unsafe { *retval } as c_int) {
        retval = unsafe { retval.add(1) };
    }
    retval as *mut c_char
}

#[unsafe(no_mangle)]
pub extern "C" fn path_head_length() -> c_int {
    #[cfg(not(unix))]
    {
        3
    }
    #[cfg(unix)]
    {
        1
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
            if ascii_isalpha(first) && second == b':' && vim_ispathsep_nocolon(third as c_int) {
                return true;
            }
        }
        vim_ispathsep_nocolon(first as c_int)
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_tail(fname: *const c_char) -> *mut c_char {
    if fname.is_null() {
        return b"\0".as_ptr() as *mut c_char;
    }

    let mut tail = unsafe { get_past_head(fname) };
    let mut p = tail as *const c_char;
    unsafe {
        while *p != 0 {
            if vim_ispathsep_nocolon(*p as c_int) {
                tail = p.add(1) as *mut c_char;
            }
            mb_ptr_adv(&mut p);
        }
    }
    tail
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn after_pathsep(b: *const c_char, p: *const c_char) -> bool {
    if p.is_null() || b.is_null() || p <= b {
        return false;
    }
    let prev = unsafe { *p.offset(-1) as u8 };
    if !vim_ispathsep(prev as c_int) {
        return false;
    }
    unsafe { utf_head_off(b, p.offset(-1)) == 0 }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_tail_with_sep(fname: *mut c_char) -> *mut c_char {
    if fname.is_null() {
        return std::ptr::null_mut();
    }
    let past_head = unsafe { get_past_head(fname) };
    let mut tail = unsafe { path_tail(fname) };
    while tail > past_head && unsafe { after_pathsep(fname, tail) } {
        tail = unsafe { tail.offset(-1) };
    }
    tail
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn invocation_path_tail(invocation: *const c_char, len: *mut usize) -> *const c_char {
    if invocation.is_null() {
        if !len.is_null() {
            unsafe { *len = 0 };
        }
        return b"\0".as_ptr() as *const c_char;
    }

    let mut tail = unsafe { get_past_head(invocation) };
    let mut p = tail as *const c_char;
    unsafe {
        while *p != 0 && *p != b' ' as c_char {
            let was_sep = vim_ispathsep_nocolon(*p as c_int);
            mb_ptr_adv(&mut p);
            if was_sep {
                tail = p as *mut c_char;
            }
        }
    }

    if !len.is_null() {
        unsafe { *len = p as usize - tail as usize };
    }

    tail as *const c_char
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_next_component(fname: *const c_char) -> *const c_char {
    if fname.is_null() {
        return std::ptr::null();
    }
    let mut f = fname;
    unsafe {
        while *f != 0 && !vim_ispathsep(*f as c_int) {
            mb_ptr_adv(&mut f);
        }
        if *f != 0 {
            f = f.add(1);
        }
    }
    f
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_has_wildcard(mut p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    unsafe {
        while *p != 0 {
            #[cfg(unix)]
            {
                if *p == b'\\' as c_char && *p.add(1) != 0 {
                    p = p.add(2);
                    continue;
                }
                let wildcards = b"*?[{`'$";
                let curr = *p as u8;
                if wildcards.contains(&curr) || (curr == b'~' && *p.add(1) != 0) {
                    return true;
                }
            }
            #[cfg(not(unix))]
            {
                let wildcards = b"?*$[`";
                let curr = *p as u8;
                if wildcards.contains(&curr) || (curr == b'~' && *p.add(1) != 0) {
                    return true;
                }
            }
            mb_ptr_adv(&mut p);
        }
    }
    false
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_has_exp_wildcard(mut p: *const c_char) -> bool {
    if p.is_null() {
        return false;
    }
    unsafe {
        while *p != 0 {
            #[cfg(unix)]
            {
                if *p == b'\\' as c_char && *p.add(1) != 0 {
                    p = p.add(2);
                    continue;
                }
                let wildcards = b"*?[{";
                let curr = *p as u8;
                if wildcards.contains(&curr) {
                    return true;
                }
            }
            #[cfg(not(unix))]
            {
                let wildcards = b"*?[";
                let curr = *p as u8;
                if wildcards.contains(&curr) {
                    return true;
                }
            }
            mb_ptr_adv(&mut p);
        }
    }
    false
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_fnamecmp(fname1: *const c_char, fname2: *const c_char) -> c_int {
    if fname1.is_null() || fname2.is_null() {
        return 0;
    }
    #[cfg(not(unix))]
    {
        // Simple delegating for Windows compile check
        let len1 = unsafe { libc::strlen(fname1) };
        let len2 = unsafe { libc::strlen(fname2) };
        unsafe { path_fnamencmp(fname1, fname2, std::cmp::max(len1, len2)) }
    }
    #[cfg(unix)]
    {
        unsafe { mb_strcmp_ic(p_fic != 0, fname1, fname2) }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_fnamencmp(fname1: *const c_char, fname2: *const c_char, len: usize) -> c_int {
    if fname1.is_null() || fname2.is_null() || len == 0 {
        return 0;
    }
    #[cfg(not(unix))]
    {
        let mut p1 = fname1;
        let mut p2 = fname2;
        let mut remaining_len = len;
        let mut c1 = 0;
        let mut c2 = 0;

        while remaining_len > 0 {
            c1 = unsafe { utf_ptr2char(p1) };
            c2 = unsafe { utf_ptr2char(p2) };
            if c1 == 0 || c2 == 0 {
                break;
            }
            let is_sep1 = c1 == b'/' as c_int || c1 == b'\\' as c_int;
            let is_sep2 = c2 == b'/' as c_int || c2 == b'\\' as c_int;
            let both_seps = is_sep1 && is_sep2;

            let not_equal = if both_seps {
                false
            } else if unsafe { p_fic != 0 } {
                c1 != c2 && unsafe { utf_fold(c1) != utf_fold(c2) }
            } else {
                c1 != c2
            };

            if not_equal {
                break;
            }
            let step = unsafe { utfc_ptr2len(p1) } as usize;
            if step > remaining_len {
                break;
            }
            remaining_len -= step;
            p1 = unsafe { p1.add(step) };
            p2 = unsafe { p2.add(utfc_ptr2len(p2) as usize) };
        }

        c1 = unsafe { utf_ptr2char(p1) };
        c2 = unsafe { utf_ptr2char(p2) };
        if unsafe { p_fic != 0 } {
            unsafe { utf_fold(c1) - utf_fold(c2) }
        } else {
            c1 - c2
        }
    }
    #[cfg(unix)]
    {
        unsafe {
            if p_fic != 0 {
                mb_strnicmp(fname1, fname2, len)
            } else {
                strncmp(fname1, fname2, len)
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_with_extension(path: *const c_char, extension: *const c_char) -> bool {
    if path.is_null() || extension.is_null() {
        return false;
    }
    unsafe {
        let mut last_dot = std::ptr::null();
        let mut curr = path;
        while *curr != 0 {
            if *curr == b'.' as c_char {
                last_dot = curr;
            }
            curr = curr.add(1);
        }
        if last_dot.is_null() {
            return false;
        }
        mb_strcmp_ic(p_fic != 0, last_dot.add(1), extension) == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[unsafe(no_mangle)]
    pub static mut p_fic: c_int = 0;

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn utf_head_off(_base: *const c_char, _p: *const c_char) -> c_int {
        0
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn mb_strcmp_ic(ic: bool, s1: *const c_char, s2: *const c_char) -> c_int {
        let mut p1 = s1;
        let mut p2 = s2;
        unsafe {
            while *p1 != 0 && *p2 != 0 {
                let c1 = if ic { (*p1 as u8).to_ascii_lowercase() } else { *p1 as u8 };
                let c2 = if ic { (*p2 as u8).to_ascii_lowercase() } else { *p2 as u8 };
                if c1 != c2 {
                    return (c1 as c_int) - (c2 as c_int);
                }
                p1 = p1.add(1);
                p2 = p2.add(1);
            }
            let c1 = if ic { (*p1 as u8).to_ascii_lowercase() } else { *p1 as u8 };
            let c2 = if ic { (*p2 as u8).to_ascii_lowercase() } else { *p2 as u8 };
            (c1 as c_int) - (c2 as c_int)
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn mb_strnicmp(s1: *const c_char, s2: *const c_char, mut n: usize) -> c_int {
        let mut p1 = s1;
        let mut p2 = s2;
        unsafe {
            while n > 0 && *p1 != 0 && *p2 != 0 {
                let c1 = (*p1 as u8).to_ascii_lowercase();
                let c2 = (*p2 as u8).to_ascii_lowercase();
                if c1 != c2 {
                    return (c1 as c_int) - (c2 as c_int);
                }
                p1 = p1.add(1);
                p2 = p2.add(1);
                n -= 1;
            }
            if n == 0 {
                0
            } else {
                ((*p1 as u8).to_ascii_lowercase() as c_int) - ((*p2 as u8).to_ascii_lowercase() as c_int)
            }
        }
    }

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
    fn test_path_tail() {
        let p1 = CString::new("dir/file.txt").unwrap();
        let p2 = CString::new("file.txt").unwrap();
        let p3 = CString::new("dir/").unwrap();
        unsafe {
            let t1 = CString::from_raw(path_tail(p1.as_ptr()));
            assert_eq!(t1.to_str().unwrap(), "file.txt");
            let _ = t1.into_raw(); // prevent double free

            let t2 = CString::from_raw(path_tail(p2.as_ptr()));
            assert_eq!(t2.to_str().unwrap(), "file.txt");
            let _ = t2.into_raw();

            let t3 = CString::from_raw(path_tail(p3.as_ptr()));
            assert_eq!(t3.to_str().unwrap(), "");
            let _ = t3.into_raw();
        }
    }

    #[test]
    fn test_path_next_component() {
        let p1 = CString::new("dir/subdir/file.txt").unwrap();
        unsafe {
            let next = path_next_component(p1.as_ptr());
            let c_str = std::ffi::CStr::from_ptr(next);
            assert_eq!(c_str.to_str().unwrap(), "subdir/file.txt");
        }
    }

    #[test]
    fn test_path_has_wildcard() {
        let p1 = CString::new("dir/*.txt").unwrap();
        let p2 = CString::new("dir/file.txt").unwrap();
        unsafe {
            assert!(path_has_wildcard(p1.as_ptr()));
            assert!(!path_has_wildcard(p2.as_ptr()));
        }
    }

    #[test]
    fn test_path_fnamecmp_unix() {
        #[cfg(unix)]
        {
            let p1 = CString::new("file.txt").unwrap();
            let p2 = CString::new("FILE.TXT").unwrap();
            unsafe {
                p_fic = 1; // ignore case
                assert_eq!(path_fnamecmp(p1.as_ptr(), p2.as_ptr()), 0);
                p_fic = 0; // case sensitive
                assert_ne!(path_fnamecmp(p1.as_ptr(), p2.as_ptr()), 0);
            }
        }
    }
}
