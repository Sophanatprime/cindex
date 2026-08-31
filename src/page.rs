use std::str::FromStr;

use anyhow::{Result, anyhow, bail};
use smallvec::SmallVec;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(non_camel_case_types)]
pub enum IndexPageKind {
    /// Arabic number, 0~9.
    Arabic,
    /// Lowercase roman number, i=1,v=5,x=10,l=50,c=100,d=500,m=1000.
    roman,
    /// Uppercase roman number, I=1,V=5,X=10,L=50,C=100,D=500,M=1000.
    Roman,
    /// Lowercase latin number, a-z.
    alpha,
    /// Uppercase latin number, A-Z.
    Alpha,
    /// 中文数字串。如：一零五，一〇五。
    ZhDigits,
    /// 中文数。如：一百零五。
    ZhNumber,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexPage {
    pub kind: IndexPageKind,
    pub value: u64,
}

impl IndexPage {
    #[allow(non_snake_case)]
    pub const fn Arabic(value: u64) -> Self {
        Self {
            kind: IndexPageKind::Arabic,
            value,
        }
    }

    #[allow(non_snake_case)]
    pub const fn roman(value: u64) -> Self {
        Self {
            kind: IndexPageKind::roman,
            value,
        }
    }

    #[allow(non_snake_case)]
    pub const fn Roman(value: u64) -> Self {
        Self {
            kind: IndexPageKind::Roman,
            value,
        }
    }

    #[allow(non_snake_case)]
    pub const fn alpha(value: u64) -> Self {
        Self {
            kind: IndexPageKind::alpha,
            value,
        }
    }

    #[allow(non_snake_case)]
    pub const fn Alpha(value: u64) -> Self {
        Self {
            kind: IndexPageKind::Alpha,
            value,
        }
    }

    #[allow(non_snake_case)]
    pub const fn ZhDigits(value: u64) -> Self {
        Self {
            kind: IndexPageKind::ZhDigits,
            value,
        }
    }

    #[allow(non_snake_case)]
    pub const fn ZhNumber(value: u64) -> Self {
        Self {
            kind: IndexPageKind::ZhNumber,
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Precedence(pub(crate) [u8; 7]);

impl Precedence {
    pub fn display(&self) -> String {
        let s = [
            "roman", "Arabic", "alpha", "Roman", "Alpha", "ZhDigits", "ZhNumber",
        ];
        let mut res =
            String::with_capacity(s.iter().map(|i| i.len()).sum::<usize>() + (s.len() - 1) * 2);
        res.push('[');
        for i in 0..s.len() {
            res.push_str(s[self.0[i] as usize]);
            if i == s.len() - 1 {
                res.push(']');
            } else {
                res.push_str(", ");
            }
        }
        res
    }
}

pub trait PrecedenceProvider {
    fn precedence_index(&self, page_kind: IndexPageKind) -> Option<usize>;

    fn to_precedence(&self) -> Precedence {
        let pos_table = [
            (0usize, self.precedence_index(IndexPageKind::roman)),
            (1usize, self.precedence_index(IndexPageKind::Arabic)),
            (2usize, self.precedence_index(IndexPageKind::alpha)),
            (3usize, self.precedence_index(IndexPageKind::Roman)),
            (4usize, self.precedence_index(IndexPageKind::Alpha)),
            (5usize, self.precedence_index(IndexPageKind::ZhDigits)),
            (6usize, self.precedence_index(IndexPageKind::ZhNumber)),
        ];
        let mut present = pos_table
            .iter()
            .filter_map(|s| s.1.and_then(|i| Some((s.0, i))))
            .collect::<SmallVec<[(usize, usize); 7]>>();
        present.sort_by_key(|i| i.1);
        pos_table.iter().for_each(|s| {
            if matches!(s.1, None) {
                present.push((s.0, 255));
            }
        });

        assert_eq!(present.len(), 7);
        let mut prec_table = [0; 7];
        for (i, (p, _)) in present.into_iter().enumerate() {
            prec_table[p] = i as u8;
        }
        Precedence(prec_table)
    }
}

pub struct StringPrecedence<T: AsRef<str>>(pub T);
pub struct IteratorPrecedence<I: AsRef<str>, T: Iterator<Item = I> + Clone>(pub T);
pub struct TablePrecedence<const N: usize>(pub [u8; N]);

impl<T: AsRef<str>> PrecedenceProvider for StringPrecedence<T> {
    fn precedence_index(&self, page_kind: IndexPageKind) -> Option<usize> {
        self.0.as_ref().find(match page_kind {
            IndexPageKind::roman => 'r',
            IndexPageKind::Arabic => 'n',
            IndexPageKind::alpha => 'a',
            IndexPageKind::Roman => 'R',
            IndexPageKind::Alpha => 'A',
            IndexPageKind::ZhDigits => '串',
            IndexPageKind::ZhNumber => '数',
        })
    }
}

impl<T: AsRef<str>> PrecedenceProvider for T {
    fn precedence_index(&self, page_kind: IndexPageKind) -> Option<usize> {
        StringPrecedence(self).precedence_index(page_kind)
    }
}

impl<I: AsRef<str>, A: Clone + Iterator<Item = I>> PrecedenceProvider for IteratorPrecedence<I, A> {
    fn precedence_index(&self, page_kind: IndexPageKind) -> Option<usize> {
        self.0.clone().position(|s| match page_kind {
            IndexPageKind::roman => matches!(s.as_ref(), "roman"),
            IndexPageKind::Arabic => matches!(s.as_ref(), "Arabic"),
            IndexPageKind::alpha => matches!(s.as_ref(), "alpha"),
            IndexPageKind::Roman => matches!(s.as_ref(), "Roman"),
            IndexPageKind::Alpha => matches!(s.as_ref(), "Alpha"),
            IndexPageKind::ZhDigits => matches!(s.as_ref(), "ZhDigits"),
            IndexPageKind::ZhNumber => matches!(s.as_ref(), "ZhNumber"),
        })
    }
}

impl PrecedenceProvider for TablePrecedence<7> {
    fn precedence_index(&self, page_kind: IndexPageKind) -> Option<usize> {
        let prec = match page_kind {
            IndexPageKind::roman => self.0[0],
            IndexPageKind::Arabic => self.0[1],
            IndexPageKind::alpha => self.0[2],
            IndexPageKind::Roman => self.0[3],
            IndexPageKind::Alpha => self.0[4],
            IndexPageKind::ZhDigits => self.0[5],
            IndexPageKind::ZhNumber => self.0[6],
        };
        Some(prec as _)
    }

    fn to_precedence(&self) -> Precedence {
        Precedence(self.0)
    }
}

impl IndexPage {
    pub fn as_u64(&self) -> u64 {
        self.value
    }

    pub fn as_kind(&self) -> (&str, u64) {
        match self.kind {
            IndexPageKind::Arabic => ("Arabic", self.value),
            IndexPageKind::roman => ("roman", self.value),
            IndexPageKind::Roman => ("Roman", self.value),
            IndexPageKind::alpha => ("alpha", self.value),
            IndexPageKind::Alpha => ("Alpha", self.value),
            IndexPageKind::ZhDigits => ("ZhDigits", self.value),
            IndexPageKind::ZhNumber => ("ZhNumber", self.value),
        }
    }
}

impl FromStr for IndexPage {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        if s.is_empty() {
            bail!("empty index page");
        }

        if s.bytes().all(|b| b.is_ascii_digit()) {
            let value = s
                .parse::<u64>()
                .map_err(|_| anyhow!("arabic number overflow"))?;

            return Ok(IndexPage {
                kind: IndexPageKind::Arabic,
                value,
            });
        }

        if s.bytes().all(|b| b.is_ascii_alphabetic()) {
            let is_lower = s.bytes().all(|b| b.is_ascii_lowercase());
            let is_upper = s.bytes().all(|b| b.is_ascii_uppercase());

            if !is_lower && !is_upper {
                bail!("mixed-case alphabet is not allowed");
            }

            let is_roman = if is_lower {
                s.bytes()
                    .all(|b| matches!(b, b'i' | b'v' | b'x' | b'l' | b'c' | b'd' | b'm'))
            } else {
                s.bytes()
                    .all(|b| matches!(b, b'I' | b'V' | b'X' | b'L' | b'C' | b'D' | b'M'))
            };

            if is_roman {
                let res = parse_roman(s)?;
                return Ok(if is_lower {
                    IndexPage {
                        kind: IndexPageKind::roman,
                        value: res,
                    }
                } else {
                    IndexPage {
                        kind: IndexPageKind::Roman,
                        value: res,
                    }
                });
            }

            let res = parse_alpha(s)?;
            return Ok(if is_lower {
                IndexPage {
                    kind: IndexPageKind::alpha,
                    value: res,
                }
            } else {
                IndexPage {
                    kind: IndexPageKind::Alpha,
                    value: res,
                }
            });
        }

        if s.chars().all(is_zh_digit) {
            return parse_zh_digits(s).map(|n| IndexPage {
                kind: IndexPageKind::ZhDigits,
                value: n,
            });
        }

        bail!("invalid number: {}", s)
    }
}

fn parse_roman(s: &str) -> Result<u64> {
    #[inline]
    fn eat(chars: &[u8], pos: &mut usize, ch: u8) -> bool {
        if *pos < chars.len() && chars[*pos] == ch {
            *pos += 1;
            true
        } else {
            false
        }
    }

    let bytes = s.as_bytes();

    if bytes.is_empty() {
        bail!("empty roman numeral");
    }

    let upper;
    let chars: &[u8] = if bytes[0].is_ascii_lowercase() {
        upper = bytes.iter().map(u8::to_ascii_uppercase).collect::<Vec<_>>();
        &upper
    } else {
        bytes
    };

    let mut pos = 0usize;
    let mut value = 0u64;

    while eat(chars, &mut pos, b'M') {
        value = value
            .checked_add(1000)
            .ok_or_else(|| anyhow!("roman numeral overflow"))?;
    }

    if pos + 1 < chars.len() && chars[pos] == b'C' && chars[pos + 1] == b'M' {
        value += 900;
        pos += 2;
    } else if pos + 1 < chars.len() && chars[pos] == b'C' && chars[pos + 1] == b'D' {
        value += 400;
        pos += 2;
    } else {
        if eat(chars, &mut pos, b'D') {
            value += 500;
        }

        let mut count = 0;
        while count < 3 && eat(chars, &mut pos, b'C') {
            value += 100;
            count += 1;
        }

        if pos < chars.len() && chars[pos] == b'C' {
            bail!("too many consecutive C");
        }
    }

    if pos + 1 < chars.len() && chars[pos] == b'X' && chars[pos + 1] == b'C' {
        value += 90;
        pos += 2;
    } else if pos + 1 < chars.len() && chars[pos] == b'X' && chars[pos + 1] == b'L' {
        value += 40;
        pos += 2;
    } else {
        if eat(chars, &mut pos, b'L') {
            value += 50;
        }

        let mut count = 0;
        while count < 3 && eat(chars, &mut pos, b'X') {
            value += 10;
            count += 1;
        }

        if pos < chars.len() && chars[pos] == b'X' {
            bail!("too many consecutive X");
        }
    }

    if pos + 1 < chars.len() && chars[pos] == b'I' && chars[pos + 1] == b'X' {
        value += 9;
        pos += 2;
    } else if pos + 1 < chars.len() && chars[pos] == b'I' && chars[pos + 1] == b'V' {
        value += 4;
        pos += 2;
    } else {
        if eat(chars, &mut pos, b'V') {
            value += 5;
        }

        let mut count = 0;
        while count < 3 && eat(chars, &mut pos, b'I') {
            value += 1;
            count += 1;
        }

        if pos < chars.len() && chars[pos] == b'I' {
            bail!("too many consecutive I");
        }
    }

    if pos != chars.len() {
        bail!("invalid roman numeral");
    }

    Ok(value)
}

fn parse_alpha(s: &str) -> Result<u64> {
    if s.is_empty() {
        bail!("empty alphabetic sequence");
    }

    let mut value = 0u64;

    let mut is_lower = false;
    let mut is_upper = false;

    for c in s.chars() {
        let digit = match c {
            'a'..='z' => {
                is_lower = true;
                (c as u64) - ('a' as u64) + 1
            }
            'A'..='Z' => {
                is_upper = true;
                (c as u64) - ('A' as u64) + 1
            }
            _ => bail!("invalid alphabetic character '{}'", c),
        };

        if is_lower && is_upper {
            bail!("mixed-case alphabetic sequence");
        }

        value = value
            .checked_mul(26)
            .ok_or_else(|| anyhow!("alphabetic value overflow"))?;

        value = value
            .checked_add(digit)
            .ok_or_else(|| anyhow!("alphabetic value overflow"))?;
    }

    Ok(value)
}

fn is_zh_digit(c: char) -> bool {
    matches!(
        c,
        '零' | '〇' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
    )
}

fn parse_zh_digits(s: &str) -> Result<u64> {
    let mut value = 0u64;
    for c in s.chars() {
        let n = match c {
            '零' | '〇' => 0,
            '一' => 1,
            '二' => 2,
            '三' => 3,
            '四' => 4,
            '五' => 5,
            '六' => 6,
            '七' => 7,
            '八' => 8,
            '九' => 9,
            _ => bail!("invalid character of chinese digits '{}'", c),
        };
        value = value
            .checked_mul(10)
            .ok_or_else(|| anyhow!("chinese digits overflow"))?;
        value = value
            .checked_add(n)
            .ok_or_else(|| anyhow!("chinese digits overflow"))?
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arabic() {
        assert_eq!(IndexPage::from_str("000").unwrap(), IndexPage::Arabic(0));
        assert_eq!(IndexPage::from_str("002").unwrap(), IndexPage::Arabic(2));
        assert_eq!(IndexPage::from_str("2").unwrap(), IndexPage::Arabic(2));
        assert_eq!(IndexPage::from_str("102").unwrap(), IndexPage::Arabic(102));
        assert!(IndexPage::from_str("-1").is_err());
    }

    #[test]
    fn test_lowercase_roman() {
        assert_eq!(IndexPage::from_str("x").unwrap(), IndexPage::roman(10));
        assert_eq!(IndexPage::from_str("cclv").unwrap(), IndexPage::roman(255));
        assert_eq!(
            IndexPage::from_str("mmmiv").unwrap(),
            IndexPage::roman(3004)
        );
        assert!(IndexPage::from_str("liiii").is_err());
    }

    #[test]
    fn test_uppercase_roman() {
        assert_eq!(IndexPage::from_str("X").unwrap(), IndexPage::Roman(10));
        assert_eq!(IndexPage::from_str("CCLV").unwrap(), IndexPage::Roman(255));
        assert_eq!(
            IndexPage::from_str("MMMIV").unwrap(),
            IndexPage::Roman(3004)
        );
        assert!(IndexPage::from_str("LIIII").is_err());
    }

    #[test]
    fn test_lowercase_alpha() {
        assert_eq!(IndexPage::from_str("a").unwrap(), IndexPage::alpha(1));
        assert_eq!(IndexPage::from_str("ia").unwrap(), IndexPage::alpha(235));
        assert_eq!(
            IndexPage::from_str("aaaaaa").unwrap(),
            IndexPage::alpha(12356631)
        );
        assert!(IndexPage::from_str("aaaaa.a").is_err());
    }

    #[test]
    fn test_uppercase_alpha() {
        assert_eq!(IndexPage::from_str("A").unwrap(), IndexPage::Alpha(1));
        assert_eq!(IndexPage::from_str("IA").unwrap(), IndexPage::Alpha(235));
        assert_eq!(
            IndexPage::from_str("AAAAAA").unwrap(),
            IndexPage::Alpha(12356631)
        );
        assert!(IndexPage::from_str("AAAAA.A").is_err());
    }

    #[test]
    fn test_zhdigits() {
        assert_eq!(
            IndexPage::from_str("〇〇一").unwrap(),
            IndexPage::ZhDigits(1)
        );
        assert_eq!(
            IndexPage::from_str("一零八零零九").unwrap(),
            IndexPage::ZhDigits(108009)
        );
        assert_eq!(
            IndexPage::from_str("一〇八〇〇九").unwrap(),
            IndexPage::ZhDigits(108009)
        );
        assert!(IndexPage::from_str("壹").is_err());
    }
}
