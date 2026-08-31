use modular_bitfield::{
    bitfield,
    specifiers::{B3, B5, B10, B11},
};

mod data;
pub(crate) use data::MAX_RADICAL;
pub use data::Radical;
use data::*;

/// 单个汉字的信息
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CjkInfo {
    /// 部首。
    radical: Radical,
    /// 除部首外的笔画数，可能为负数。
    additional_strokes: i8,
    /// Unicode总笔画数（kTotalStrokes）。
    k_total_strokes: u8,
    /// 字形的总笔画数。
    glyph_total_strokes: u8,
    extra: CjkExtra,
}

impl Default for CjkInfo {
    fn default() -> Self {
        CjkInfo {
            radical: Radical::R1,
            additional_strokes: 0,
            k_total_strokes: 0,
            glyph_total_strokes: 0,
            extra: CjkExtra::new(),
        }
    }
}

impl CjkInfo {
    pub fn is_invalid(&self) -> bool {
        self.k_total_strokes == 0
    }
}

#[repr(transparent)]
#[bitfield]
#[derive(Debug, Clone, Copy)]
struct CjkExtra {
    mandarin: B10,
    mandarin_tone: B3,
    cantonese: B11,
    cantonese_tone: B3,
    #[skip]
    extra: B5,
}

impl CjkInfo {
    pub const fn new(
        radical: Radical,
        additional_strokes: i8,
        k_total_strokes: u8,
        glyph_total_strokes: u8,
        mandarin: u16,
        mandarin_tone: u8,
        cantonese: u16,
        cantonese_tone: u8,
        extra: u8,
    ) -> Self {
        let bits = (mandarin as u32 & 0x3FF)
            | ((mandarin_tone as u32 & 0x7) << 10)
            | ((cantonese as u32 & 0x7FF) << 13)
            | ((cantonese_tone as u32 & 0x7) << 24)
            | ((extra as u32 & 0x1F) << 27);
        CjkInfo {
            radical,
            additional_strokes,
            k_total_strokes,
            glyph_total_strokes,
            extra: CjkExtra::from_bytes(bits.to_le_bytes()),
        }
    }

    /// 部首。
    pub fn radical(&self) -> Radical {
        self.radical
    }

    /// 除部首外的笔画数。可能为负数。
    pub fn additional_strokes(&self) -> i8 {
        self.additional_strokes
    }

    /// Unicode总笔画数（kTotalStrokes）。
    pub fn k_total_strokes(&self) -> u8 {
        self.k_total_strokes
    }

    /// 字形的总笔画数。
    pub fn glyph_total_strokes(&self) -> u8 {
        self.glyph_total_strokes
    }

    /// 普通话读音。
    pub fn mandarin(&self) -> Option<(&'static str, u8)> {
        let m = self.extra.mandarin();
        m.ne(&0).then_some((
            MANDARIN_READINGS.index(m as usize).unwrap(),
            self.extra.mandarin_tone(),
        ))
    }

    /// 粤语读音。
    pub fn cantonese(&self) -> Option<(&'static str, u8)> {
        let c = self.extra.cantonese();
        c.ne(&0).then_some((
            CANTONESE_READINGS.index(c as usize).unwrap(),
            self.extra.cantonese_tone(),
        ))
    }
}

pub fn radical(s: &str) -> Option<Radical> {
    STR_RADICAL.get(s).copied()
}

pub fn radical_strs()
-> impl Iterator<Item = &'static &'static str> + DoubleEndedIterator + ExactSizeIterator {
    STR_RADICAL.keys()
}

pub fn radical_entries() -> impl Iterator<Item = (&'static &'static str, &'static Radical)>
+ DoubleEndedIterator
+ ExactSizeIterator {
    STR_RADICAL.entries()
}

pub fn kangxi_ideograph(r: Radical) -> (Option<char>, char) {
    let res = KX_IDEOS[r as usize];
    (res.0.ne(&'\0').then_some(res.0), res.1)
}

pub fn cantonese_index(s: &str) -> Option<usize> {
    CANTONESE_READINGS.get_index(s)
}

pub fn mandarin_index(s: &str) -> Option<usize> {
    MANDARIN_READINGS.get_index(s)
}

pub fn cjk_info(c: char) -> Option<CjkInfo> {
    let slot = c as u32;
    // Safety: can be sure that the index is less than len.
    unsafe {
        match slot {
            // CJK Uni
            0x4E00..=0x9FFF => *INFO_CJK_UNI.get_unchecked((slot - 0x4E00) as usize),
            // CJK Ext A
            0x3400..=0x4DBF => *INFO_CJK_EXT_A.get_unchecked((slot - 0x3400) as usize),
            // CJK Comp
            0xF900..=0xFAFF => *INFO_CJK_COMP.get_unchecked((slot - 0xF900) as usize),
            // CJK CJK Ext B
            0x20000..=0x2A6DF => *INFO_CJK_EXT_B.get_unchecked((slot - 0x20000) as usize),
            // CJK Ext C,D,E,F,I
            0x2A700..=0x2EBF0 => *INFO_CJK_EXT_CDEFI.get_unchecked((slot - 0x2A700) as usize),
            // CJK Comp Supp
            0x2F800..=0x2FA1F => *INFO_CJK_COMP_SUPP.get_unchecked((slot - 0x2F800) as usize),
            // CJK Ext G,H,J
            0x30000..=0x3347F => *INFO_CJK_EXT_GHJ.get_unchecked((slot - 0x30000) as usize),
            _ => None,
        }
    }
}

/// 笔画
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stroke {
    /// 横
    S1 = 1,
    /// 竖
    S2,
    /// 撇
    S3,
    /// 点（捺）
    S4,
    /// 折
    S5,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RawStrokes(pub(crate) &'static [u8]);

impl RawStrokes {
    pub unsafe fn from_strokes(bytes: &'static [u8]) -> Option<Self> {
        for (idx, &byte) in bytes.iter().enumerate() {
            let low = byte & 0x0F;
            let high = byte >> 4;
            if low > 5 || high == 0 || high > 5 {
                return None;
            }
            if low == 0 && idx + 1 < bytes.len() {
                return None;
            }
        }
        Some(RawStrokes(bytes))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        if let Some(&last) = self.0.last() {
            if last & 0x07 != 0 {
                self.0.len() * 2
            } else {
                self.0.len() * 2 - 1
            }
        } else {
            0
        }
    }

    pub fn get(&self, index: usize) -> Option<Stroke> {
        if index >= self.len() {
            return None;
        }

        let byte_idx = index / 2;
        let is_lower_nibble = index % 2 != 0;
        let byte = self.0[byte_idx];
        let stroke = if is_lower_nibble {
            byte & 0x0F
        } else {
            byte >> 4
        };

        Some(match stroke {
            1 => Stroke::S1,
            2 => Stroke::S2,
            3 => Stroke::S3,
            4 => Stroke::S4,
            5 => Stroke::S5,
            _ => unreachable!("Corrupted stroke data or invalid padding: {:02X}", stroke),
        })
    }

    pub fn strokes(&self) -> RawStrokesIter {
        RawStrokesIter {
            raw: self.0,
            front_cursor: 0,
            back_cursor: self.len(),
        }
    }
}

pub struct RawStrokesIter {
    raw: &'static [u8],
    front_cursor: usize,
    back_cursor: usize,
}

impl RawStrokesIter {
    /// 将解析出的 4 bit 整数映射为 Stroke 枚举
    #[inline(always)]
    fn decode_stroke(val: u8) -> Stroke {
        match val {
            1 => Stroke::S1,
            2 => Stroke::S2,
            3 => Stroke::S3,
            4 => Stroke::S4,
            5 => Stroke::S5,
            _ => unreachable!("Corrupted stroke data or invalid padding: {}", val),
        }
    }

    /// 根据虚拟游标位置获取具体的笔画数值 (1~5)
    #[inline(always)]
    fn get_raw_stroke_at(&self, cursor: usize) -> u8 {
        let byte_idx = cursor / 2;
        let is_lower_nibble = cursor % 2 != 0;
        let byte = self.raw[byte_idx];
        if is_lower_nibble {
            byte & 0x0F
        } else {
            byte >> 4
        }
    }
}

impl Iterator for RawStrokesIter {
    type Item = Stroke;
    fn next(&mut self) -> Option<Self::Item> {
        if self.front_cursor >= self.back_cursor {
            return None;
        }
        let val = self.get_raw_stroke_at(self.front_cursor);
        self.front_cursor += 1;
        Some(Self::decode_stroke(val))
    }
}

impl DoubleEndedIterator for RawStrokesIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front_cursor >= self.back_cursor {
            return None;
        }
        self.back_cursor -= 1;
        let val = self.get_raw_stroke_at(self.back_cursor);
        Some(Self::decode_stroke(val))
    }
}

impl ExactSizeIterator for RawStrokesIter {
    #[inline]
    fn len(&self) -> usize {
        self.back_cursor.saturating_sub(self.front_cursor)
    }
}

impl std::iter::FusedIterator for RawStrokesIter {}

pub fn ordered_strokes(c: char) -> Option<RawStrokes> {
    let slot = c as u32;
    let s = unsafe {
        match slot {
            // CJK Uni
            0x4E00..=0x9FFF => {
                let idx = (slot - 0x4E00) as usize;
                let start = *ORDER_IDX_CJK_UNI.get_unchecked(idx) as usize;
                let end = *ORDER_IDX_CJK_UNI.get_unchecked(idx + 1) as usize;
                Some(RawStrokes(ORDER_CJK_UNI.get_unchecked(start..end)))
            }
            // CJK Ext A
            0x3400..=0x4DBF => {
                let idx = (slot - 0x3400) as usize;
                let start = *ORDER_IDX_CJK_EXT_A.get_unchecked(idx) as usize;
                let end = *ORDER_IDX_CJK_EXT_A.get_unchecked(idx + 1) as usize;
                Some(RawStrokes(ORDER_CJK_EXT_A.get_unchecked(start..end)))
            }
            // CJK Comp
            0xF900..=0xFAFF => {
                let idx = (slot - 0xF900) as usize;
                let start = *ORDER_IDX_CJK_COMP.get_unchecked(idx) as usize;
                let end = *ORDER_IDX_CJK_COMP.get_unchecked(idx + 1) as usize;
                Some(RawStrokes(ORDER_CJK_COMP.get_unchecked(start..end)))
            }
            // CJK CJK Ext B
            0x20000..=0x2A6DF => {
                let idx = (slot - 0x20000) as usize;
                let start = *ORDER_IDX_CJK_EXT_B.get_unchecked(idx) as usize;
                let end = *ORDER_IDX_CJK_EXT_B.get_unchecked(idx + 1) as usize;
                Some(RawStrokes(ORDER_CJK_EXT_B.get_unchecked(start..end)))
            }
            // CJK Ext C,D,E,F,I
            0x2A700..=0x2EBF0 => {
                let idx = (slot - 0x2A700) as usize;
                let start = *ORDER_IDX_CJK_EXT_CDEFI.get_unchecked(idx) as usize;
                let end = *ORDER_IDX_CJK_EXT_CDEFI.get_unchecked(idx + 1) as usize;
                Some(RawStrokes(ORDER_CJK_EXT_CDEFI.get_unchecked(start..end)))
            }
            // CJK Comp Supp
            0x2F800..=0x2FA1F => {
                let idx = (slot - 0x2F800) as usize;
                let start = *ORDER_IDX_CJK_COMP_SUPP.get_unchecked(idx) as usize;
                let end = *ORDER_IDX_CJK_COMP_SUPP.get_unchecked(idx + 1) as usize;
                Some(RawStrokes(ORDER_CJK_COMP_SUPP.get_unchecked(start..end)))
            }
            // CJK Ext G,H,J
            0x30000..=0x3347F => {
                let idx = (slot - 0x30000) as usize;
                let start = *ORDER_IDX_CJK_EXT_GHJ.get_unchecked(idx) as usize;
                let end = *ORDER_IDX_CJK_EXT_GHJ.get_unchecked(idx + 1) as usize;
                Some(RawStrokes(ORDER_CJK_EXT_GHJ.get_unchecked(start..end)))
            }
            _ => None,
        }
    };
    s.and_then(|s| (!s.0.is_empty()).then_some(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strokes() {
        assert_eq!(ordered_strokes('\u{9FFD}'), None);

        let ding_stroke = ordered_strokes('丁');
        assert_eq!(ding_stroke, Some(RawStrokes(&[0x12])));

        let mu_stroke = ordered_strokes('木');
        assert_eq!(mu_stroke, Some(RawStrokes(&[0x12, 0x34])));
        let mut iter = mu_stroke.unwrap().strokes();
        assert_eq!(iter.next(), Some(Stroke::S1));
        assert_eq!(iter.next_back(), Some(Stroke::S4));
        assert_eq!(iter.next_back(), Some(Stroke::S3));
        assert_eq!(iter.next(), Some(Stroke::S2));
        assert_eq!(iter.next(), None);

        assert_eq!(
            ordered_strokes('𢝃'),
            Some(RawStrokes(&[0x41, 0x43, 0x45, 0x25, 0x24, 0x54, 0x40]))
        );
    }

    #[test]
    fn test_info() {
        let info = cjk_info('𢝃').unwrap();
        println!("{:?}, {:?}", info, info.mandarin());
        println!(
            "{:?}, {:?}",
            cjk_info('𢝂').unwrap(),
            cjk_info('𢝂').unwrap().mandarin()
        );

        let max_len = CANTONESE_READINGS.iter().map(|b| b.len()).max().unwrap();
        println!("cantonese max len: {}", max_len);
        let max_len = MANDARIN_READINGS.iter().map(|b| b.len()).max().unwrap();
        println!("mandarin max len: {}", max_len);
    }
}
