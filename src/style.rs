use std::{borrow::Cow, marker::PhantomData, path::Path, str::FromStr};

use anyhow::{Context, Result, anyhow, bail};
use compact_str::CompactString;
use memchr::memchr3;
use smallvec::SmallVec;

use crate::page::IndexPage;

#[derive(Debug, Clone, PartialEq)]
pub struct IndexEntry {
    pub levels: SmallVec<[(Option<String>, String); 1]>,
    pub range: RangeKind,
    pub page_commands: Option<CompactString>,
    pub pages: SmallVec<[IndexPage; 1]>,
    pub pages_raw: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeKind {
    Open,
    Close,
    None,
}

impl IndexEntry {
    pub fn is_same(&self, other: &Self) -> bool {
        if self.levels.len() == other.levels.len() {
            for (l, r) in self.levels.iter().zip(other.levels.iter()) {
                match &(l, r) {
                    ((None, dl), (None, dr)) => {
                        if dl != dr {
                            return false;
                        }
                    }
                    ((Some(kl), dl), (None, dr)) => {
                        if kl != dr || dl != dr {
                            return false;
                        }
                    }
                    ((None, dl), (Some(kr), dr)) => {
                        if dl != kr || dl != dr {
                            return false;
                        }
                    }
                    ((Some(kl), dl), (Some(kr), dr)) => {
                        if (kl != kr) || (dl != dr) {
                            return false;
                        }
                    }
                }
            }
            !(self.pages.is_empty() != self.pages.is_empty())
        } else {
            false
        }
    }

    pub fn parse_index_line(ist: &IstInputStyle, line: &str) -> Result<Option<Self>> {
        #[derive(Debug, Clone, Copy, PartialEq)]
        enum Kind {
            Value,
            Special,
            Page,
        }

        if line.trim_ascii().is_empty() || line.starts_with(ist.comment) {
            return Ok(None);
        }

        let mut input = Input {
            inner: line,
            cursor: 0,
        };
        if !input.skip_str(&ist.keyword) {
            bail!("invalid index line: {}", line);
        }

        if !input.skip_char(ist.arg_open) {
            bail!("invalid index line: {}", line);
        }
        if input.is_empty() {
            bail!("invalid index line: {}", line);
        }

        let mut levels = SmallVec::default();
        let mut range = RangeKind::None;
        let mut page_commands: Option<CompactString> = None;
        let mut pages = SmallVec::default();

        let mut state = Kind::Value;
        let mut entry = String::with_capacity(input.len());
        let mut key = String::new();
        let mut arg_level = 1i32;
        let mut page_str = CompactString::new("");
        let mut pages_raw = String::new();

        loop {
            if state == Kind::Page && input.skip_str(&ist.page_compositor) {
                let page = IndexPage::from_str(&page_str)?;
                pages_raw.push_str(&page_str);
                pages_raw.push_str(&ist.page_compositor);
                pages.push(page);
                page_str.clear();
                continue;
            }

            let curr_chr = input.get()?;

            if curr_chr == ist.quote {
                let c = input.get()?;
                match state {
                    Kind::Value => entry.push(c),
                    Kind::Special => page_commands.get_or_insert_default().push(c),
                    Kind::Page => page_str.push(c),
                }
            } else if curr_chr == ist.escape {
                let next_chr = input
                    .peek()
                    .with_context(|| anyhow!("invalid index line: {}", line))?;
                if next_chr == ist.quote {
                    input.next().unwrap();
                    match state {
                        Kind::Value => {
                            entry.push(curr_chr);
                            entry.push(next_chr);
                        }
                        Kind::Special => {
                            page_commands.get_or_insert_default().push(curr_chr);
                            page_commands.get_or_insert_default().push(next_chr);
                        }
                        Kind::Page => {
                            page_str.push(curr_chr);
                            page_str.push(next_chr);
                        }
                    }
                } else if next_chr == ist.arg_open || next_chr == ist.arg_close {
                    bail!("invalid index line: {}", line);
                } else {
                    match state {
                        Kind::Value => entry.push(curr_chr),
                        Kind::Special => page_commands.get_or_insert_default().push(curr_chr),
                        Kind::Page => page_str.push(curr_chr),
                    }
                }
            } else if state == Kind::Value && curr_chr == ist.actual {
                if key.is_empty() {
                    if entry.is_empty() {
                        log::warn!(
                            "missing key while actual char '{}' is present in index line: {}",
                            ist.actual,
                            line
                        );
                    }
                    key.push_str(&entry);
                    entry.clear();
                } else {
                    log::warn!(
                        "un-escaped actual char '{}' in index line: {}",
                        curr_chr,
                        line
                    );
                    entry.push(curr_chr);
                }
            } else if state == Kind::Value && curr_chr == ist.level {
                if key.is_empty() {
                    levels.push((None, entry.clone()));
                } else {
                    levels.push((Some(key.clone()), entry.clone()));
                }
                key.clear();
                entry.clear();
            } else if state == Kind::Value && curr_chr == ist.encap {
                if arg_level == 1 {
                    if key.is_empty() {
                        levels.push((None, entry.clone()));
                    } else {
                        levels.push((Some(key.clone()), entry.clone()));
                    }
                    key.clear();
                    entry.clear();
                    state = Kind::Special;
                } else {
                    log::warn!(
                        "un-escaped encap char '{}' in index line: {}",
                        ist.encap,
                        line
                    );
                    entry.push(curr_chr);
                }
            } else if state == Kind::Special && curr_chr == ist.range_open {
                range = RangeKind::Open;
            } else if state == Kind::Special && curr_chr == ist.range_close {
                range = RangeKind::Close;
            } else if curr_chr == ist.arg_open {
                match state {
                    Kind::Value => entry.push(curr_chr),
                    Kind::Special => page_commands.get_or_insert_default().push(curr_chr),
                    Kind::Page => page_str.push(curr_chr),
                }
                arg_level += 1;
            } else if curr_chr == ist.arg_close {
                arg_level -= 1;
                match state {
                    Kind::Value if arg_level == 0 => {
                        state = Kind::Page;
                        if !input.skip_char(ist.arg_open) {
                            bail!("invalid index line: {}", line);
                        }
                        arg_level += 1;
                    }
                    Kind::Special if arg_level == 0 => {
                        state = Kind::Page;
                        if !input.skip_char(ist.arg_open) {
                            bail!("invalid index line: {}", line);
                        }
                        arg_level += 1;
                    }
                    Kind::Page if arg_level == 0 => {
                        if input.peek().is_some() {
                            log::warn!(
                                "index line contains extra characters: {}",
                                input.rest().unwrap_or_default()
                            );
                        }

                        if page_commands.is_none() {
                            if key.is_empty() {
                                levels.push((None, entry.clone()));
                            } else {
                                levels.push((Some(key.clone()), entry.clone()));
                            }
                            key.clear();
                            entry.clear();
                        }

                        let page = IndexPage::from_str(&page_str)?;
                        pages_raw.push_str(&page_str);
                        pages.push(page);
                        page_str.clear();

                        break;
                    }
                    Kind::Value => entry.push(curr_chr),
                    Kind::Special => page_commands.get_or_insert_default().push(curr_chr),
                    Kind::Page => page_str.push(curr_chr),
                }
            } else {
                match state {
                    Kind::Value => entry.push(curr_chr),
                    Kind::Special => page_commands.get_or_insert_default().push(curr_chr),
                    Kind::Page => page_str.push(curr_chr),
                }
            }
        }
        if !entry.is_empty()
            || !key.is_empty()
            || levels.is_empty()
            || arg_level != 0
            || !input.is_empty()
        {
            bail!("invalid index line: {}", line);
        }

        //TODO: remove empty levels or not?
        // let mut new_levels = SmallVec::<[(Option<String>, String); 1]>::new();
        // let levels: smallvec::IntoIter<[_; 1]> = levels.into_iter();
        // for level in levels {
        //     if level.1.is_empty() {
        //         log::warn!("empty level in index line: {}", line);
        //     } else {
        //         new_levels.push(level);
        //     }
        // }
        let new_levels = levels;

        Ok(Some(IndexEntry {
            levels: new_levels,
            range,
            page_commands,
            pages,
            pages_raw,
        }))
    }

    pub fn from_ikv<T: AsRef<str>>(
        ist: &IstInputStyle,
        lines: &mut impl Iterator<Item = T>,
    ) -> Result<Option<Self>> {
        crate::ikv::from_ikv(lines, ist)
    }
}

struct Input<'i> {
    inner: &'i str,
    cursor: usize,
}

impl Input<'_> {
    #[inline]
    fn is_empty(&mut self) -> bool {
        self.cursor >= self.inner.len()
    }

    #[inline]
    fn len(&self) -> usize {
        self.inner.len() - self.cursor
    }

    fn get(&mut self) -> Result<char> {
        self.next()
            .with_context(|| anyhow!("invalid index line: {}", self.inner))
    }

    fn next(&mut self) -> Option<char> {
        if self.is_empty() {
            None
        } else {
            let chr = unsafe {
                self.inner
                    .get_unchecked(self.cursor..)
                    .chars()
                    .next()
                    .unwrap_unchecked()
            };
            self.cursor += chr.len_utf8();
            Some(chr)
        }
    }

    fn peek(&mut self) -> Option<char> {
        if self.is_empty() {
            None
        } else {
            let chr = unsafe {
                self.inner
                    .get_unchecked(self.cursor..)
                    .chars()
                    .next()
                    .unwrap_unchecked()
            };
            Some(chr)
        }
    }

    fn skip_char(&mut self, c: char) -> bool {
        if self.is_empty() {
            false
        } else {
            let rest = unsafe { self.inner.get_unchecked(self.cursor..) };
            if rest.starts_with(c) {
                self.cursor += c.len_utf8();
                true
            } else {
                false
            }
        }
    }

    fn skip_str(&mut self, s: &str) -> bool {
        if self.is_empty() {
            false
        } else {
            let rest = unsafe { self.inner.get_unchecked(self.cursor..) };
            if rest.starts_with(s) {
                self.cursor += s.len();
                true
            } else {
                false
            }
        }
    }

    fn rest(&mut self) -> Option<&str> {
        self.inner.get(self.cursor..)
    }
}

#[derive(Debug, Clone)]
pub struct IstInputStyle {
    /// index command. Default `"\\indexentry"`.
    pub keyword: String,
    /// argument opening delimiter. Default: `'{'`.
    pub arg_open: char,
    /// argument closing delimiter. Default: `'}'`.
    pub arg_close: char,
    /// page range opening delimiter. Default: `'('`.
    pub range_open: char,
    /// page range closing delimiter. Default: `')'`.
    pub range_close: char,
    /// index level delimiter. Default: `'!'`.
    pub level: char,
    /// actual key designator. Default: `'@'`.
    pub actual: char,
    /// page number encapsulator. Default: `'|'`.
    pub encap: char,
    /// quote symbol. Default: `'"'`.
    pub quote: char,
    /// symbol that escapes `quote`. Default: `'\\'`.
    pub escape: char,
    /// composite page delimiter. Default: `"-"`.
    pub page_compositor: String,
    /// comment line designator. Default: `'%'`.
    pub comment: char,
    /// separator of kv format. Default: `","`.
    pub separator: String,
}

#[derive(Debug, Clone)]
pub struct IstOutputStyle {
    /// index preamble. Default: `"\\begin{theindex}\n"`.
    pub preamble: String,
    /// index postamble. Default: `"\n\n\\end{theindex}\n"`.
    pub postamble: String,
    /// intergroup vertical space. Default: `"\n\n  \\indexspace\n"`.
    pub group_skip: String,
    /// new letter heading prefix. Default: `""`.
    pub heading_prefix: String,
    /// new letter heading sufix. Default: `""`.
    pub heading_suffix: String,
    /// flag designating new letter. Default: `0`.
    pub headings_flag: i32,
    /// heading for numbers (flag > 0). Default: `"Numbers"`.
    pub numhead_positive: String,
    /// heading for numbers (flag < 0). Default: `"numbers"`.
    pub numhead_negative: String,
    /// heading for symbols (flag > 0). Default: `"Symbols`.
    pub symhead_positive: String,
    /// heading for symbols (flag < 0). Default: `"symbols`.
    pub symhead_negative: String,
    /// level 0 item separator. Default: `"\n  \\item "`.
    pub item_0: String,
    /// level 1 item separator. Default: `"\n    \\subitem "`.
    pub item_1: String,
    /// level 2 item separator. Default: `"\n      \\subsubitem "`.
    pub item_2: String,
    /// levels 0/1 separator. Default: `"\n    \\subitem "`.
    pub item_01: String,
    /// levels 0/1 separator, no page number at level 0. Default: `"\n    \\subitem "`.
    pub item_x1: String,
    /// levels 1/2 separator. Default: `"\n      \\subsubitem "`.
    pub item_12: String,
    /// levels x/2 separator, no page number at level 0. Default: `"\n      \\subsubitem "`.
    pub item_x2: String,
    /// level 0 key/page delimiter. Default: `", "`.
    pub delim_0: String,
    /// level 1 key/page delimiter. Default: `", "`.
    pub delim_1: String,
    /// level 2 key/page delimiter. Default: `", "`.
    pub delim_2: String,
    /// inter page number delimiter. Default: `", "`.
    pub delim_n: String,
    /// page range designator. Default: `"--"`.
    pub delim_r: String,
    /// page list terminator. Default: `""`.
    pub delim_t: String,
    /// page encapsulator prefix. Default: `"\\"`.
    pub encap_prefix: String,
    /// page encapsulator infix. Default: `"{"`.
    pub encap_infix: String,
    /// page encapsulator suffix. Default: `"}"`.
    pub encap_suffix: String,
    /// page type precedence. Default: `"rnaRA"`.
    pub page_precedence: String,
    // delimiter to replace the range delimiter and the second page number of a two page list.
    // When present, it overrides `delim_r`. Default `""`.
    pub suffix_2p: String,
    // delimiter to replace the range delimiter and the second page number of a three page list.
    // When present, it overrides `delim_r` and `suffix_mp`. Default `""`.
    pub suffix_3p: String,
    // Delimiter to replace the range delimiter and the second page number of a multiple page list (three or more pages).
    // When present, it overrides `delim_r`. Default `""`.
    pub suffix_mp: String,
    /// 笔画数前缀. Default: `""`.
    pub stroke_prefix: String,
    /// 笔画数后缀. Default: `" 画"`.
    pub stroke_suffix: String,
    /// 部首前缀. Default: `""`.
    pub radical_prefix: String,
    /// 部首后缀. Default: `"部"`.
    pub radical_suffix: String,
    /// 是否输出简化部首. Default: `1`.
    pub radical_simplified_flag: i32,
    /// 简化部首前缀. Default: `"（"`.
    pub radical_simplified_prefix: String,
    /// 简化部首的分隔符. Default: `"、"`.
    pub radical_simplified_delimiter: String,
    /// 简化部首后缀. Default: `"）"`.
    pub radical_simplified_suffix: String,
}

#[derive(Debug, Clone)]
pub struct IstFile {
    /// index command. Default `"\\indexentry"`.
    pub keyword: String,
    /// argument opening delimiter. Default: `'{'`.
    pub arg_open: char,
    /// argument closing delimiter. Default: `'}'`.
    pub arg_close: char,
    /// page range opening delimiter. Default: `'('`.
    pub range_open: char,
    /// page range closing delimiter. Default: `')'`.
    pub range_close: char,
    /// index level delimiter. Default: `'!'`.
    pub level: char,
    /// actual key designator. Default: `'@'`.
    pub actual: char,
    /// page number encapsulator. Default: `'|'`.
    pub encap: char,
    /// quote symbol. Default: `'"'`.
    pub quote: char,
    /// symbol that escapes `quote`. Default: `'\\'`.
    pub escape: char,
    /// composite page delimiter. Default: `"-"`.
    pub page_compositor: String,

    /// separator of kv format. Default: `","`.
    pub separator: String,

    /// index preamble. Default: `"\\begin{theindex}\n"`.
    pub preamble: String,
    /// index postamble. Default: `"\n\n\\end{theindex}\n"`.
    pub postamble: String,
    /// intergroup vertical space. Default: `"\n\n  \\indexspace\n"`.
    pub group_skip: String,
    /// new letter heading prefix. Default: `""`.
    pub heading_prefix: String,
    /// new letter heading sufix. Default: `""`.
    pub heading_suffix: String,
    /// flag designating new letter. Default: `0`.
    pub headings_flag: i32,
    /// heading for numbers (flag > 0). Default: `"Numbers"`.
    pub numhead_positive: String,
    /// heading for numbers (flag < 0). Default: `"numbers"`.
    pub numhead_negative: String,
    /// heading for symbols (flag > 0). Default: `"Symbols`.
    pub symhead_positive: String,
    /// heading for symbols (flag < 0). Default: `"symbols`.
    pub symhead_negative: String,
    /// level 0 item separator. Default: `"\n  \\item "`.
    pub item_0: String,
    /// level 1 item separator. Default: `"\n    \\subitem "`.
    pub item_1: String,
    /// level 2 item separator. Default: `"\n      \\subsubitem "`.
    pub item_2: String,
    /// levels 0/1 separator. Default: `"\n    \\subitem "`.
    pub item_01: String,
    /// levels 0/1 separator, no page number at level 0. Default: `"\n    \\subitem "`.
    pub item_x1: String,
    /// levels 1/2 separator. Default: `"\n      \\subsubitem "`.
    pub item_12: String,
    /// levels x/2 separator, no page number at level 0. Default: `"\n      \\subsubitem "`.
    pub item_x2: String,
    /// level 0 key/page delimiter. Default: `", "`.
    pub delim_0: String,
    /// level 1 key/page delimiter. Default: `", "`.
    pub delim_1: String,
    /// level 2 key/page delimiter. Default: `", "`.
    pub delim_2: String,
    /// inter page number delimiter. Default: `", "`.
    pub delim_n: String,
    /// page range designator. Default: `"--"`.
    pub delim_r: String,
    /// page list terminator. Default: `""`.
    pub delim_t: String,
    /// page encapsulator prefix. Default: `"\\"`.
    pub encap_prefix: String,
    /// page encapsulator infix. Default: `"{"`.
    pub encap_infix: String,
    /// page encapsulator suffix. Default: `"}"`.
    pub encap_suffix: String,
    /// page type precedence. Default: `"rnaRA"`.
    pub page_precedence: String,
    // delimiter to replace the range delimiter and the second page number of a two page list.
    // When present, it overrides `delim_r`. Default `""`.
    pub suffix_2p: String,
    // delimiter to replace the range delimiter and the second page number of a three page list.
    // When present, it overrides `delim_r` and `suffix_mp`. Default `""`.
    pub suffix_3p: String,
    // Delimiter to replace the range delimiter and the second page number of a multiple page list (three or more pages).
    // When present, it overrides `delim_r`. Default `""`.
    pub suffix_mp: String,
    /// comment line designator. Default: `'%'`.
    pub comment: char,
    /// 笔画数前缀. Default: `""`.
    pub stroke_prefix: String,
    /// 笔画数后缀. Default: `" 画"`.
    pub stroke_suffix: String,
    /// 部首前缀. Default: `""`.
    pub radical_prefix: String,
    /// 部首后缀. Default: `"部"`.
    pub radical_suffix: String,
    /// 是否输出简化部首. Default: `1`.
    pub radical_simplified_flag: i32,
    /// 简化部首前缀. Default: `"（"`.
    pub radical_simplified_prefix: String,
    /// 简化部首的分隔符. Default: `"、"`.
    pub radical_simplified_delimiter: String,
    /// 简化部首后缀. Default: `"）"`.
    pub radical_simplified_suffix: String,
    /// 下面是在 cindex 不产生功能的选项。
    pub setpage_prefix: PhantomData<String>,
    pub setpage_suffix: PhantomData<String>,
    pub line_max: PhantomData<i32>,
    pub indent_space: PhantomData<String>,
    pub indent_length: PhantomData<i32>,
}

impl Default for IstFile {
    fn default() -> Self {
        IstFile {
            keyword: "\\indexentry".into(),
            arg_open: '{',
            arg_close: '}',
            range_open: '(',
            range_close: ')',
            level: '!',
            actual: '@',
            encap: '|',
            quote: '\"',
            escape: '\\',
            page_compositor: "-".into(),
            separator: ",".into(),
            preamble: "\\begin{theindex}\n".into(),
            postamble: "\n\n\\end{theindex}\n".into(),
            group_skip: "\n\n  \\indexspace\n".into(),
            heading_prefix: "".into(),
            heading_suffix: "".into(),
            headings_flag: 0,
            numhead_positive: "Numbers".into(),
            numhead_negative: "numbers".into(),
            symhead_positive: "Symbols".into(),
            symhead_negative: "symbols".into(),
            item_0: "\n  \\item ".into(),
            item_1: "\n    \\subitem ".into(),
            item_2: "\n      \\subsubitem ".into(),
            item_01: "\n    \\subitem ".into(),
            item_x1: "\n    \\subitem ".into(),
            item_12: "\n      \\subsubitem ".into(),
            item_x2: "\n      \\subsubitem ".into(),
            delim_0: ", ".into(),
            delim_1: ", ".into(),
            delim_2: ", ".into(),
            delim_n: ", ".into(),
            delim_r: "--".into(),
            delim_t: "".into(),
            encap_prefix: "\\".into(),
            encap_infix: "{".into(),
            encap_suffix: "}".into(),
            page_precedence: "rnaRA".into(),
            suffix_2p: "".into(),
            suffix_3p: "".into(),
            suffix_mp: "".into(),
            comment: '%',
            stroke_prefix: "".into(),
            stroke_suffix: " 画".into(),
            radical_prefix: "".into(),
            radical_suffix: "部".into(),
            radical_simplified_flag: 1,
            radical_simplified_prefix: "（".into(),
            radical_simplified_delimiter: "、".into(),
            radical_simplified_suffix: "）".into(),
            setpage_prefix: Default::default(),
            setpage_suffix: Default::default(),
            line_max: Default::default(),
            indent_space: Default::default(),
            indent_length: Default::default(),
        }
    }
}

impl IstFile {
    pub fn read(path: impl AsRef<Path>) -> Result<IstFile> {
        let path = path.as_ref();
        let mut ist_file = IstFile::default();
        parse_ist_string(&mut ist_file, &std::fs::read_to_string(path)?)?;
        Ok(ist_file)
    }

    pub fn read_string(ist_content: &str) -> Result<IstFile> {
        let mut ist_file = IstFile::default();
        parse_ist_string(&mut ist_file, ist_content)?;
        Ok(ist_file)
    }

    pub fn append(&mut self, ist_content: &str) -> Result<()> {
        parse_ist_string(self, ist_content)
    }

    pub fn split(self) -> (IstInputStyle, IstOutputStyle) {
        (
            IstInputStyle {
                keyword: self.keyword,
                arg_open: self.arg_open,
                arg_close: self.arg_close,
                range_open: self.range_open,
                range_close: self.range_close,
                level: self.level,
                actual: self.actual,
                encap: self.encap,
                quote: self.quote,
                escape: self.escape,
                page_compositor: self.page_compositor,
                comment: self.comment,
                separator: self.separator,
            },
            IstOutputStyle {
                preamble: self.preamble,
                postamble: self.postamble,
                group_skip: self.group_skip,
                heading_prefix: self.heading_prefix,
                heading_suffix: self.heading_suffix,
                headings_flag: self.headings_flag,
                numhead_positive: self.numhead_positive,
                numhead_negative: self.numhead_negative,
                symhead_positive: self.symhead_positive,
                symhead_negative: self.symhead_negative,
                item_0: self.item_0,
                item_1: self.item_1,
                item_2: self.item_2,
                item_01: self.item_01,
                item_x1: self.item_x1,
                item_12: self.item_12,
                item_x2: self.item_x2,
                delim_0: self.delim_0,
                delim_1: self.delim_1,
                delim_2: self.delim_2,
                delim_n: self.delim_n,
                delim_r: self.delim_r,
                delim_t: self.delim_t,
                encap_prefix: self.encap_prefix,
                encap_infix: self.encap_infix,
                encap_suffix: self.encap_suffix,
                page_precedence: self.page_precedence,
                suffix_2p: self.suffix_2p,
                suffix_3p: self.suffix_3p,
                suffix_mp: self.suffix_mp,
                stroke_prefix: self.stroke_prefix,
                stroke_suffix: self.stroke_suffix,
                radical_prefix: self.radical_prefix,
                radical_suffix: self.radical_suffix,
                radical_simplified_flag: self.radical_simplified_flag,
                radical_simplified_prefix: self.radical_simplified_prefix,
                radical_simplified_delimiter: self.radical_simplified_delimiter,
                radical_simplified_suffix: self.radical_simplified_suffix,
            },
        )
    }
}

fn parse_ist_string(ist: &mut IstFile, ist_content: &str) -> Result<()> {
    let input = &mut ist_content.trim_ascii_start();

    while !input.is_empty() {
        let len = memchr3(b'\n', b'\t', b' ', input.as_bytes()).unwrap_or(input.len());
        let key = input[..len].trim_ascii_end();
        if key.starts_with('%') {
            if input.as_bytes()[len] != b'\n' {
                match memchr::memchr(b'\n', input[len..].as_bytes()) {
                    Some(new_len) => *input = input[len + new_len..].trim_ascii_start(),
                    None => *input = "",
                }
            } else {
                *input = input[len + 1..].trim_ascii_start();
            }
            continue;
        }

        *input = input[len + 1..].trim_ascii_start();
        match key {
            "keyword" => ist.keyword = scan_string(input)?.to_string(),
            "arg_open" => ist.arg_open = scan_char(input)?,
            "arg_close" => ist.arg_close = scan_char(input)?,
            "range_open" => ist.range_open = scan_char(input)?,
            "range_close" => ist.range_close = scan_char(input)?,
            "level" => ist.level = scan_char(input)?,
            "actual" => ist.actual = scan_char(input)?,
            "encap" => ist.encap = scan_char(input)?,
            "quote" => ist.quote = scan_char(input)?,
            "escape" => ist.escape = scan_char(input)?,
            "page_compositor" => ist.page_compositor = scan_string(input)?.to_string(),
            "separator" => ist.separator = scan_string(input)?.to_string(),
            "preamble" => ist.preamble = scan_string(input)?.to_string(),
            "postamble" => ist.postamble = scan_string(input)?.to_string(),
            "group_skip" => ist.group_skip = scan_string(input)?.to_string(),
            "heading_prefix" => ist.heading_prefix = scan_string(input)?.to_string(),
            "heading_suffix" => ist.heading_suffix = scan_string(input)?.to_string(),
            "headings_flag" => ist.headings_flag = scan_number(input)? as _,
            "numhead_positive" => ist.numhead_positive = scan_string(input)?.to_string(),
            "numhead_negative" => ist.numhead_negative = scan_string(input)?.to_string(),
            "symhead_positive" => ist.symhead_positive = scan_string(input)?.to_string(),
            "symhead_negative" => ist.symhead_negative = scan_string(input)?.to_string(),
            "item_0" => ist.item_0 = scan_string(input)?.to_string(),
            "item_1" => ist.item_1 = scan_string(input)?.to_string(),
            "item_2" => ist.item_2 = scan_string(input)?.to_string(),
            "item_01" => ist.item_01 = scan_string(input)?.to_string(),
            "item_x1" => ist.item_x1 = scan_string(input)?.to_string(),
            "item_12" => ist.item_12 = scan_string(input)?.to_string(),
            "item_x2" => ist.item_x2 = scan_string(input)?.to_string(),
            "delim_0" => ist.delim_0 = scan_string(input)?.to_string(),
            "delim_1" => ist.delim_1 = scan_string(input)?.to_string(),
            "delim_2" => ist.delim_2 = scan_string(input)?.to_string(),
            "delim_n" => ist.delim_n = scan_string(input)?.to_string(),
            "delim_r" => ist.delim_r = scan_string(input)?.to_string(),
            "delim_t" => ist.delim_t = scan_string(input)?.to_string(),
            "encap_prefix" => ist.encap_prefix = scan_string(input)?.to_string(),
            "encap_infix" => ist.encap_infix = scan_string(input)?.to_string(),
            "encap_suffix" => ist.encap_suffix = scan_string(input)?.to_string(),
            "page_precedence" => ist.page_precedence = scan_string(input)?.to_string(),
            "suffix_2p" => ist.suffix_2p = scan_string(input)?.to_string(),
            "suffix_3p" => ist.suffix_3p = scan_string(input)?.to_string(),
            "suffix_mp" => ist.suffix_mp = scan_string(input)?.to_string(),
            "comment" => ist.comment = scan_char(input)?,
            "stroke_prefix" => ist.stroke_prefix = scan_string(input)?.to_string(),
            "stroke_suffix" => ist.stroke_suffix = scan_string(input)?.to_string(),
            "radical_prefix" => ist.radical_prefix = scan_string(input)?.to_string(),
            "radical_suffix" => ist.radical_suffix = scan_string(input)?.to_string(),
            "radical_simplified_flag" => ist.radical_simplified_flag = scan_number(input)? as _,
            "radical_simplified_prefix" => {
                ist.radical_simplified_prefix = scan_string(input)?.to_string()
            }
            "radical_simplified_delimiter" => {
                ist.radical_simplified_delimiter = scan_string(input)?.to_string()
            }
            "radical_simplified_suffix" => {
                ist.radical_simplified_suffix = scan_string(input)?.to_string()
            }
            "setpage_prefix" => _ = scan_string(input)?.to_string(),
            "setpage_suffix" => _ = scan_string(input)?.to_string(),
            "line_max" => _ = scan_number(input)?,
            "indent_space" => _ = scan_string(input)?.to_string(),
            "indent_length" => _ = scan_number(input)?,
            _ => {
                log::warn!("unknown key: '{}'", key);
                if input.is_empty() {
                    break;
                }
                match input.as_bytes()[0] {
                    b'"' | b'`' | b'r' => {
                        scan_string(input)?.to_string();
                    }
                    b'\'' => {
                        scan_char(input)?;
                    }
                    b'+' | b'-' => {
                        scan_number(input)?;
                    }
                    b => {
                        if b.is_ascii_digit() {
                            scan_number(input)?;
                        } else {
                            bail!("invalid value for \"{}\"", key);
                        }
                    }
                }
            }
        }
        let new_input = input.trim_ascii_start();
        if input.len() > new_input.len() {
            *input = new_input;
        } else {
            bail!("ist key should be separated by space(s)");
        }
    }
    Ok(())
}

fn scan_escaped_char(input: &mut &str) -> Result<char> {
    let raw_input = *input;
    if input.is_empty() {
        bail!("invalid escaping sequence: \\{}", raw_input);
    }
    if input.as_bytes()[0] == b'x' {
        if input.len() < 2 {
            bail!("incomplete escaping sequence: \\{}", raw_input);
        }
        let len = input.as_bytes()[1..]
            .iter()
            .position(|v| !v.is_ascii_hexdigit())
            .unwrap_or(input.len() - 1);
        let num = &input[1..=len];
        *input = &input[1 + len..];
        let n = u32::from_str_radix(num, 16)?;
        Ok(char::from_u32(n).context(format!("invalid escaped character: \\{}", raw_input))?)
    } else if input.as_bytes()[0] == b'u' {
        if input.len() < 4 {
            bail!("incomplete escaping sequence: \\{}", raw_input);
        }
        let num = if input.as_bytes()[1] == b'{' {
            let idx = memchr::memchr(b'}', input.as_bytes())
                .context(format!("incomplete escaping sequence: \\{}", raw_input))?;
            let num = &input[1..idx];
            *input = &input[idx + 1..];
            num
        } else {
            if input.len() < 5 {
                bail!("incomplete escaping sequence: \\{}", raw_input);
            }
            let num = &input[1..=4];
            *input = &input[5..];
            num
        };
        let n = u32::from_str_radix(num, 16)?;
        Ok(char::from_u32(n).context(format!("invalid escaped character: \\{}", raw_input))?)
    } else {
        bail!("invalid escaped character: \\{}", raw_input);
    }
}

fn scan_number(input: &mut &str) -> Result<i64> {
    let s = *input;
    if s.is_empty() {
        bail!("empty");
    }

    let bytes = s.as_bytes();
    let mut i = 0;

    // sign
    let neg = match bytes[0] {
        b'+' => {
            i += 1;
            false
        }
        b'-' => {
            i += 1;
            true
        }
        _ => false,
    };

    if i == bytes.len() {
        bail!("expected digits");
    }

    // radix
    let radix = if bytes.len() >= i + 2 && bytes[i] == b'0' {
        match bytes[i + 1] {
            b'x' | b'X' => {
                i += 2;
                16
            }
            b'b' | b'B' => {
                i += 2;
                2
            }
            _ => 10,
        }
    } else {
        10
    };

    let mut value: i64 = 0;
    let mut prev_sep = false;
    let mut has_digit = false;

    while i < bytes.len() {
        let b = bytes[i];

        if b == b'_' {
            if !has_digit || prev_sep {
                bail!("invalid '_' in number");
            }
            prev_sep = true;
            i += 1;
            continue;
        }

        let digit = match radix {
            2 => match b {
                b'0'..=b'1' => (b - b'0') as i64,
                _ => break,
            },
            10 => match b {
                b'0'..=b'9' => (b - b'0') as i64,
                _ => break,
            },
            16 => match b {
                b'0'..=b'9' => (b - b'0') as i64,
                b'a'..=b'f' => (b - b'a' + 10) as i64,
                b'A'..=b'F' => (b - b'A' + 10) as i64,
                _ => break,
            },
            _ => unreachable!(),
        };

        has_digit = true;
        prev_sep = false;

        value = value
            .checked_mul(radix as i64)
            .ok_or_else(|| anyhow!("integer overflow"))?;
        value = value
            .checked_add(digit)
            .ok_or_else(|| anyhow!("integer overflow"))?;

        i += 1;
    }

    if !has_digit {
        bail!("expected digits");
    }
    if prev_sep {
        bail!("number cannot end with '_'");
    }
    if neg {
        value = value
            .checked_neg()
            .ok_or_else(|| anyhow!("integer overflow"))?;
    }

    *input = &s[i..];
    Ok(value)
}

fn scan_char(input: &mut &str) -> Result<char> {
    let raw_input = *input;
    if input.is_empty() {
        bail!("empty");
    }

    *input = &input[1..];
    if input == &"'" {
        bail!("invalid character: {}", raw_input);
    }

    let c = match input.as_bytes()[0] {
        b'\\' => {
            *input = &input[1..];
            if input.len() < 2 {
                bail!("unclosed character: {}", raw_input);
            }
            let (c, len) = match input.chars().next().unwrap() {
                'a' => ('\u{07}', 1), // BEL
                'b' => ('\u{08}', 1), // BS
                'f' => ('\u{0C}', 1),
                'n' => ('\u{0A}', 1),
                'r' => ('\u{0D}', 1),
                't' => ('\u{09}', 1),
                'v' => ('\u{0B}', 1),
                'u' | 'x' => (scan_escaped_char(input)?, 0),
                '0' => ('\0', 1),
                c if c.is_ascii_alphanumeric() => {
                    bail!(
                        "escaped character cannot be an ASCII number or alphabetic: {}",
                        raw_input
                    );
                }
                c => (c, c.len_utf8()),
            };
            *input = &input[len..];
            c
        }
        _ => {
            let mut chars = input.chars();
            let c = chars.next().unwrap();
            *input = chars.as_str();
            c
        }
    };
    if input.starts_with('\'') {
        *input = &input[1..];
        Ok(c)
    } else {
        bail!("unclosed character: {}", raw_input);
    }
}

fn scan_string<'i>(input: &mut &'i str) -> Result<Cow<'i, str>> {
    if input.is_empty() {
        bail!("empty");
    }

    fn scan_inner<'i>(input: &mut &'i str, ends: &str, raw: bool) -> Result<Cow<'i, str>> {
        let raw_input = *input;
        assert!(!ends.is_empty());
        if raw {
            let len = input
                .find(ends)
                .context(format!("unclosed string: {}", raw_input))?;
            let res = &input[..len];
            *input = &input[len + ends.len()..];
            Ok(Cow::Borrowed(res))
        } else {
            let first = ends.as_bytes()[0];
            let mut res = String::new();
            loop {
                if input.len() < ends.len() {
                    bail!("unclosed string");
                }

                let idx = memchr::memchr2(b'\\', first, input.as_bytes())
                    .context(format!("unclosed string: {}", raw_input))?;
                if input.as_bytes()[idx] == b'\\' {
                    res.push_str(&input[..idx]);
                    *input = &input[idx + 1..];
                    if let Some(c) = input.chars().next() {
                        let (c, len) = match c {
                            'a' => ('\u{07}', 1), // BEL
                            'b' => ('\u{08}', 1), // BS
                            'f' => ('\u{0C}', 1),
                            'n' => ('\u{0A}', 1),
                            'r' => ('\u{0D}', 1),
                            't' => ('\u{09}', 1),
                            'v' => ('\u{0B}', 1),
                            'u' | 'x' => (scan_escaped_char(input)?, 0),
                            '0' => ('\0', 1),
                            _ if c.is_ascii_alphanumeric() => {
                                bail!(
                                    "escaped character cannot be an ASCII number or alphabetic: {}",
                                    raw_input
                                );
                            }
                            _ => (c, c.len_utf8()),
                        };
                        res.push(c);
                        *input = &input[len..];
                    } else {
                        bail!("unclosed string: {}", raw_input);
                    }
                } else {
                    res.push_str(&input[..idx]);
                    *input = &input[idx..];
                    if input.starts_with(ends) {
                        *input = &input[ends.len()..];
                        break;
                    } else {
                        let c = input.chars().next().unwrap();
                        res.push(c);
                        *input = &input[c.len_utf8()..];
                    }
                }
            }
            Ok(Cow::Owned(res))
        }
    }

    match input.as_bytes()[0] {
        b'\'' => bail!("string should be start with \' or \" or `: {}", input),
        b'\"' => {
            *input = &input[1..];
            scan_inner(input, "\"", false)
        }
        b'`' => {
            *input = &input[1..];
            scan_inner(input, "`", true)
        }
        b'r' => {
            *input = &input[1..];
            let hash_len = {
                let mut len = 0;
                for c in input.chars() {
                    if c == '#' {
                        len += 1;
                    } else {
                        break;
                    }
                }
                len
            };
            *input = &input[hash_len..];
            if !input.starts_with('\"') {
                bail!("value should be a string: {}", input);
            }
            *input = &input[1..];
            let mut ends = CompactString::new("\"");
            (0..hash_len).for_each(|_| ends.push('#'));
            scan_inner(input, &ends, true)
        }
        _ => bail!("value should be a string: {}", input),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let ref mut s = r##"`abc\d`"##;
        assert_eq!(scan_string(s).unwrap(), r##"abc\d"##);
        assert_eq!(*s, "");

        let ref mut s = r##""\a\b\f\n\r\t\v\"\u4e00\x400\0""##;
        assert_eq!(
            scan_string(s).unwrap(),
            "\x07\x08\x0c\x0a\x0d\x09\x0b\"\u{4e00}\u{400}\0"
        );
        assert_eq!(*s, "");

        let ref mut s = r##"r"abc'\""##;
        assert_eq!(scan_string(s).unwrap(), r##"abc'\"##);
        assert_eq!(*s, "");

        let ref mut s = r###"r#"abc'#"\"##"###;
        assert_eq!(scan_string(s).unwrap(), r##"abc'#"\"##);
        assert_eq!(*s, "#");
    }

    #[test]
    fn test_char() {
        let ref mut s = "'\"''\t''x''\\u4e00''\\x0d''\\x400''\0'";
        assert_eq!(scan_char(s).unwrap(), '\"');
        assert_eq!(scan_char(s).unwrap(), '\t');
        assert_eq!(scan_char(s).unwrap(), 'x');
        assert_eq!(scan_char(s).unwrap(), '\u{4e00}');
        assert_eq!(scan_char(s).unwrap(), '\x0d');
        assert_eq!(scan_char(s).unwrap(), '\u{400}');
        assert_eq!(scan_char(s).unwrap(), '\0');
        assert_eq!(*s, "");
    }

    #[test]
    fn test_number() {
        let ref mut s = "12345";
        assert_eq!(scan_number(s).unwrap(), 12345);
        assert_eq!(*s, "");

        let ref mut s = "0x12_345";
        assert_eq!(scan_number(s).unwrap(), 0x12_345);
        assert_eq!(*s, "");

        let ref mut s = "0b10_001";
        assert_eq!(scan_number(s).unwrap(), 0b10_001);
        assert_eq!(*s, "");

        let ref mut s = "12_345.0";
        assert_eq!(scan_number(s).unwrap(), 12_345);
        assert_eq!(*s, ".0");

        let ref mut s = "-12_345.0";
        assert_eq!(scan_number(s).unwrap(), -12_345);
        assert_eq!(*s, ".0");

        let ref mut s = "+12_345.0";
        assert_eq!(scan_number(s).unwrap(), 12_345);
        assert_eq!(*s, ".0");

        assert!(scan_number(&mut "_1").is_err());
        assert!(scan_number(&mut "1_").is_err());
        assert!(scan_number(&mut "1__2").is_err());
        assert!(scan_number(&mut "0x_ff").is_err());
        assert!(scan_number(&mut "0b_1").is_err());
    }

    #[test]
    fn test_ist() {
        let ist = r##"actual '"' preamble "\n\\startindex\n"
        % ppp "a "
        postamble r"\n\stopindex\n"
        "##;
        let ist = IstFile::read_string(ist).unwrap();
        assert_eq!(ist.actual, '\"');
        assert_eq!(ist.preamble, "\n\\startindex\n");
        assert_eq!(ist.postamble, r"\n\stopindex\n");
    }

    #[test]
    fn test_index_entry() {
        let mut ist = IstFile::default();
        ist.actual = '=';
        ist.quote = '!';
        ist.level = '>';
        let (ist_input, _) = ist.split();

        let s = r##"\indexentry{@currdir=\verb!*&\@currdir&|hdclindex{2}{usage}}{MMMMI-1}"##;
        assert_eq!(
            IndexEntry::parse_index_line(&ist_input, s).unwrap(),
            Some(IndexEntry {
                levels: vec![(
                    Some(String::from("@currdir")),
                    String::from(r"\verb*&\@currdir&")
                )]
                .into(),
                range: RangeKind::None,
                page_commands: Some(CompactString::from("hdclindex{2}{usage}")),
                pages: vec![IndexPage::Roman(4001), IndexPage::Arabic(1)].into(),
                pages_raw: String::from("MMMMI-1"),
            })
        );
        let s = r##"\indexentry{TeX and LaTeX2e commands:=\TeX{} and \LaTeXe{} commands:>sixt@@n={\verbatim@font !\verb*&!\sixt@@n&}|hdclindex{64}{}}{MMMMV-01-55-MMMD-64-M}"##;
        assert_eq!(
            IndexEntry::parse_index_line(&ist_input, s).unwrap(),
            Some(IndexEntry {
                levels: vec![
                    (
                        Some(String::from("TeX and LaTeX2e commands:")),
                        String::from(r"\TeX{} and \LaTeXe{} commands:")
                    ),
                    (
                        Some(String::from("sixt@@n")),
                        String::from(r"{\verbatim@font \verb*&\sixt@@n&}")
                    )
                ]
                .into(),
                range: RangeKind::None,
                page_commands: Some(CompactString::from("hdclindex{64}{}")),
                pages: vec![
                    IndexPage::Roman(4005),
                    IndexPage::Arabic(1),
                    IndexPage::Arabic(55),
                    IndexPage::Roman(3500),
                    IndexPage::Arabic(64),
                    IndexPage::Roman(1000),
                ]
                .into(),
                pages_raw: String::from("MMMMV-01-55-MMMD-64-M"),
            })
        );
        let s =
            r##"\indexentry{sixt@@n={\verbatim@font !\verb*&!\sixt@@n&}|(hdclindex{}{NA}}{MMMMV}"##;
        assert_eq!(
            IndexEntry::parse_index_line(&ist_input, s).unwrap(),
            Some(IndexEntry {
                levels: vec![(
                    Some(String::from("sixt@@n")),
                    String::from(r"{\verbatim@font \verb*&\sixt@@n&}")
                )]
                .into(),
                range: RangeKind::Open,
                page_commands: Some(CompactString::from("hdclindex{}{NA}")),
                pages: vec![IndexPage::Roman(4005)].into(),
                pages_raw: String::from("MMMMV")
            })
        );
    }
}
