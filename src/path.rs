use std::ffi::{c_char, c_int};

const NUL: u8 = 0;
const MAXPATHL: usize = 4096;

#[cfg(unix)]
const PATHSEPSTR: &[u8] = b"/\0";

#[cfg(unix)]
const PATHSEP: u8 = b'/';

#[cfg(not(unix))]
const PATHSEPSTR: &[u8] = b"\\\0";

#[cfg(not(unix))]
const PATHSEP: u8 = b'\\';

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum FileComparison {
    EqualFiles = 1,
    DifferentFiles = 2,
    BothFilesMissing = 4,
    OneFileMissing = 6,
    EqualFileNames = 7,
}

#[repr(C)]
pub struct FileID {
    pub inode: u64,
    pub device_id: u64,
}

#[repr(C, align(8))]
pub struct FileInfo {
    pub _data: [u8; 160],
}

#[repr(C, align(8))]
pub struct Directory {
    pub _data: [u8; 456],
}

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
    pub fn strcasecmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn strrchr(s: *const c_char, c: c_int) -> *mut c_char;
    pub fn xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize;

    // Windows compatibility helpers from mbyte.c
    pub fn utf_ptr2char(p: *const c_char) -> c_int;
    pub fn utfc_ptr2len(p: *const c_char) -> c_int;
    pub fn utf_fold(c: c_int) -> c_int;

    // Declared in Neovim's os/fs.c or os/stdpaths.c
    pub fn os_dirname(buf: *mut c_char, len: usize) -> c_int;
    pub fn os_realpath(path: *const c_char, buf: *mut c_char, len: usize) -> *mut c_char;
    pub fn os_fileid(name: *const c_char, file_id: *mut FileID) -> bool;
    pub fn os_fileid_equal(file_id_1: *const FileID, file_id_2: *const FileID) -> bool;
    pub fn expand_env(src: *const c_char, dst: *mut c_char, dst_len: c_int);
    pub fn os_fileinfo_link(name: *const c_char, file_info: *mut FileInfo) -> bool;
    pub fn os_scandir(dir: *mut Directory, path: *const c_char) -> bool;
    pub fn os_scandir_next(dir: *mut Directory) -> *const c_char;
    pub fn os_closedir(dir: *mut Directory);
    pub fn os_fileinfo_id_equal(file_info_1: *const FileInfo, file_info_2: *const FileInfo) -> bool;

    #[cfg(not(unix))]
    pub fn slash_adjust(p: *mut c_char);

    // Memory and OS utility functions from Neovim
    pub fn xmalloc(size: usize) -> *mut c_char;
    pub fn xrealloc(ptr: *mut c_char, size: usize) -> *mut c_char;
    pub fn xfree(ptr: *mut c_char);
    pub fn xstrdup(s: *const c_char) -> *mut c_char;
    pub fn os_isdir(name: *const c_char) -> bool;
    pub fn os_fileinfo(name: *const c_char, file_info: *mut FileInfo) -> bool;
    pub fn mb_toupper(a: c_int) -> c_int;
    pub fn mb_tolower(a: c_int) -> c_int;
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_full_compare(
    s1: *const c_char,
    s2: *const c_char,
    checkname: bool,
    expandenv: bool,
) -> FileComparison {
    let mut expanded1 = [0; MAXPATHL];
    let mut full1 = [0; MAXPATHL];
    let mut full2 = [0; MAXPATHL];
    let mut file_id_1 = FileID { inode: 0, device_id: 0 };
    let mut file_id_2 = FileID { inode: 0, device_id: 0 };

    if expandenv {
        unsafe {
            expand_env(s1, expanded1.as_mut_ptr(), MAXPATHL as c_int);
        }
    } else {
        unsafe {
            xstrlcpy(expanded1.as_mut_ptr(), s1, MAXPATHL);
        }
    }

    let id_ok_1 = unsafe { os_fileid(expanded1.as_ptr(), &mut file_id_1) };
    let id_ok_2 = unsafe { os_fileid(s2, &mut file_id_2) };

    if !id_ok_1 && !id_ok_2 {
        if checkname {
            unsafe {
                vim_FullName(expanded1.as_ptr(), full1.as_mut_ptr(), MAXPATHL, false);
                vim_FullName(s2, full2.as_mut_ptr(), MAXPATHL, false);
            }
            if unsafe { path_fnamecmp(full1.as_ptr(), full2.as_ptr()) } == 0 {
                return FileComparison::EqualFileNames;
            }
        }
        return FileComparison::BothFilesMissing;
    }

    if !id_ok_1 || !id_ok_2 {
        return FileComparison::OneFileMissing;
    }

    if unsafe { os_fileid_equal(&file_id_1, &file_id_2) } {
        return FileComparison::EqualFiles;
    }

    FileComparison::DifferentFiles
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_fix_case(name: *mut c_char) {
    let mut file_info = FileInfo { _data: [0; 160] };
    if unsafe { !os_fileinfo_link(name, &mut file_info) } {
        return;
    }

    let slash = unsafe { strrchr(name, b'/' as c_int) };
    let mut tail = name;
    let mut dir = Directory { _data: [0; 456] };
    let ok;

    if slash.is_null() {
        ok = unsafe { os_scandir(&mut dir, b".\0".as_ptr() as *const c_char) };
    } else {
        unsafe {
            *slash = 0;
        }
        ok = unsafe { os_scandir(&mut dir, name) };
        unsafe {
            *slash = b'/' as c_char;
        }
        tail = unsafe { slash.add(1) };
    }

    if !ok {
        return;
    }

    let taillen = unsafe { strlen(tail) };
    loop {
        let entry = unsafe { os_scandir_next(&mut dir) };
        if entry.is_null() {
            break;
        }

        let entry_len = unsafe { strlen(entry) };
        if unsafe { strcasecmp(tail, entry) } == 0 && taillen == entry_len {
            let mut newname = [0; MAXPATHL + 1];
            unsafe {
                xstrlcpy(newname.as_mut_ptr(), name, MAXPATHL + 1);
                let offset = tail as usize - name as usize;
                xstrlcpy(newname.as_mut_ptr().add(offset), entry, MAXPATHL - offset + 1);
            }
            let mut file_info_new = FileInfo { _data: [0; 160] };
            if unsafe { os_fileinfo_link(newname.as_ptr(), &mut file_info_new) }
                && unsafe { os_fileinfo_id_equal(&file_info, &file_info_new) }
            {
                unsafe {
                    strcpy(tail, entry);
                }
                break;
            }
        }
    }

    unsafe {
        os_closedir(&mut dir);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_try_shorten_fname(full_path: *mut c_char) -> *mut c_char {
    if full_path.is_null() {
        return std::ptr::null_mut();
    }
    let mut dirname = [0; MAXPATHL];
    let mut p = full_path;
    if unsafe { os_dirname(dirname.as_mut_ptr(), MAXPATHL) } == 1 {
        let shortened = unsafe { path_shorten_fname(full_path, dirname.as_ptr()) };
        if !shortened.is_null() && unsafe { *shortened } != 0 {
            p = shortened;
        }
    }
    p
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_shorten_fname(full_path: *mut c_char, dir_name: *const c_char) -> *mut c_char {
    if full_path.is_null() {
        return std::ptr::null_mut();
    }
    assert!(!dir_name.is_null());
    let len = unsafe { strlen(dir_name) };
    if unsafe { path_fnamencmp(dir_name, full_path, len) } != 0 {
        return std::ptr::null_mut();
    }

    if len == path_head_length() as usize && unsafe { is_path_head(dir_name) } {
        return unsafe { full_path.add(len) };
    }

    let mut p = unsafe { full_path.add(len) };
    if unsafe { !vim_ispathsep(*p as c_int) } {
        return std::ptr::null_mut();
    }

    unsafe {
        loop {
            p = p.add(1);
            if !vim_ispathsep_nocolon(*p as c_int) {
                break;
            }
        }
    }
    p
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_full_dir_name(directory: *mut c_char, buffer: *mut c_char, len: usize) -> c_int {
    if unsafe { strlen(directory) } == 0 {
        return unsafe { os_dirname(buffer, len) };
    }

    if !unsafe { os_realpath(directory, buffer, len) }.is_null() {
        return 1;
    }

    if unsafe { path_is_absolute(directory) } {
        return 0;
    }

    let mut old_dir = [0; MAXPATHL];
    if unsafe { os_dirname(old_dir.as_mut_ptr(), MAXPATHL) } == 0 {
        return 0;
    }

    unsafe {
        xstrlcpy(buffer, old_dir.as_ptr(), len);
    }
    if unsafe { append_path(buffer, directory, len) } == 0 {
        return 0;
    }

    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_to_absolute(fname: *const c_char, buf: *mut c_char, len: usize, force: c_int) -> c_int {
    unsafe { *buf = 0 };

    let mut relative_directory_buf = vec![0; len];
    let relative_directory = relative_directory_buf.as_mut_ptr() as *mut c_char;
    let mut end_of_path = fname;

    let is_abs = unsafe { path_is_absolute(fname) };
    #[allow(unused_mut)]
    let mut should_expand = force != 0 || !is_abs;

    #[cfg(not(unix))]
    {
        if !should_expand && !fname.is_null() {
            let first = unsafe { *fname as u8 };
            if first == b'/' || first == b'\\' {
                should_expand = true;
            }
        }
    }

    if should_expand {
        let mut p = unsafe { strrchr(fname, b'/' as c_int) };
        #[cfg(not(unix))]
        {
            if p.is_null() {
                p = unsafe { strrchr(fname, b'\\' as c_int) };
            }
            if p.is_null() && !fname.is_null() {
                let first = unsafe { *fname as u8 };
                let second = unsafe { *fname.add(1) as u8 };
                if ascii_isalpha(first) && second == b':' {
                    p = unsafe { fname.add(1) as *mut c_char };
                }
            }
        }
        if p.is_null() && unsafe { strcmp(fname, b"..\0".as_ptr() as *const c_char) } == 0 {
            p = unsafe { fname.add(2) as *mut c_char };
        }
        if !p.is_null() {
            if unsafe { vim_ispathsep(*p as c_int) } && unsafe { strcmp(p.add(1), b"..\0".as_ptr() as *const c_char) } == 0 {
                p = unsafe { p.add(3) };
            }
            assert!(p as *const c_char >= fname);
            let copy_len = p as usize - fname as usize + 1;
            unsafe {
                std::ptr::copy_nonoverlapping(fname, relative_directory, copy_len);
                *relative_directory.add(copy_len) = 0;
            }
            end_of_path = unsafe {
                if vim_ispathsep(*p as c_int) {
                    p.add(1)
                } else {
                    p
                }
            };
        } else {
            unsafe { *relative_directory = 0 };
        }

        if unsafe { path_full_dir_name(relative_directory, buf, len) } == 0 {
            return 0;
        }
    }

    unsafe { append_path(buf, end_of_path, len) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn append_path(path: *mut c_char, to_append: *const c_char, max_len: usize) -> c_int {
    let current_length = unsafe { strlen(path) };
    let to_append_length = unsafe { strlen(to_append) };

    if to_append_length == 0 || unsafe { strcmp(to_append, b".\0".as_ptr() as *const c_char) } == 0 {
        return 1;
    }

    let mut current_length = current_length;
    if current_length > 0 && unsafe { !vim_ispathsep_nocolon(*path.add(current_length - 1) as c_int) } {
        if current_length + 1 + 1 > max_len {
            return 0;
        }
        unsafe {
            xstrlcpy(path.add(current_length), PATHSEPSTR.as_ptr() as *const c_char, max_len - current_length);
        }
        current_length += 1;
    }

    if current_length + to_append_length + 1 > max_len {
        return 0;
    }

    unsafe {
        xstrlcpy(path.add(current_length), to_append, max_len - current_length);
    }
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_FullName(fname: *const c_char, buf: *mut c_char, len: usize, force: bool) -> c_int {
    unsafe { *buf = 0 };
    if fname.is_null() {
        return 0;
    }

    let fname_len = unsafe { strlen(fname) };
    if fname_len > (len - 1) {
        unsafe {
            xstrlcpy(buf, fname, len);
        }
        #[cfg(not(unix))]
        unsafe {
            slash_adjust(buf);
        }
        return 0;
    }

    if unsafe { path_with_url(fname) } != 0 {
        unsafe {
            xstrlcpy(buf, fname, len);
        }
        return 1;
    }

    let rv = unsafe { path_to_absolute(fname, buf, len, force as c_int) };
    if rv == 0 {
        unsafe {
            xstrlcpy(buf, fname, len);
        }
    }
    #[cfg(not(unix))]
    unsafe {
        slash_adjust(buf);
    }
    rv
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn shorten_dir_len(str_: *mut c_char, trim_len: c_int) {
    let tail = unsafe { path_tail(str_) };
    let mut d = str_;
    let mut skip = false;
    let mut dirchunk_len = 0;
    let mut s = str_;
    loop {
        if s >= tail {
            unsafe {
                *d = *s;
                d = d.add(1);
                if *s as u8 == NUL {
                    break;
                }
                s = s.add(1);
            }
        } else if unsafe { vim_ispathsep(*s as c_int) } {
            unsafe {
                *d = *s;
                d = d.add(1);
                skip = false;
                dirchunk_len = 0;
                s = s.add(1);
            }
        } else if !skip {
            unsafe {
                *d = *s;
                d = d.add(1);
                if *s as u8 != b'~' && *s as u8 != b'.' {
                    dirchunk_len += 1;
                    if dirchunk_len >= trim_len {
                        skip = true;
                    }
                }
                let mut l = utfc_ptr2len(s);
                while l > 1 {
                    s = s.add(1);
                    *d = *s;
                    d = d.add(1);
                    l -= 1;
                }
                s = s.add(1);
            }
        } else {
            unsafe {
                s = s.add(1);
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn shorten_dir(str_: *mut c_char) {
    unsafe { shorten_dir_len(str_, 1) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dir_of_file_exists(fname: *mut c_char) -> bool {
    let p = unsafe { path_tail_with_sep(fname) };
    if p == fname {
        return true;
    }
    let c = unsafe { *p };
    unsafe { *p = NUL as c_char };
    let retval = unsafe { os_isdir(fname) };
    unsafe { *p = c };
    retval
}

unsafe fn do_concat_fnames(fname1: *mut c_char, len1: usize, fname2: *const c_char, len2: usize, sep: bool) -> *mut c_char {
    if sep && unsafe { *fname1 as u8 != NUL && !after_pathsep(fname1, fname1.add(len1)) } {
        unsafe {
            *fname1.add(len1) = PATHSEP as c_char;
            std::ptr::copy(fname2, fname1.add(len1 + 1), len2 + 1);
        }
    } else {
        unsafe {
            std::ptr::copy(fname2, fname1.add(len1), len2 + 1);
        }
    }
    fname1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn concat_fnames(fname1: *const c_char, fname2: *const c_char, sep: bool) -> *mut c_char {
    let len1 = unsafe { strlen(fname1) };
    let len2 = unsafe { strlen(fname2) };
    let dest = unsafe { xmalloc(len1 + len2 + 3) };
    unsafe {
        std::ptr::copy(fname1, dest, len1 + 1);
        do_concat_fnames(dest, len1, fname2, len2, sep)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn concat_fnames_realloc(fname1: *mut c_char, fname2: *const c_char, sep: bool) -> *mut c_char {
    let len1 = unsafe { strlen(fname1) };
    let len2 = unsafe { strlen(fname2) };
    let new_ptr = unsafe { xrealloc(fname1, len1 + len2 + 3) };
    unsafe { do_concat_fnames(new_ptr, len1, fname2, len2, sep) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn add_pathsep(p: *mut c_char) -> bool {
    let len = unsafe { strlen(p) };
    if unsafe { *p as u8 != NUL && !after_pathsep(p, p.add(len)) } {
        let pathsep_len = PATHSEPSTR.len();
        if len > MAXPATHL - pathsep_len {
            return false;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(PATHSEPSTR.as_ptr() as *const c_char, p.add(len), pathsep_len);
        }
    }
    true
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn FullName_save(fname: *const c_char, force: bool) -> *mut c_char {
    if fname.is_null() {
        return std::ptr::null_mut();
    }
    let buf = unsafe { xmalloc(MAXPATHL) };
    if unsafe { vim_FullName(fname, buf, MAXPATHL, force) } == 0 {
        unsafe {
            xfree(buf);
            xstrdup(fname)
        }
    } else {
        buf
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn save_abs_path(name: *const c_char) -> *mut c_char {
    if !unsafe { path_is_absolute(name) } {
        unsafe { FullName_save(name, true) }
    } else {
        unsafe { xstrdup(name) }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn fix_fname(fname: *const c_char) -> *mut c_char {
    #[cfg(unix)]
    {
        unsafe { FullName_save(fname, true) }
    }
    #[cfg(not(unix))]
    {
        use std::ffi::CStr;
        let s = unsafe { CStr::from_ptr(fname) }.to_bytes();
        let has_double_dots = s.windows(2).any(|w| w == b"..");
        let has_double_slashes = s.windows(2).any(|w| w == b"//");
        let has_double_backslashes = s.windows(2).any(|w| w == b"\\\\");
        let starts_with_slash = !s.is_empty() && (s[0] == b'/' || s[0] == b'\\');

        if !unsafe { vim_isAbsName(fname) }
            || has_double_dots
            || has_double_slashes
            || has_double_backslashes
            || starts_with_slash
        {
            unsafe { FullName_save(fname, false) }
        } else {
            let res = unsafe { xstrdup(fname) };
            #[cfg(target_os = "windows")]
            {
                unsafe { path_fix_case(res) };
            }
            res
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn same_directory(f1: *mut c_char, f2: *mut c_char) -> bool {
    let mut ffname = [0 as c_char; MAXPATHL];
    if f1.is_null() || f2.is_null() {
        return false;
    }
    unsafe { vim_FullName(f1, ffname.as_mut_ptr(), MAXPATHL, false) };
    let t1 = unsafe { path_tail_with_sep(ffname.as_mut_ptr()) };
    let t2 = unsafe { path_tail_with_sep(f2) };
    let offset1 = unsafe { t1.offset_from(ffname.as_ptr()) };
    let offset2 = unsafe { t2.offset_from(f2) };
    offset1 == offset2 && unsafe { pathcmp(ffname.as_ptr(), f2, offset1 as c_int) } == 0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pathcmp(p: *const c_char, q: *const c_char, maxlen: c_int) -> c_int {
    let mut i = 0;
    let mut j = 0;
    let mut s: *const c_char = std::ptr::null();

    loop {
        if maxlen >= 0 && (i >= maxlen as usize || j >= maxlen as usize) {
            break;
        }

        let c1 = unsafe { utf_ptr2char(p.add(i)) };
        let c2 = unsafe { utf_ptr2char(q.add(j)) };

        // End of "p": check if "q" also ends or just has a slash.
        if c1 == NUL as c_int {
            if c2 == NUL as c_int {
                return 0;
            }
            s = q;
            i = j;
            break;
        }

        // End of "q": check if "p" just has a slash.
        if c2 == NUL as c_int {
            s = p;
            break;
        }

        let match_fic = unsafe {
            if p_fic != 0 {
                mb_toupper(c1) != mb_toupper(c2)
            } else {
                c1 != c2
            }
        };

        #[cfg(target_os = "windows")]
        let match_fic = match_fic && !(
            (c1 == b'/' as c_int && c2 == b'\\' as c_int)
            || (c1 == b'\\' as c_int && c2 == b'/' as c_int)
        );

        if match_fic {
            if vim_ispathsep(c1) {
                return -1;
            }
            if vim_ispathsep(c2) {
                return 1;
            }
            return unsafe {
                if p_fic != 0 {
                    mb_toupper(c1) - mb_toupper(c2)
                } else {
                    c1 - c2
                }
            };
        }

        i += unsafe { utfc_ptr2len(p.add(i)) } as usize;
        j += unsafe { utfc_ptr2len(q.add(j)) } as usize;
    }

    if s.is_null() {
        return 0;
    }

    let c1 = unsafe { utf_ptr2char(s.add(i)) };
    let c2 = unsafe { utf_ptr2char(s.add(i + utfc_ptr2len(s.add(i)) as usize)) };

    // ignore a trailing slash, but not "//" or ":/"
    let is_slash = if cfg!(target_os = "windows") {
        c1 == b'/' as c_int || c1 == b'\\' as c_int
    } else {
        c1 == b'/' as c_int
    };

    if c2 == NUL as c_int
        && i > 0
        && !unsafe { after_pathsep(s, s.add(i)) }
        && is_slash
    {
        return 0;
    }

    if s == q {
        return -1;
    }
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn simplify_filename(filename: *mut c_char) -> usize {
    let mut components = 0;
    let mut stripping_disabled = false;
    let mut relative = true;

    let mut p = filename;
    #[cfg(target_os = "windows")]
    {
        if unsafe { *p as u8 != NUL && *p.add(1) as u8 == b':' } {
            p = unsafe { p.add(2) };
        }
    }

    if unsafe { vim_ispathsep(*p as c_int) } {
        relative = false;
        while unsafe { vim_ispathsep(*p as c_int) } {
            p = unsafe { p.add(1) };
        }
    }
    let mut start = p;
    let mut p_end = unsafe { p.add(strlen(p)) };

    #[cfg(unix)]
    {
        // Posix says that "//path" is unchanged but "///path" is "/path".
        if start > unsafe { filename.add(2) } {
            unsafe {
                std::ptr::copy(p, filename.add(1), p_end.offset_from(p) as usize + 1);
                p_end = p_end.offset(-(p.offset_from(filename.add(1)) as isize));
                p = filename.add(1);
                start = p;
            }
        }
    }

    loop {
        // At this point "p" is pointing to the char following a single "/"
        // or "p" is at the "start" of the (absolute or relative) path name.
        if unsafe { vim_ispathsep(*p as c_int) } {
            unsafe {
                std::ptr::copy(p.add(1), p, p_end.offset_from(p.add(1)) as usize + 1);
                p_end = p_end.offset(-1);
            }
        } else if unsafe { *p as u8 == b'.' && (vim_ispathsep(*p.add(1) as c_int) || *p.add(1) as u8 == NUL) } {
            if p == start && relative {
                p = unsafe { p.add(1 + (*p.add(1) as u8 != NUL) as usize) };
            } else {
                let mut tail = unsafe { p.add(1) };
                if unsafe { *p.add(1) as u8 != NUL } {
                    while unsafe { vim_ispathsep(*tail as c_int) } {
                        tail = unsafe { tail.add(utfc_ptr2len(tail) as usize) };
                    }
                } else if p > start {
                    p = unsafe { p.offset(-1) };
                }
                unsafe {
                    std::ptr::copy(tail, p, p_end.offset_from(tail) as usize + 1);
                    p_end = p_end.offset(-(tail.offset_from(p) as isize));
                }
            }
        } else if unsafe { *p as u8 == b'.' && *p.add(1) as u8 == b'.' && (vim_ispathsep(*p.add(2) as c_int) || *p.add(2) as u8 == NUL) } {
            let mut tail = unsafe { p.add(2) };
            while unsafe { vim_ispathsep(*tail as c_int) } {
                tail = unsafe { tail.add(utfc_ptr2len(tail) as usize) };
            }

            if components > 0 {
                let mut do_strip = false;

                if !stripping_disabled {
                    let saved_char = unsafe { *p.offset(-1) };
                    unsafe { *p.offset(-1) = NUL as c_char };
                    let mut file_info = FileInfo { _data: [0; 160] };
                    if !unsafe { os_fileinfo_link(filename, &mut file_info) } {
                        do_strip = true;
                    }
                    unsafe { *p.offset(-1) = saved_char };

                    p = unsafe { p.offset(-1) };
                    while p > start && !unsafe { after_pathsep(start, p) } {
                        let offset = unsafe { utf_head_off(start, p.offset(-1)) + 1 };
                        p = unsafe { p.offset(-(offset as isize)) };
                    }

                    if !do_strip {
                        let saved_char2 = unsafe { *tail };
                        unsafe { *tail = NUL as c_char };
                        if unsafe { os_fileinfo(filename, &mut file_info) } {
                            do_strip = true;
                        } else {
                            stripping_disabled = true;
                        }
                        unsafe { *tail = saved_char2 };

                        if do_strip {
                            let mut new_file_info = FileInfo { _data: [0; 160] };
                            if p == start && relative {
                                unsafe { os_fileinfo(b".\0".as_ptr() as *const c_char, &mut new_file_info) };
                            } else {
                                let saved_char3 = unsafe { *p };
                                unsafe { *p = NUL as c_char };
                                unsafe { os_fileinfo(filename, &mut new_file_info) };
                                unsafe { *p = saved_char3 };
                            }

                            if !unsafe { os_fileinfo_id_equal(&file_info, &new_file_info) } {
                                do_strip = false;
                            }
                        }
                    }
                }

                if !do_strip {
                    p = tail;
                    components = 0;
                } else {
                    if p == start && relative && unsafe { *tail.offset(-1) as u8 == b'.' } {
                        unsafe {
                            *p = b'.' as c_char;
                            p = p.add(1);
                            *p = NUL as c_char;
                        }
                    } else {
                        if p > start && unsafe { *tail.offset(-1) as u8 == b'.' } {
                            p = unsafe { p.offset(-1) };
                        }
                        unsafe {
                            std::ptr::copy(tail, p, p_end.offset_from(tail) as usize + 1);
                            p_end = p_end.offset(-(tail.offset_from(p) as isize));
                        }
                    }
                    components -= 1;
                }
            } else if p == start && !relative {
                unsafe {
                    std::ptr::copy(tail, p, p_end.offset_from(tail) as usize + 1);
                    p_end = p_end.offset(-(tail.offset_from(p) as isize));
                }
            } else {
                if p == unsafe { start.add(2) } && unsafe { *p.offset(-2) as u8 == b'.' } {
                    unsafe {
                        std::ptr::copy(p, p.offset(-2), p_end.offset_from(p) as usize + 1);
                        p_end = p_end.offset(-2);
                        tail = tail.offset(-2);
                    }
                }
                p = tail;
            }
        } else {
            components += 1;
            p = unsafe { path_next_component(p) as *mut c_char };
        }

        if unsafe { *p as u8 == NUL } {
            break;
        }
    }

    unsafe { p_end.offset_from(filename) as usize }
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

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize {
        let src_len = unsafe { strlen(src) };
        if dsize == 0 {
            return src_len;
        }
        let copy_len = std::cmp::min(src_len, dsize - 1);
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst, copy_len);
            *dst.add(copy_len) = 0;
        }
        src_len
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn os_dirname(buf: *mut c_char, len: usize) -> c_int {
        let mock_dir = b"/home/user\0";
        if len < mock_dir.len() {
            return 0;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(mock_dir.as_ptr() as *const c_char, buf, mock_dir.len());
        }
        1
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn os_realpath(path: *const c_char, buf: *mut c_char, len: usize) -> *mut c_char {
        let first = unsafe { *path as u8 };
        if first != b'/' {
            return std::ptr::null_mut();
        }
        let path_len = unsafe { strlen(path) };
        if path_len >= len {
            return std::ptr::null_mut();
        }
        unsafe {
            std::ptr::copy_nonoverlapping(path, buf, path_len + 1);
        }
        buf
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn expand_env(src: *const c_char, dst: *mut c_char, dst_len: c_int) {
        let src_len = unsafe { strlen(src) };
        let copy_len = std::cmp::min(src_len, dst_len as usize - 1);
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst, copy_len);
            *dst.add(copy_len) = 0;
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn os_fileid(name: *const c_char, file_id: *mut FileID) -> bool {
        let name_str = unsafe { std::ffi::CStr::from_ptr(name) }.to_string_lossy();
        if name_str.contains("exist") || name_str.starts_with('/') {
            unsafe {
                if name_str.contains("file1") {
                    (*file_id).inode = 11111;
                } else if name_str.contains("file2") {
                    (*file_id).inode = 22222;
                } else {
                    (*file_id).inode = 12345;
                }
                (*file_id).device_id = 1;
            }
            true
        } else {
            false
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn os_fileid_equal(file_id_1: *const FileID, file_id_2: *const FileID) -> bool {
        unsafe { (*file_id_1).inode == (*file_id_2).inode && (*file_id_1).device_id == (*file_id_2).device_id }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn os_fileinfo_link(_name: *const c_char, _file_info: *mut FileInfo) -> bool {
        true
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn os_fileinfo_id_equal(_file_info_1: *const FileInfo, _file_info_2: *const FileInfo) -> bool {
        true
    }

    static mut SCANDIR_COUNT: usize = 0;

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn os_scandir(_dir: *mut Directory, _path: *const c_char) -> bool {
        unsafe {
            SCANDIR_COUNT = 0;
        }
        true
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn os_scandir_next(_dir: *mut Directory) -> *const c_char {
        unsafe {
            SCANDIR_COUNT += 1;
            if SCANDIR_COUNT == 1 {
                b"FILE.txt\0".as_ptr() as *const c_char
            } else {
                std::ptr::null()
            }
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn os_closedir(_dir: *mut Directory) {}

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
            let t1 = std::ffi::CStr::from_ptr(path_tail(p1.as_ptr()));
            assert_eq!(t1.to_str().unwrap(), "file.txt");

            let t2 = std::ffi::CStr::from_ptr(path_tail(p2.as_ptr()));
            assert_eq!(t2.to_str().unwrap(), "file.txt");

            let t3 = std::ffi::CStr::from_ptr(path_tail(p3.as_ptr()));
            assert_eq!(t3.to_str().unwrap(), "");
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

    #[test]
    fn test_path_shorten_fname() {
        let full_path = CString::new("/home/user/docs/file.txt").unwrap();
        let dir_name = CString::new("/home/user").unwrap();
        unsafe {
            let shortened = path_shorten_fname(full_path.as_ptr() as *mut c_char, dir_name.as_ptr());
            assert!(!shortened.is_null());
            let res = std::ffi::CStr::from_ptr(shortened).to_str().unwrap();
            assert_eq!(res, "docs/file.txt");
        }
    }

    #[test]
    fn test_path_try_shorten_fname() {
        let full_path = CString::new("/home/user/docs/file.txt").unwrap();
        let full_path_ptr = full_path.into_raw();
        unsafe {
            let shortened = path_try_shorten_fname(full_path_ptr);
            let res = std::ffi::CStr::from_ptr(shortened).to_str().unwrap();
            assert_eq!(res, "docs/file.txt");
            let _ = CString::from_raw(full_path_ptr);
        }
    }

    #[test]
    fn test_path_fix_case() {
        let name = CString::new("dir/file.txt").unwrap();
        let name_ptr = name.into_raw();
        unsafe {
            path_fix_case(name_ptr);
            let name_fixed = std::ffi::CStr::from_ptr(name_ptr).to_str().unwrap();
            assert_eq!(name_fixed, "dir/FILE.txt");
            let _ = CString::from_raw(name_ptr);
        }
    }

    #[test]
    fn test_path_full_compare() {
        let s1 = CString::new("exist_file1").unwrap();
        let s2 = CString::new("exist_file2").unwrap();
        let s3 = CString::new("missing_file").unwrap();
        unsafe {
            let cmp = path_full_compare(s1.as_ptr(), s1.as_ptr(), false, false);
            assert!(matches!(cmp, FileComparison::EqualFiles));

            let cmp2 = path_full_compare(s1.as_ptr(), s2.as_ptr(), false, false);
            assert!(matches!(cmp2, FileComparison::DifferentFiles));

            let cmp3 = path_full_compare(s3.as_ptr(), s3.as_ptr(), true, false);
            assert!(matches!(cmp3, FileComparison::EqualFileNames));
        }
    }

    #[test]
    fn test_vim_fullname() {
        let fname = CString::new("docs/file.txt").unwrap();
        let mut buf = [0; MAXPATHL];
        unsafe {
            let res = vim_FullName(fname.as_ptr(), buf.as_mut_ptr(), MAXPATHL, false);
            assert_eq!(res, 1); // OK
            let res_str = std::ffi::CStr::from_ptr(buf.as_ptr()).to_str().unwrap();
            assert_eq!(res_str, "/home/user/docs/file.txt");
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn xmalloc(size: usize) -> *mut c_char {
        let layout = std::alloc::Layout::from_size_align(size + 8, 8).unwrap();
        unsafe {
            let ptr = std::alloc::alloc(layout);
            *(ptr as *mut usize) = size;
            ptr.add(8) as *mut c_char
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn xfree(ptr: *mut c_char) {
        if ptr.is_null() {
            return;
        }
        unsafe {
            let orig_ptr = ptr.offset(-8) as *mut u8;
            let size = *(orig_ptr as *mut usize);
            let layout = std::alloc::Layout::from_size_align(size + 8, 8).unwrap();
            std::alloc::dealloc(orig_ptr, layout);
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn xrealloc(ptr: *mut c_char, size: usize) -> *mut c_char {
        if ptr.is_null() {
            return unsafe { xmalloc(size) };
        }
        unsafe {
            let orig_ptr = ptr.offset(-8) as *mut u8;
            let old_size = *(orig_ptr as *mut usize);
            let layout = std::alloc::Layout::from_size_align(old_size + 8, 8).unwrap();
            let new_ptr = std::alloc::realloc(orig_ptr, layout, size + 8);
            *(new_ptr as *mut usize) = size;
            new_ptr.add(8) as *mut c_char
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn xstrdup(s: *const c_char) -> *mut c_char {
        unsafe {
            let len = strlen(s);
            let dest = xmalloc(len + 1);
            std::ptr::copy_nonoverlapping(s, dest, len + 1);
            dest
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn os_isdir(name: *const c_char) -> bool {
        let s = unsafe { std::ffi::CStr::from_ptr(name) }.to_str().unwrap_or("");
        s.contains("dir") || s == "."
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn os_fileinfo(name: *const c_char, _file_info: *mut FileInfo) -> bool {
        let s = unsafe { std::ffi::CStr::from_ptr(name) }.to_str().unwrap_or("");
        s.contains("exist") || s == "." || (s.starts_with('/') && !s.contains("missing"))
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn mb_toupper(a: c_int) -> c_int {
        if a >= b'a' as c_int && a <= b'z' as c_int {
            a - 32
        } else {
            a
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn mb_tolower(a: c_int) -> c_int {
        if a >= b'A' as c_int && a <= b'Z' as c_int {
            a + 32
        } else {
            a
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn utf_ptr2char(p: *const c_char) -> c_int {
        if p.is_null() {
            return 0;
        }
        unsafe { *p as c_int }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn utfc_ptr2len(p: *const c_char) -> c_int {
        if p.is_null() || unsafe { *p } == 0 {
            return 0;
        }
        let b = unsafe { *p as u8 };
        if b < 0x80 {
            1
        } else if (b & 0xE0) == 0xC0 {
            2
        } else if (b & 0xF0) == 0xE0 {
            3
        } else if (b & 0xF8) == 0xF0 {
            4
        } else {
            1
        }
    }

    #[test]
    fn test_shorten_dir() {
        let path = CString::new("/home/user/docs/file.txt").unwrap();
        let ptr = path.into_raw();
        unsafe {
            shorten_dir(ptr);
            let res = std::ffi::CStr::from_ptr(ptr).to_str().unwrap();
            #[cfg(unix)]
            assert_eq!(res, "/h/u/d/file.txt");
            let _ = CString::from_raw(ptr);
        }
    }

    #[test]
    fn test_dir_of_file_exists() {
        let path1 = CString::new("/home/user/dir/file.txt").unwrap();
        let path2 = CString::new("file.txt").unwrap();
        let ptr1 = path1.into_raw();
        let ptr2 = path2.into_raw();
        unsafe {
            assert!(dir_of_file_exists(ptr1));
            assert!(dir_of_file_exists(ptr2));
            let _ = CString::from_raw(ptr1);
            let _ = CString::from_raw(ptr2);
        }
    }

    #[test]
    fn test_concat_fnames() {
        let f1 = CString::new("/home/user").unwrap();
        let f2 = CString::new("docs").unwrap();
        unsafe {
            let res = concat_fnames(f1.as_ptr(), f2.as_ptr(), true);
            let res_str = std::ffi::CStr::from_ptr(res).to_str().unwrap();
            #[cfg(unix)]
            assert_eq!(res_str, "/home/user/docs");
            xfree(res);
        }
    }

    #[test]
    fn test_add_pathsep() {
        let mut buf = [0 as c_char; 100];
        let p = CString::new("path").unwrap();
        unsafe {
            std::ptr::copy_nonoverlapping(p.as_ptr(), buf.as_mut_ptr(), 5);
            assert!(add_pathsep(buf.as_mut_ptr()));
            let res_str = std::ffi::CStr::from_ptr(buf.as_ptr()).to_str().unwrap();
            #[cfg(unix)]
            assert_eq!(res_str, "path/");
        }
    }

    #[test]
    fn test_simplify_filename() {
        let path = CString::new("/home/user/../user/docs/./file.txt").unwrap();
        let ptr = path.into_raw();
        unsafe {
            let new_len = simplify_filename(ptr);
            let res = std::ffi::CStr::from_ptr(ptr).to_str().unwrap();
            #[cfg(unix)]
            assert_eq!(res, "/home/user/docs/file.txt");
            assert_eq!(new_len, res.len());
            let _ = CString::from_raw(ptr);
        }
    }

    #[test]
    fn test_pathcmp() {
        let p1 = CString::new("/home/user/docs").unwrap();
        let p2 = CString::new("/home/user/docs/").unwrap();
        unsafe {
            assert_eq!(pathcmp(p1.as_ptr(), p2.as_ptr(), -1), 0);
        }
    }
}
