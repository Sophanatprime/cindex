use std::{cmp::Ordering, ffi::*, ptr::null};

use crate::style::*;
use bstr::{BStr, ByteSlice};

#[repr(C)]
pub struct StrRef {
    pub ptr: *const u8,
    pub len: usize,
}

impl StrRef {
    pub fn null() -> Self {
        StrRef {
            ptr: null(),
            len: 0,
        }
    }

    pub fn from_str(s: &str) -> Self {
        StrRef {
            ptr: s.as_ptr(),
            len: s.len(),
        }
    }

    pub unsafe fn as_str_unchecked<'s>(&self) -> &'s str {
        unsafe {
            let b = std::slice::from_raw_parts(self.ptr, self.len);
            std::str::from_utf8_unchecked(b)
        }
    }

    pub unsafe fn as_str<'s>(&self) -> Option<&'s str> {
        unsafe {
            let b = std::slice::from_raw_parts(self.ptr, self.len);
            std::str::from_utf8(b).ok()
        }
    }

    pub fn move_step(&mut self, len: usize) {
        let len = len.min(self.len);
        unsafe {
            self.ptr = self.ptr.add(len);
            self.len -= len;
        }
    }

    pub fn with_str(&mut self, s: &str) {
        self.ptr = s.as_ptr();
        self.len = s.len();
    }

    pub fn clear(&mut self) {
        self.ptr = null();
        self.len = 0;
    }
}

pub struct Writer(pub Box<dyn std::io::Write>);

#[repr(C)]
pub struct Api {
    pub idx_input_read_all:
        unsafe extern "C" fn(*const IstInputStyle, *mut Vec<IndexEntry>, *const StrRef) -> bool,
    pub ikv_input_read_all:
        unsafe extern "C" fn(*const IstInputStyle, *mut Vec<IndexEntry>, *const StrRef) -> bool,
    pub write_to_output: unsafe extern "C" fn(*mut Writer, *const u8, usize) -> bool,
    pub log: unsafe extern "C" fn(u8, *const u8, usize) -> bool,
    pub ist_input: *const IstInputStyle,
    pub ist_input_fn: ist::IstInputFn,
    pub ist_output: *const IstOutputStyle,
    pub ist_output_fn: ist::IstOutputFn,
    pub strref: strref::StrRefFn,
    pub utf8char: utf8char::Utf8CharFn,
    pub index: index::IndexFn,
    pub data: data::DataFn,
    pub pcre2: pcre2::Pcre2Fn,
}

impl Default for Api {
    fn default() -> Self {
        Api {
            idx_input_read_all,
            ikv_input_read_all,
            write_to_output,
            log,
            ist_input: null(),
            ist_input_fn: Default::default(),
            ist_output: null(),
            ist_output_fn: Default::default(),
            strref: Default::default(),
            utf8char: Default::default(),
            index: Default::default(),
            data: Default::default(),
            pcre2: Default::default(),
        }
    }
}

impl Api {
    pub fn set_ist_input(&mut self, ist_input: *const IstInputStyle) -> &mut Self {
        self.ist_input = ist_input;
        self
    }

    pub fn set_ist_output(&mut self, ist_output: *const IstOutputStyle) -> &mut Self {
        self.ist_output = ist_output;
        self
    }
}

pub unsafe extern "C" fn idx_input_read_all(
    ist_input: *const IstInputStyle,
    arr: *mut Vec<IndexEntry>,
    s: *const StrRef,
) -> bool {
    if arr.is_null() || ist_input.is_null() || s.is_null() {
        return false;
    }

    let ist = unsafe { &*ist_input };
    let s = unsafe { (&*s).as_str_unchecked() };
    let mut ok_lines = 0;
    let mut err_lines = 0;
    let mut file_lines = 0;

    for (i, line) in s.lines().enumerate() {
        file_lines += 1;
        match IndexEntry::parse_index_line(ist, line) {
            Ok(Some(entry)) => {
                unsafe { (*arr).push(entry) };
                ok_lines += 1;
            }
            Ok(None) => {}
            Err(e) => {
                log::error!(target: "cindex", "line[{}] {}", i + 1, e);
                err_lines += 1;
            }
        }
    }

    log::info!(
        target: "cindex",
        "Found {} lines, accpet {} lines, reject {} lines.",
        file_lines, ok_lines, err_lines
    );
    true
}

pub unsafe extern "C" fn ikv_input_read_all(
    ist_input: *const IstInputStyle,
    arr: *mut Vec<IndexEntry>,
    s: *const StrRef,
) -> bool {
    struct StrPos<'s> {
        s: &'s str,
        index: usize,
    }

    impl<'s> AsRef<str> for StrPos<'s> {
        fn as_ref(&self) -> &str {
            self.s
        }
    }

    if arr.is_null() || ist_input.is_null() || s.is_null() {
        return false;
    }

    let ist = unsafe { &*ist_input };
    let s = unsafe { (&*s).as_str_unchecked() };
    let mut ok_lines = 0;
    let mut err_lines = 0;
    let mut blocks = 0;

    let mut lines = s
        .lines()
        .enumerate()
        .map(|(index, s)| StrPos { s, index })
        .peekable();

    loop {
        let Some(next) = lines.peek() else {
            break;
        };
        let start_i = next.index;
        blocks += 1;
        match IndexEntry::from_ikv(ist, &mut lines) {
            Ok(Some(entry)) => {
                unsafe { (*arr).push(entry) };
                ok_lines += 1;
            }
            Ok(None) => {}
            Err(e) => {
                log::error!(target: "cindex", "line[{}] {}", start_i + 1, e);
                err_lines += 1;
            }
        }
    }

    log::info!(
        target: "cindex",
        "Found {} blocks, accpet {} blocks, reject {} blocks.",
        blocks, ok_lines, err_lines
    );
    true
}

unsafe extern "C" fn write_to_output(buf: *mut Writer, ptr: *const u8, len: usize) -> bool {
    unsafe {
        let bs = BStr::new(std::slice::from_raw_parts(ptr, len));
        if let Err(e) = bs.to_str() {
            log::error!(target: "cindex", "{}, raised at {}", e, bs);
            return false;
        };
        (*buf)
            .0
            .write_all(bs.as_bytes())
            .inspect_err(|e| log::error!(target: "cindex", "{}", e))
            .is_ok()
    }
}

unsafe extern "C" fn log(level: u8, ptr: *const u8, len: usize) -> bool {
    unsafe {
        let Some(s) = StrRef { ptr, len }.as_str() else {
            return false;
        };
        let level = match level {
            1 => log::Level::Error,
            2 => log::Level::Warn,
            3 => log::Level::Info,
            4 => log::Level::Debug,
            5 => log::Level::Trace,
            _ => return false,
        };
        log::log!(target: "cindex", level, "{}", s);
        true
    }
}

mod ist {
    use super::*;

    #[repr(C)]
    pub struct IstInputFn {
        pub keyword: unsafe extern "C" fn(*const IstInputStyle, *mut StrRef) -> bool,
        pub arg_open: unsafe extern "C" fn(*const IstInputStyle) -> u32,
        pub arg_close: unsafe extern "C" fn(*const IstInputStyle) -> u32,
        pub range_open: unsafe extern "C" fn(*const IstInputStyle) -> u32,
        pub range_close: unsafe extern "C" fn(*const IstInputStyle) -> u32,
        pub level: unsafe extern "C" fn(*const IstInputStyle) -> u32,
        pub actual: unsafe extern "C" fn(*const IstInputStyle) -> u32,
        pub encap: unsafe extern "C" fn(*const IstInputStyle) -> u32,
        pub quote: unsafe extern "C" fn(*const IstInputStyle) -> u32,
        pub escape: unsafe extern "C" fn(*const IstInputStyle) -> u32,
        pub page_compositor: unsafe extern "C" fn(*const IstInputStyle, *mut StrRef) -> bool,
        pub comment: unsafe extern "C" fn(*const IstInputStyle) -> u32,
        pub separator: unsafe extern "C" fn(*const IstInputStyle, *mut StrRef) -> bool,
    }

    impl Default for IstInputFn {
        fn default() -> Self {
            IstInputFn {
                keyword,
                arg_open,
                arg_close,
                range_open,
                range_close,
                level,
                actual,
                encap,
                quote,
                escape,
                page_compositor,
                comment,
                separator,
            }
        }
    }

    #[repr(C)]
    pub struct IstOutputFn {
        pub preamble: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub postamble: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub group_skip: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub heading_prefix: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub heading_suffix: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub headings_flag: unsafe extern "C" fn(*const IstOutputStyle) -> i32,
        pub numhead_positive: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub numhead_negative: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub symhead_positive: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub symhead_negative: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub item_0: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub item_1: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub item_2: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub item_01: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub item_x1: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub item_12: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub item_x2: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub delim_0: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub delim_1: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub delim_2: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub delim_n: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub delim_r: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub delim_t: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub encap_prefix: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub encap_infix: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub encap_suffix: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub page_precedence: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub suffix_2p: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub suffix_3p: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub suffix_mp: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub stroke_prefix: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub stroke_suffix: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub radical_prefix: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub radical_suffix: unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub radical_simplified_flag: unsafe extern "C" fn(*const IstOutputStyle) -> i32,
        pub radical_simplified_prefix:
            unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub radical_simplified_separator:
            unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
        pub radical_simplified_suffix:
            unsafe extern "C" fn(*const IstOutputStyle, *mut StrRef) -> bool,
    }

    impl Default for IstOutputFn {
        fn default() -> Self {
            IstOutputFn {
                preamble,
                postamble,
                group_skip,
                heading_prefix,
                heading_suffix,
                headings_flag,
                numhead_positive,
                numhead_negative,
                symhead_positive,
                symhead_negative,
                item_0,
                item_1,
                item_2,
                item_01,
                item_x1,
                item_12,
                item_x2,
                delim_0,
                delim_1,
                delim_2,
                delim_n,
                delim_r,
                delim_t,
                encap_prefix,
                encap_infix,
                encap_suffix,
                page_precedence,
                suffix_2p,
                suffix_3p,
                suffix_mp,
                stroke_prefix,
                stroke_suffix,
                radical_prefix,
                radical_suffix,
                radical_simplified_flag,
                radical_simplified_prefix,
                radical_simplified_separator,
                radical_simplified_suffix,
            }
        }
    }

    macro_rules! gen_ist_fn {
        ($stru:ty [ $func:ident : char ]) => {
            pub unsafe extern "C" fn $func(ist: *const $stru) -> u32 {
                if ist.is_null() {
                    u32::MAX
                } else {
                    unsafe { (&*ist).$func as _ }
                }
            }
        };
        ($stru:ty [ $func:ident : i32 ]) => {
            pub unsafe extern "C" fn $func(ist: *const $stru) -> i32 {
                if ist.is_null() {
                    i32::MAX
                } else {
                    unsafe { (&*ist).$func }
                }
            }
        };
        ($stru:ty [ $func:ident : String ]) => {
            pub unsafe extern "C" fn $func(ist: *const $stru, out: *mut StrRef) -> bool {
                if ist.is_null() || out.is_null() {
                    return false;
                }
                unsafe {
                    (&mut *out).with_str(&(&*ist).$func);
                }
                true
            }
        };
    }

    gen_ist_fn!(IstInputStyle [keyword: String]);
    gen_ist_fn!(IstInputStyle [arg_open: char]);
    gen_ist_fn!(IstInputStyle [arg_close: char]);
    gen_ist_fn!(IstInputStyle [range_open: char]);
    gen_ist_fn!(IstInputStyle [range_close: char]);
    gen_ist_fn!(IstInputStyle [level: char]);
    gen_ist_fn!(IstInputStyle [actual: char]);
    gen_ist_fn!(IstInputStyle [encap: char]);
    gen_ist_fn!(IstInputStyle [quote: char]);
    gen_ist_fn!(IstInputStyle [escape: char]);
    gen_ist_fn!(IstInputStyle [page_compositor: String]);
    gen_ist_fn!(IstInputStyle [comment: char]);
    gen_ist_fn!(IstInputStyle [separator: String]);
    gen_ist_fn!(IstOutputStyle [preamble: String]);
    gen_ist_fn!(IstOutputStyle [postamble: String]);
    gen_ist_fn!(IstOutputStyle [group_skip: String]);
    gen_ist_fn!(IstOutputStyle [heading_prefix: String]);
    gen_ist_fn!(IstOutputStyle [heading_suffix: String]);
    gen_ist_fn!(IstOutputStyle [headings_flag: i32]);
    gen_ist_fn!(IstOutputStyle [numhead_positive: String]);
    gen_ist_fn!(IstOutputStyle [numhead_negative: String]);
    gen_ist_fn!(IstOutputStyle [symhead_positive: String]);
    gen_ist_fn!(IstOutputStyle [symhead_negative: String]);
    gen_ist_fn!(IstOutputStyle [item_0: String]);
    gen_ist_fn!(IstOutputStyle [item_1: String]);
    gen_ist_fn!(IstOutputStyle [item_2: String]);
    gen_ist_fn!(IstOutputStyle [item_01: String]);
    gen_ist_fn!(IstOutputStyle [item_x1: String]);
    gen_ist_fn!(IstOutputStyle [item_12: String]);
    gen_ist_fn!(IstOutputStyle [item_x2: String]);
    gen_ist_fn!(IstOutputStyle [delim_0: String]);
    gen_ist_fn!(IstOutputStyle [delim_1: String]);
    gen_ist_fn!(IstOutputStyle [delim_2: String]);
    gen_ist_fn!(IstOutputStyle [delim_n: String]);
    gen_ist_fn!(IstOutputStyle [delim_r: String]);
    gen_ist_fn!(IstOutputStyle [delim_t: String]);
    gen_ist_fn!(IstOutputStyle [encap_prefix: String]);
    gen_ist_fn!(IstOutputStyle [encap_infix: String]);
    gen_ist_fn!(IstOutputStyle [encap_suffix: String]);
    gen_ist_fn!(IstOutputStyle [page_precedence: String]);
    gen_ist_fn!(IstOutputStyle [suffix_2p: String]);
    gen_ist_fn!(IstOutputStyle [suffix_3p: String]);
    gen_ist_fn!(IstOutputStyle [suffix_mp: String]);
    gen_ist_fn!(IstOutputStyle [stroke_prefix: String]);
    gen_ist_fn!(IstOutputStyle [stroke_suffix: String]);
    gen_ist_fn!(IstOutputStyle [radical_prefix: String]);
    gen_ist_fn!(IstOutputStyle [radical_suffix: String]);
    gen_ist_fn!(IstOutputStyle [radical_simplified_flag: i32]);
    gen_ist_fn!(IstOutputStyle [radical_simplified_prefix: String]);
    gen_ist_fn!(IstOutputStyle [radical_simplified_separator: String]);
    gen_ist_fn!(IstOutputStyle [radical_simplified_suffix: String]);
}

mod strref {
    use super::*;
    use icu_casemap::{CaseMapper, CaseMapperBorrowed};
    use unicode_segmentation::UnicodeSegmentation;

    #[repr(C)]
    pub struct StrRefFn {
        pub strlen: unsafe extern "C" fn(*const u8) -> usize,
        pub from_buf: unsafe extern "C" fn(*const u8, usize, *mut StrRef) -> bool,
        pub cmp: unsafe extern "C" fn(*const StrRef, *const StrRef) -> i64,
        pub cmp_buf: unsafe extern "C" fn(*const StrRef, *const u8, usize) -> i64,
        pub cmp_ignore_case: unsafe extern "C" fn(*const u8, usize, *const u8, usize) -> i64,
        pub slice:
            unsafe extern "C" fn(*const StrRef, start: usize, end: usize, *mut StrRef) -> bool,
        pub head: unsafe extern "C" fn(*const u8) -> u32,
        pub char_next: unsafe extern "C" fn(*mut StrRef) -> u32,
        pub line_next: unsafe extern "C" fn(*mut StrRef, *mut StrRef) -> bool,
        pub split_char_next: unsafe extern "C" fn(*mut StrRef, u32, *mut StrRef) -> bool,
        pub split_str_next:
            unsafe extern "C" fn(*mut StrRef, *const u8, usize, *mut StrRef) -> bool,
        pub split_spaces_next: unsafe extern "C" fn(*mut StrRef, *mut StrRef) -> bool,
        pub graphemes_next: unsafe extern "C" fn(*mut StrRef, *mut StrRef) -> bool,
        pub unicode_words_next: unsafe extern "C" fn(*mut StrRef, *mut StrRef) -> bool,
        pub split_word_bounds_next: unsafe extern "C" fn(*mut StrRef, *mut StrRef) -> bool,
        pub trim_ascii_spaces: unsafe extern "C" fn(*const StrRef, bool, bool, *mut StrRef) -> bool,
    }

    impl Default for StrRefFn {
        fn default() -> Self {
            StrRefFn {
                strlen,
                from_buf,
                cmp,
                cmp_buf,
                cmp_ignore_case,
                slice,
                head,
                char_next,
                line_next,
                split_char_next,
                split_str_next,
                split_spaces_next,
                graphemes_next,
                unicode_words_next,
                split_word_bounds_next,
                trim_ascii_spaces,
            }
        }
    }

    pub unsafe extern "C" fn strlen(ptr: *const u8) -> usize {
        let mut len = 0;
        while unsafe { *ptr.add(len) } != 0 {
            len += 1;
        }
        len
    }

    pub unsafe extern "C" fn from_buf(ptr: *const u8, len: usize, out: *mut StrRef) -> bool {
        if ptr.is_null() || out.is_null() {
            false
        } else {
            unsafe {
                (*out).ptr = ptr;
                (*out).len = len;
            }
            true
        }
    }

    pub unsafe extern "C" fn cmp(lhs: *const StrRef, rhs: *const StrRef) -> i64 {
        unsafe {
            let lhs = &(*lhs);
            let rhs = &(*rhs);
            if lhs.ptr.is_null() || rhs.ptr.is_null() {
                return -2;
            }
            let lhs = BStr::new(std::slice::from_raw_parts(lhs.ptr, lhs.len));
            let rhs = BStr::new(std::slice::from_raw_parts(rhs.ptr, rhs.len));
            match lhs.cmp(rhs) {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            }
        }
    }

    pub unsafe extern "C" fn cmp_buf(lhs: *const StrRef, ptr: *const u8, len: usize) -> i64 {
        unsafe { cmp(lhs, &StrRef { ptr, len } as *const _) }
    }

    pub unsafe extern "C" fn cmp_ignore_case(
        lhs_ptr: *const u8,
        lhs_len: usize,
        rhs_ptr: *const u8,
        rhs_len: usize,
    ) -> i64 {
        static CASEMAPPER: CaseMapperBorrowed<'static> = CaseMapper::new();

        if lhs_ptr.is_null() || rhs_ptr.is_null() {
            return -2;
        }

        unsafe {
            let Some(lhs) = StrRef {
                ptr: lhs_ptr,
                len: lhs_len,
            }
            .as_str() else {
                return -2;
            };
            let Some(rhs) = StrRef {
                ptr: rhs_ptr,
                len: rhs_len,
            }
            .as_str() else {
                return -2;
            };
            match CASEMAPPER
                .fold_string(lhs)
                .cmp(&CASEMAPPER.fold_string(rhs))
            {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            }
        }
    }

    pub unsafe extern "C" fn slice(
        s: *const StrRef,
        start: usize,
        end: usize,
        out: *mut StrRef,
    ) -> bool {
        unsafe {
            let s = &*s;
            if s.ptr.is_null() {
                return false;
            }

            if start > end || start > s.len {
                false
            } else {
                let ptr = s.ptr.add(start);
                if !((*ptr as i8) >= -0x40) {
                    return false;
                }
                if end >= s.len {
                    out.replace(StrRef {
                        ptr,
                        len: s.len - start,
                    });
                } else {
                    let end_ptr = s.ptr.add(end);
                    if (*end_ptr as i8) >= -0x40 {
                        out.replace(StrRef {
                            ptr,
                            len: end - start,
                        });
                    } else {
                        return false;
                    }
                }
                true
            }
        }
    }

    pub unsafe extern "C" fn head(s: *const u8) -> u32 {
        if s.is_null() {
            return u32::MAX;
        }
        let b0 = unsafe { *s };
        if b0 < 0x80 {
            return b0 as u32;
        }

        let (len, min_code) = match b0 {
            0xC2..=0xDF => (2, 0x80),
            0xE0..=0xEF => (3, 0x800),
            0xF0..=0xF4 => (4, 0x10000),
            _ => return u32::MAX,
        };

        let mut code = match len {
            2 => (b0 & 0x1F) as u32,
            3 => (b0 & 0x0F) as u32,
            4 => (b0 & 0x07) as u32,
            _ => unreachable!(),
        };

        for i in 1..len {
            let byte = unsafe { *s.add(i) };
            if !(0x80..=0xBF).contains(&byte) {
                return u32::MAX;
            }
            code = (code << 6) | (byte as u32 & 0x3F);
        }

        if code < min_code {
            return u32::MAX;
        }
        if code > 0x10FFFF || (0xD800..=0xDFFF).contains(&code) {
            return u32::MAX;
        }

        code
    }

    pub unsafe extern "C" fn char_next(s: *mut StrRef) -> u32 {
        unsafe {
            if s.is_null() || (*s).ptr.is_null() {
                return u32::MAX;
            }

            let bs = std::slice::from_raw_parts((*s).ptr, (*s).len);
            match BStr::new(bs).chars().next() {
                Some(c) => {
                    (*s).move_step(c.len_utf8());
                    c as u32
                }
                None => u32::MAX,
            }
        }
    }

    pub unsafe extern "C" fn line_next(s: *mut StrRef, out: *mut StrRef) -> bool {
        unsafe {
            if s.is_null() || (*s).ptr.is_null() {
                return false;
            }

            let bs = std::slice::from_raw_parts((*s).ptr, (*s).len);
            let mut bs = BStr::new(bs).lines();
            if let Some(l) = bs.next() {
                (*out).ptr = l.as_ptr();
                (*out).len = l.len();
                let rest = bs.as_bytes();
                (*s).ptr = rest.as_ptr();
                (*s).len = rest.len();
                true
            } else {
                *s = StrRef::null();
                false
            }
        }
    }

    pub unsafe extern "C" fn split_char_next(s: *mut StrRef, c: u32, out: *mut StrRef) -> bool {
        let c = char::from_u32(c);
        unsafe {
            if s.is_null() || (&*s).len == 0 || c.is_none() {
                return false;
            }
            let c = c.unwrap_unchecked();
            let bs = (&*s).as_str_unchecked();
            let mut spl = bs.split(c);
            match spl.next() {
                Some(spl) => {
                    (*out).with_str(spl);
                    let len = spl.len() + c.len_utf8();
                    (*s).move_step(len);
                    true
                }
                None => {
                    *s = StrRef::null();
                    false
                }
            }
        }
    }

    pub unsafe extern "C" fn split_str_next(
        s: *mut StrRef,
        ptr: *const u8,
        len: usize,
        out: *mut StrRef,
    ) -> bool {
        unsafe {
            if s.is_null() || (&*s).len == 0 || ptr.is_null() {
                return false;
            }
            let bs = (&*s).as_str_unchecked();
            let Some(pat) = StrRef { ptr, len }.as_str() else {
                return false;
            };
            let mut spl = bs.split(pat);
            match spl.next() {
                Some(spl) => {
                    (*out).with_str(spl);
                    let len = spl.len() + pat.len();
                    (*s).move_step(len);
                    true
                }
                None => {
                    *s = StrRef::null();
                    false
                }
            }
        }
    }

    pub unsafe extern "C" fn split_spaces_next(s: *mut StrRef, out: *mut StrRef) -> bool {
        unsafe {
            if s.is_null() {
                return false;
            }
            let bs = (&*s).as_str_unchecked();
            let mut spl = bs.split_ascii_whitespace();
            match spl.next() {
                Some(sp_removed) => {
                    (*out).with_str(sp_removed);
                    (*s).move_step(
                        spl.next()
                            .map(|s| s.as_ptr().offset_from_unsigned(sp_removed.as_ptr()))
                            .unwrap_or(bs.len()),
                    );
                    true
                }
                None => {
                    *s = StrRef::null();
                    false
                }
            }
        }
    }

    pub unsafe extern "C" fn graphemes_next(s: *mut StrRef, out: *mut StrRef) -> bool {
        unsafe {
            if s.is_null() {
                return false;
            }
            let Some(bs) = (*s).as_str() else {
                return false;
            };
            let mut ge_iter = bs.graphemes(true);
            match ge_iter.next() {
                Some(ge) => {
                    (*out).with_str(ge);
                    (*s).with_str(ge_iter.as_str());
                    true
                }
                None => {
                    *s = StrRef::null();
                    false
                }
            }
        }
    }

    pub unsafe extern "C" fn unicode_words_next(s: *mut StrRef, out: *mut StrRef) -> bool {
        unsafe {
            if s.is_null() {
                return false;
            }
            let Some(bs) = (*s).as_str() else {
                return false;
            };
            let mut ge_iter = bs.unicode_words();
            match ge_iter.next() {
                Some(ge) => {
                    (*out).with_str(ge);
                    (*s).move_step((*out).ptr.add((*out).len).offset_from_unsigned((*s).ptr));
                    true
                }
                None => {
                    *s = StrRef::null();
                    false
                }
            }
        }
    }

    pub unsafe extern "C" fn split_word_bounds_next(s: *mut StrRef, out: *mut StrRef) -> bool {
        unsafe {
            if s.is_null() {
                return false;
            }
            let Some(bs) = (*s).as_str() else {
                return false;
            };
            let mut ge_iter = bs.split_word_bounds();
            match ge_iter.next() {
                Some(ge) => {
                    (*out).with_str(ge);
                    (*s).with_str(ge_iter.as_str());
                    true
                }
                None => {
                    *s = StrRef::null();
                    false
                }
            }
        }
    }

    pub unsafe extern "C" fn trim_ascii_spaces(
        s: *const StrRef,
        left: bool,
        right: bool,
        out: *mut StrRef,
    ) -> bool {
        unsafe {
            if s.is_null() || (*s).ptr.is_null() {
                return false;
            }
            let mut bs = std::slice::from_raw_parts((*s).ptr, (*s).len);
            if left {
                bs = bs.trim_ascii_start();
            }
            if right {
                bs = bs.trim_ascii_end();
            }
            (*out).ptr = bs.as_ptr();
            (*out).len = bs.len();
            true
        }
    }
}

mod utf8char {
    use icu_casemap::{CaseMapper, CaseMapperBorrowed};
    use icu_collections::codepointtrie::TrieValue;
    use icu_properties::{
        CodePointMapData, CodePointMapDataBorrowed, CodePointSetData, CodePointSetDataBorrowed,
        props::{GeneralCategory, Script},
    };

    #[repr(C)]
    pub struct Utf8CharFn {
        pub from_i64: unsafe extern "C" fn(i64) -> u32,
        pub is_valid: unsafe extern "C" fn(u32) -> bool,
        pub encode_utf8: unsafe extern "C" fn(u32, *mut [u8; 5]) -> bool,
        pub len_utf8: unsafe extern "C" fn(u32) -> usize,
        pub script: unsafe extern "C" fn(u32) -> u32,
        pub general_category: unsafe extern "C" fn(u32) -> u8,
        pub is_alphabetic: unsafe extern "C" fn(u32) -> bool,
        pub is_numeric: unsafe extern "C" fn(u32) -> bool,
        pub is_math: unsafe extern "C" fn(u32) -> bool,
        pub is_ideographic: unsafe extern "C" fn(u32) -> bool,
        pub is_visible: unsafe extern "C" fn(u32) -> bool,
        pub simple_fold: unsafe extern "C" fn(u32) -> u32,
    }

    impl Default for Utf8CharFn {
        fn default() -> Self {
            Utf8CharFn {
                from_i64,
                is_valid,
                encode_utf8,
                len_utf8,
                script,
                general_category,
                is_alphabetic,
                is_numeric,
                is_math,
                is_ideographic,
                is_visible,
                simple_fold,
            }
        }
    }

    pub unsafe extern "C" fn from_i64(n: i64) -> u32 {
        match u32::try_from(n).ok().and_then(char::from_u32) {
            Some(c) => c as u32,
            None => n.clamp(0, 0xffff_ffff) as _,
        }
    }

    pub unsafe extern "C" fn is_valid(n: u32) -> bool {
        char::from_u32(n).is_some()
    }

    pub unsafe extern "C" fn encode_utf8(n: u32, buf: *mut [u8; 5]) -> bool {
        match char::from_u32(n) {
            Some(c) => {
                c.encode_utf8(unsafe { &mut *buf });
                true
            }
            None => false,
        }
    }

    pub unsafe extern "C" fn len_utf8(n: u32) -> usize {
        char::from_u32(n).map(|c| c.len_utf8()).unwrap_or(0)
    }

    pub unsafe extern "C" fn script(n: u32) -> u32 {
        static SCRIPT: CodePointMapDataBorrowed<Script> = CodePointMapData::new();
        SCRIPT.get32(n).to_u32()
    }

    pub unsafe extern "C" fn general_category(n: u32) -> u8 {
        static GC: CodePointMapDataBorrowed<GeneralCategory> = CodePointMapData::new();
        GC.get32(n) as _
    }

    pub unsafe extern "C" fn is_alphabetic(n: u32) -> bool {
        char::from_u32(n).is_some_and(|c| c.is_alphabetic())
    }

    pub unsafe extern "C" fn is_numeric(n: u32) -> bool {
        char::from_u32(n).is_some_and(|c| c.is_numeric())
    }

    pub unsafe extern "C" fn is_math(n: u32) -> bool {
        use icu_properties::props::Math;
        static MATH: CodePointSetDataBorrowed = CodePointSetData::new::<Math>();
        MATH.contains32(n)
    }

    pub unsafe extern "C" fn is_ideographic(n: u32) -> bool {
        use icu_properties::props::Ideographic;
        static IDEO: CodePointSetDataBorrowed = CodePointSetData::new::<Ideographic>();
        IDEO.contains32(n)
    }

    pub unsafe extern "C" fn is_visible(n: u32) -> bool {
        use icu_properties::props::{DefaultIgnorableCodePoint, Print, WhiteSpace};
        static PRINT: CodePointSetDataBorrowed = CodePointSetData::new::<Print>();
        static SPACE: CodePointSetDataBorrowed = CodePointSetData::new::<WhiteSpace>();
        static IGNORABLE: CodePointSetDataBorrowed =
            CodePointSetData::new::<DefaultIgnorableCodePoint>();
        PRINT.contains32(n) && !IGNORABLE.contains32(n) && !SPACE.contains32(n)
    }

    pub unsafe extern "C" fn simple_fold(n: u32) -> u32 {
        static CASEMAPPER: CaseMapperBorrowed<'static> = CaseMapper::new();
        char::from_u32(n)
            .map(|c| CASEMAPPER.simple_fold(c) as u32)
            .unwrap_or(n)
    }
}

mod index {
    use super::*;
    use crate::{IndexPage, MergedEntry, MergedPage};

    #[repr(C)]
    pub struct IndexFn {
        pub indices_new_boxed: unsafe extern "C" fn() -> *mut Vec<IndexEntry>,
        pub indices_push_boxed_entry:
            unsafe extern "C" fn(*mut Vec<IndexEntry>, *mut IndexEntry) -> bool,

        pub indices_len: unsafe extern "C" fn(*const Vec<IndexEntry>) -> usize,
        pub indices_at: unsafe extern "C" fn(*const Vec<IndexEntry>, usize) -> *const IndexEntry,

        pub entry_as_const: unsafe extern "C" fn(*mut IndexEntry) -> *const IndexEntry,
        pub entry_new_boxed: unsafe extern "C" fn() -> *mut IndexEntry,
        pub entry_new_cloned: unsafe extern "C" fn(*const IndexEntry) -> *mut IndexEntry,
        pub entry_levels_push:
            unsafe extern "C" fn(*mut IndexEntry, *const u8, usize, *const u8, usize) -> bool,
        pub entry_range_set: unsafe extern "C" fn(*mut IndexEntry, *const u8, usize) -> bool,
        pub entry_page_commands_set:
            unsafe extern "C" fn(*mut IndexEntry, *const u8, usize) -> bool,
        pub entry_pages_push: unsafe extern "C" fn(*mut IndexEntry, *const u8, usize, u64) -> bool,
        pub entry_pages_raw_set: unsafe extern "C" fn(*mut IndexEntry, *const u8, usize) -> bool,

        pub entry_levels_count: unsafe extern "C" fn(*const IndexEntry) -> usize,
        pub entry_key_n: unsafe extern "C" fn(*const IndexEntry, usize, *mut StrRef) -> bool,
        pub entry_is_same: unsafe extern "C" fn(*const IndexEntry, *const IndexEntry) -> bool,

        pub merged_levels_count: unsafe extern "C" fn(*const MergedEntry) -> usize,
        pub merged_key_n: unsafe extern "C" fn(*const MergedEntry, usize, *mut StrRef) -> bool,
        pub merged_level_n: unsafe extern "C" fn(*const MergedEntry, usize, *mut StrRef) -> bool,
        pub merged_pages_count: unsafe extern "C" fn(*const MergedEntry) -> usize,
        pub merged_page_command_n:
            unsafe extern "C" fn(*const MergedEntry, usize, *mut StrRef) -> bool,
        pub merged_pages_raw_n:
            unsafe extern "C" fn(*const MergedEntry, usize, *mut StrRef, *mut StrRef) -> bool,
        pub merged_page_span_n: unsafe extern "C" fn(*const MergedEntry, usize) -> usize,
        pub merged_max_same_key:
            unsafe extern "C" fn(*const MergedEntry, *const MergedEntry) -> usize,
    }

    impl Default for IndexFn {
        fn default() -> Self {
            IndexFn {
                indices_new_boxed,
                indices_push_boxed_entry,
                indices_len,
                indices_at,
                entry_as_const,
                entry_new_boxed,
                entry_new_cloned,
                entry_levels_push,
                entry_range_set,
                entry_page_commands_set,
                entry_pages_push,
                entry_pages_raw_set,
                entry_levels_count,
                entry_key_n,
                entry_is_same,
                merged_levels_count,
                merged_key_n,
                merged_level_n,
                merged_pages_count,
                merged_page_command_n,
                merged_pages_raw_n,
                merged_page_span_n,
                merged_max_same_key,
            }
        }
    }

    pub unsafe extern "C" fn indices_new_boxed() -> *mut Vec<IndexEntry> {
        Box::into_raw(Box::new(Vec::new()))
    }

    pub unsafe extern "C" fn indices_push_boxed_entry(
        arr: *mut Vec<IndexEntry>,
        entry: *mut IndexEntry,
    ) -> bool {
        if arr.is_null() || entry.is_null() {
            return false;
        }
        unsafe {
            (*arr).push(*Box::from_raw(entry));
        }
        true
    }

    pub unsafe extern "C" fn indices_len(indices: *const Vec<IndexEntry>) -> usize {
        unsafe { (&*indices).len() }
    }

    pub unsafe extern "C" fn indices_at(
        indices: *const Vec<IndexEntry>,
        n: usize,
    ) -> *const IndexEntry {
        unsafe {
            if indices.is_null() {
                return null();
            }
            (&*indices).get(n).map(|e| e as *const _).unwrap_or(null())
        }
    }

    pub unsafe extern "C" fn entry_as_const(entry: *mut IndexEntry) -> *const IndexEntry {
        entry
    }

    pub unsafe extern "C" fn entry_new_boxed() -> *mut IndexEntry {
        Box::into_raw(Box::new(IndexEntry {
            levels: Default::default(),
            range: RangeKind::None,
            page_commands: None,
            pages: Default::default(),
            pages_raw: Default::default(),
        }))
    }

    pub unsafe extern "C" fn entry_new_cloned(entry: *const IndexEntry) -> *mut IndexEntry {
        unsafe { Box::into_raw(Box::new((&*entry).clone())) }
    }

    pub unsafe extern "C" fn entry_levels_push(
        entry: *mut IndexEntry,
        key_ptr: *const u8,
        key_len: usize,
        print_ptr: *const u8,
        print_len: usize,
    ) -> bool {
        let key = StrRef {
            ptr: key_ptr,
            len: key_len,
        };
        let print = StrRef {
            ptr: print_ptr,
            len: print_len,
        };
        if entry.is_null() || print.ptr.is_null() {
            return false;
        }
        unsafe {
            let key = if key.ptr.is_null() {
                None
            } else {
                let Some(key) = key.as_str() else {
                    return false;
                };
                Some(key.to_string())
            };
            let Some(print) = print.as_str() else {
                return false;
            };
            (&mut *entry).levels.push((key, print.to_string()));
        }
        true
    }

    pub unsafe extern "C" fn entry_range_set(
        entry: *mut IndexEntry,
        ptr: *const u8,
        len: usize,
    ) -> bool {
        let range = StrRef { ptr, len };
        if entry.is_null() || range.ptr.is_null() {
            return false;
        }
        unsafe {
            let Some(range) = range.as_str() else {
                return false;
            };
            let range = match range {
                "Open" | "open" | "-1" => RangeKind::Open,
                "None" | "none" | "nil" | "0" => RangeKind::None,
                "Close" | "close" | "1" => RangeKind::Close,
                _ => return false,
            };
            (&mut *entry).range = range;
        }
        true
    }

    pub unsafe extern "C" fn entry_page_commands_set(
        entry: *mut IndexEntry,
        ptr: *const u8,
        len: usize,
    ) -> bool {
        let page_commands = StrRef { ptr, len };
        if entry.is_null() {
            return false;
        }
        unsafe {
            if page_commands.ptr.is_null() {
                (*entry).page_commands = None;
            } else {
                let Some(page_commands) = page_commands.as_str() else {
                    return false;
                };
                (*entry).page_commands = Some(page_commands.into());
            }
        }
        true
    }

    pub unsafe extern "C" fn entry_pages_push(
        entry: *mut IndexEntry,
        kind_ptr: *const u8,
        kind_len: usize,
        kind_val: u64,
    ) -> bool {
        let kind = StrRef {
            ptr: kind_ptr,
            len: kind_len,
        };
        if entry.is_null() || kind.ptr.is_null() {
            return false;
        }
        unsafe {
            let page_num = match kind.as_str_unchecked() {
                "Arabic" => IndexPage::Arabic(kind_val),
                "roman" => IndexPage::roman(kind_val),
                "Roman" => IndexPage::Roman(kind_val),
                "alpha" => IndexPage::alpha(kind_val),
                "Alpha" => IndexPage::Alpha(kind_val),
                "ZhDigits" => IndexPage::ZhDigits(kind_val),
                "ZhNumber" => IndexPage::ZhNumber(kind_val),
                _ => return false,
            };
            (*entry).pages.push(page_num);
        }
        true
    }

    pub unsafe extern "C" fn entry_pages_raw_set(
        entry: *mut IndexEntry,
        ptr: *const u8,
        len: usize,
    ) -> bool {
        let pages_raw = StrRef { ptr, len };
        if entry.is_null() {
            return false;
        }
        unsafe {
            (*entry).pages_raw.clear();
            if !pages_raw.ptr.is_null() {
                let Some(raw) = pages_raw.as_str() else {
                    return false;
                };
                (*entry).pages_raw.push_str(raw);
            }
        }
        true
    }

    pub unsafe extern "C" fn entry_levels_count(entry: *const IndexEntry) -> usize {
        unsafe { (&*entry).levels.len() }
    }

    pub unsafe extern "C" fn entry_key_n(
        entry: *const IndexEntry,
        n: usize,
        out: *mut StrRef,
    ) -> bool {
        unsafe {
            if entry.is_null() || (&*entry).levels.len() <= n {
                return false;
            }

            match &(&*entry).levels[n] {
                (Some(k), _) => (&mut *out).with_str(k),
                (None, k) => (&mut *out).with_str(k),
            }
        }
        true
    }

    pub unsafe extern "C" fn entry_is_same(lhs: *const IndexEntry, rhs: *const IndexEntry) -> bool {
        if lhs.is_null() || rhs.is_null() {
            return false;
        }
        let lhs = unsafe { &*lhs };
        let rhs = unsafe { &*rhs };
        lhs.is_same(rhs)
    }

    pub unsafe extern "C" fn merged_levels_count(entry: *const MergedEntry) -> usize {
        unsafe { (&*entry).levels.len() }
    }

    pub unsafe extern "C" fn merged_key_n(
        entry: *const MergedEntry,
        n: usize,
        out: *mut StrRef,
    ) -> bool {
        unsafe {
            if entry.is_null() || (&*entry).levels.len() <= n {
                return false;
            }

            match &(&*entry).levels[n] {
                (Some(k), _) => (&mut *out).with_str(k),
                (None, k) => (&mut *out).with_str(k),
            }
        }
        true
    }

    pub unsafe extern "C" fn merged_level_n(
        entry: *const MergedEntry,
        n: usize,
        out: *mut StrRef,
    ) -> bool {
        unsafe {
            if entry.is_null() || out.is_null() || (*entry).levels.len() <= n {
                return false;
            }

            (*out).with_str((&*entry).levels[n].1.as_str());
            true
        }
    }

    pub unsafe extern "C" fn merged_pages_count(entry: *const MergedEntry) -> usize {
        unsafe { (&*entry).pages.len() }
    }

    pub unsafe extern "C" fn merged_page_command_n(
        entry: *const MergedEntry,
        n: usize,
        out: *mut StrRef,
    ) -> bool {
        unsafe {
            if entry.is_null() || out.is_null() {
                return false;
            }
            let Some(n) = (&*entry).pages.get(n) else {
                return false;
            };
            match n.page_command() {
                Some(s) => (*out).with_str(s),
                None => (*out).clear(),
            }
            true
        }
    }

    pub unsafe extern "C" fn merged_pages_raw_n(
        entry: *const MergedEntry,
        n: usize,
        o1: *mut StrRef,
        o2: *mut StrRef,
    ) -> bool {
        unsafe {
            if entry.is_null() || (&*entry).pages.len() <= n || o1.is_null() {
                return false;
            }
            match &(&*entry).pages[n] {
                MergedPage::Single { page, .. } => {
                    (*o1).with_str(&page.1);
                }
                MergedPage::Range { start, end, .. } => {
                    if o2.is_null() {
                        return false;
                    }
                    (*o1).with_str(&start.1);
                    (*o2).with_str(&end.1);
                }
            }
            true
        }
    }

    pub unsafe extern "C" fn merged_page_span_n(entry: *const MergedEntry, n: usize) -> usize {
        unsafe {
            if entry.is_null() || (&*entry).pages.len() <= n {
                return 0;
            }
            match &(&*entry).pages[n] {
                MergedPage::Single { page, .. } => {
                    if page.0.is_empty() {
                        return 0;
                    }
                    1
                }
                MergedPage::Range { start, end, .. } => {
                    if start.0.is_empty() || end.0.is_empty() {
                        return 0;
                    }
                    (1 + end.0.last().unwrap().value - start.0.last().unwrap().value) as _
                }
            }
        }
    }

    pub unsafe extern "C" fn merged_max_same_key(
        entry1: *const MergedEntry,
        entry2: *const MergedEntry,
    ) -> usize {
        let entry1 = unsafe { &*entry1 };
        let entry2 = unsafe { &*entry2 };
        let mut len = 0;
        for (l1, l2) in entry1.levels.iter().zip(entry2.levels.iter()) {
            if l1.0.as_ref().unwrap_or(&l1.1) == l2.0.as_ref().unwrap_or(&l2.1) {
                len += 1;
            } else {
                break;
            }
        }
        len
    }
}

mod data {
    use super::*;
    use crate::han::*;

    pub type OrderedStrokes = StrRef;

    #[repr(C)]
    pub struct DataFn {
        pub has_han_data: unsafe extern "C" fn(u32) -> bool,
        pub han_data: unsafe extern "C" fn(u32, *mut crate::han::CjkInfo) -> bool,
        pub original_radical: unsafe extern "C" fn(*const crate::han::CjkInfo) -> i64,
        pub kx_radical_from_kind: unsafe extern "C" fn(*const u8, usize) -> i64,
        pub kx_radical_to_index: unsafe extern "C" fn(u8) -> u16,
        pub kx_radical_char: unsafe extern "C" fn(u8) -> u32,
        pub kx_ideography_char: unsafe extern "C" fn(u8) -> u32,
        pub kx_radical_kind: unsafe extern "C" fn(u8, *mut StrRef) -> bool,
        pub mandarin: unsafe extern "C" fn(*const CjkInfo, *mut StrRef, *mut u8) -> bool,
        pub cantonese: unsafe extern "C" fn(*const CjkInfo, *mut StrRef, *mut u8) -> bool,
        pub han_ordered_strokes: unsafe extern "C" fn(u32, *mut OrderedStrokes) -> bool,
        pub han_ordered_strokes_len: unsafe extern "C" fn(*const OrderedStrokes) -> usize,
        pub han_ordered_stroke_n: unsafe extern "C" fn(*const OrderedStrokes, usize) -> u8,
        pub han_ordered_strokes_cmp_with_str:
            unsafe extern "C" fn(*const OrderedStrokes, *const u8, usize) -> i64,

        pub emoji: unsafe extern "C" fn(*const u8, usize) -> *const emojis::Emoji,
        pub emoji_str: unsafe extern "C" fn(*const emojis::Emoji, *mut StrRef) -> bool,
        pub emoji_name: unsafe extern "C" fn(*const emojis::Emoji, *mut StrRef) -> bool,
        pub emoji_group: unsafe extern "C" fn(*const emojis::Emoji) -> u8,
    }

    impl Default for DataFn {
        fn default() -> Self {
            DataFn {
                has_han_data,
                han_data,
                original_radical,
                kx_radical_from_kind,
                kx_radical_to_index,
                kx_radical_char,
                kx_ideography_char,
                kx_radical_kind,
                mandarin,
                cantonese,
                han_ordered_strokes,
                han_ordered_strokes_len,
                han_ordered_stroke_n,
                han_ordered_strokes_cmp_with_str,
                emoji,
                emoji_str,
                emoji_name,
                emoji_group,
            }
        }
    }

    pub unsafe extern "C" fn has_han_data(c: u32) -> bool {
        char::from_u32(c).and_then(cjk_info).is_some()
    }

    pub unsafe extern "C" fn han_data(c: u32, out: *mut CjkInfo) -> bool {
        match char::from_u32(c).and_then(cjk_info) {
            Some(i) => {
                unsafe {
                    out.replace(i);
                }
                true
            }
            None => false,
        }
    }

    pub unsafe extern "C" fn original_radical(info: *const crate::han::CjkInfo) -> i64 {
        unsafe {
            if info.is_null() || (*info).is_invalid() {
                return -1;
            }
            (*info).radical().original() as _
        }
    }

    pub unsafe extern "C" fn kx_radical_from_kind(ptr: *const u8, len: usize) -> i64 {
        unsafe {
            let res = if ptr.is_null() {
                u8::MAX
            } else {
                StrRef { ptr, len }
                    .as_str()
                    .and_then(|s| radical(s).map(|r| r as u8))
                    .unwrap_or(u8::MAX)
            };
            res as _
        }
    }

    pub unsafe extern "C" fn kx_radical_to_index(rad: u8) -> u16 {
        if rad <= MAX_RADICAL.1 {
            let rad: Radical = unsafe { std::mem::transmute(rad) };
            let index = rad.to_index();
            u16::from_be_bytes([index.1, index.0])
        } else {
            u16::MAX
        }
    }

    pub unsafe extern "C" fn kx_radical_char(rad: u8) -> u32 {
        let rad: Radical = if rad < MAX_RADICAL.1 {
            unsafe { std::mem::transmute(rad) }
        } else {
            return u32::MAX;
        };
        kangxi_ideograph(rad)
            .0
            .map(|n| n as u32)
            .unwrap_or(u32::MAX)
    }

    pub unsafe extern "C" fn kx_ideography_char(rad: u8) -> u32 {
        let rad: Radical = if rad < MAX_RADICAL.1 {
            unsafe { std::mem::transmute(rad) }
        } else {
            return u32::MAX;
        };
        kangxi_ideograph(rad).1 as _
    }

    pub unsafe extern "C" fn kx_radical_kind(rad: u8, out: *mut StrRef) -> bool {
        let rad: Radical = if rad < MAX_RADICAL.1 {
            unsafe { std::mem::transmute(rad) }
        } else {
            return false;
        };
        unsafe {
            (&mut *out).with_str(rad.str_repr());
        }
        true
    }

    pub unsafe extern "C" fn mandarin(
        info: *const CjkInfo,
        out_pinyin: *mut StrRef,
        out_tone: *mut u8,
    ) -> bool {
        unsafe {
            if info.is_null() || (*info).is_invalid() {
                return false;
            }
            let info = &*info;
            let Some(man) = info.mandarin() else {
                return false;
            };
            (*out_pinyin).with_str(man.0);
            *out_tone = man.1;
            true
        }
    }

    pub unsafe extern "C" fn cantonese(
        info: *const CjkInfo,
        out_cantonese: *mut StrRef,
        out_tone: *mut u8,
    ) -> bool {
        unsafe {
            if info.is_null() || (*info).is_invalid() {
                return false;
            }
            let info = &*info;
            let Some(man) = info.cantonese() else {
                return false;
            };
            (*out_cantonese).with_str(man.0);
            *out_tone = man.1;
            true
        }
    }

    pub unsafe extern "C" fn han_ordered_strokes(c: u32, out: *mut OrderedStrokes) -> bool {
        unsafe {
            let Some(c) = char::from_u32(c).and_then(ordered_strokes) else {
                return false;
            };
            (*out).ptr = c.0.as_ptr();
            (*out).len = c.0.len();
            true
        }
    }

    pub unsafe extern "C" fn han_ordered_strokes_len(
        ordered_strokes: *const OrderedStrokes,
    ) -> usize {
        unsafe {
            if ordered_strokes.is_null() || (*ordered_strokes).ptr.is_null() {
                return 0;
            }
            let OrderedStrokes { ptr, len } = *ordered_strokes;
            RawStrokes(std::slice::from_raw_parts(ptr, len)).len()
        }
    }

    pub unsafe extern "C" fn han_ordered_stroke_n(
        ordered_strokes: *const OrderedStrokes,
        n: usize,
    ) -> u8 {
        unsafe {
            if ordered_strokes.is_null() || (*ordered_strokes).ptr.is_null() {
                return 0;
            }
            let OrderedStrokes { ptr, len } = *ordered_strokes;
            let stroke = RawStrokes(std::slice::from_raw_parts(ptr, len));
            stroke.get(n).map(|s| s as u8).unwrap_or(0)
        }
    }

    pub unsafe extern "C" fn han_ordered_strokes_cmp_with_str(
        ordered_strokes: *const OrderedStrokes,
        ptr: *const u8,
        len: usize,
    ) -> i64 {
        unsafe {
            if ordered_strokes.is_null() || (*ordered_strokes).ptr.is_null() || ptr.is_null() {
                return -2;
            }
            let Some(s) = StrRef { ptr, len }.as_str() else {
                return -2;
            };
            if s.chars().any(|c| !('1'..='5').contains(&c)) {
                return -2;
            }
            let OrderedStrokes { ptr, len } = *ordered_strokes;
            let Some(o_s) = RawStrokes::from_strokes(std::slice::from_raw_parts(ptr, len)) else {
                return -2;
            };
            for (l, r) in o_s.strokes().zip(s.as_bytes()) {
                match (l as u8).cmp(&(*r - 0x30)) {
                    Ordering::Less => return -1,
                    Ordering::Equal => {}
                    Ordering::Greater => return 1,
                }
            }
            match o_s.len().cmp(&s.len()) {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            }
        }
    }

    pub unsafe extern "C" fn emoji(ptr: *const u8, len: usize) -> *const emojis::Emoji {
        let s = if ptr.is_null() {
            return null();
        } else {
            unsafe { StrRef { ptr, len }.as_str_unchecked() }
        };
        emojis::get(s).map_or(null(), |emj: &'static emojis::Emoji| emj as _)
    }

    pub unsafe extern "C" fn emoji_str(emj: *const emojis::Emoji, out: *mut StrRef) -> bool {
        if emj.is_null() {
            return false;
        }
        unsafe {
            let name: &'static str = (&*emj).as_str();
            (*out).with_str(name);
        }
        true
    }

    pub unsafe extern "C" fn emoji_name(emj: *const emojis::Emoji, out: *mut StrRef) -> bool {
        if emj.is_null() {
            return false;
        }
        unsafe {
            let name: &'static str = (&*emj).name();
            (*out).with_str(name);
        }
        true
    }

    pub unsafe extern "C" fn emoji_group(emj: *const emojis::Emoji) -> u8 {
        if emj.is_null() {
            return u8::MAX;
        }
        unsafe { (*emj).group() as _ }
    }
}

mod pcre2 {
    use super::*;
    use pcre2_sys::*;

    #[repr(C)]
    pub struct Pcre2Fn {
        pub pcre2_compile_8: unsafe extern "C" fn(
            *const u8,
            usize,
            u32,
            *mut c_int,
            *mut usize,
            *mut pcre2_compile_context_8,
        ) -> *mut pcre2_code_8,
        pub pcre2_jit_compile_8: unsafe extern "C" fn(*mut pcre2_code_8, u32) -> c_int,
        pub pcre2_match_data_create_from_pattern_8: unsafe extern "C" fn(
            *const pcre2_code_8,
            *mut pcre2_general_context_8,
        )
            -> *mut pcre2_match_data_8,
        pub pcre2_match_8: unsafe extern "C" fn(
            *const pcre2_code_8,
            *const u8,
            usize,
            usize,
            u32,
            *mut pcre2_match_data_8,
            *mut pcre2_match_context_8,
        ) -> c_int,
        pub pcre2_substitute_8: unsafe extern "C" fn(
            *const pcre2_code_8,
            *const u8,
            usize,
            usize,
            u32,
            *mut pcre2_match_data_8,
            *mut pcre2_match_context_8,
            *const u8,
            usize,
            *mut u8,
            *mut usize,
        ) -> c_int,
        pub pcre2_pattern_info_8:
            unsafe extern "C" fn(*const pcre2_code_8, u32, *mut c_void) -> c_int,
        pub pcre2_get_ovector_pointer_8:
            unsafe extern "C" fn(*mut pcre2_match_data_8) -> *mut usize,
        pub pcre2_code_free_8: unsafe extern "C" fn(*mut pcre2_code_8),
        pub pcre2_match_data_free_8: unsafe extern "C" fn(*mut pcre2_match_data_8),
        pub pcre2_get_error_message_8: unsafe extern "C" fn(c_int, *mut u8, usize) -> c_int,
    }

    impl Default for Pcre2Fn {
        fn default() -> Self {
            Pcre2Fn {
                pcre2_compile_8,
                pcre2_jit_compile_8,
                pcre2_match_data_create_from_pattern_8,
                pcre2_match_8,
                pcre2_substitute_8,
                pcre2_pattern_info_8,
                pcre2_get_ovector_pointer_8,
                pcre2_code_free_8,
                pcre2_match_data_free_8,
                pcre2_get_error_message_8,
            }
        }
    }
}
