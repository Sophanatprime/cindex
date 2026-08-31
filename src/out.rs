use std::cmp::Ordering as StdOrdering;
use std::ops::Deref;

use compact_str::{CompactString, ToCompactString};
use indexmap::IndexMap;
use smallvec::SmallVec;

use crate::ffi::lua::prelude::{Lua, LuaResult, LuaTable};
use crate::{IndexPage, IndexPageKind, Precedence, han, style::*};

#[derive(Debug, Clone)]
pub struct MergedEntry {
    pub levels: SmallVec<[(Option<String>, String); 1]>,
    pub pages: MergedPages,
}

#[derive(Debug, Clone)]
pub enum MergedPage {
    Single {
        page_command: Option<CompactString>,
        page: (SmallVec<[IndexPage; 1]>, String),
    },
    Range {
        page_command: Option<CompactString>,
        start: (SmallVec<[IndexPage; 1]>, String),
        end: (SmallVec<[IndexPage; 1]>, String),
    },
}

impl MergedPage {
    pub fn page_command(&self) -> Option<&str> {
        match self {
            MergedPage::Single { page_command, .. } => page_command.as_deref(),
            MergedPage::Range { page_command, .. } => page_command.as_deref(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MergedPages(Vec<MergedPage>);

impl Deref for MergedPages {
    type Target = [MergedPage];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct Record {
    cmd: Option<CompactString>,
    page_tuple: (SmallVec<[IndexPage; 1]>, String),
    timestamp: usize,
}

impl Record {
    fn to_raw(&self, kind: RangeKind, style: &IstInputStyle) -> String {
        format!(
            "{}..{}{}{}{}{}{}{}",
            style.arg_open,
            style.encap,
            match kind {
                RangeKind::None => CompactString::new(""),
                RangeKind::Open => style.range_open.to_compact_string(),
                RangeKind::Close => style.range_close.to_compact_string(),
            },
            match &self.cmd {
                Some(cmd) => cmd.as_str(),
                None => "",
            },
            style.arg_close,
            style.arg_open,
            &self.page_tuple.1,
            style.arg_close,
        )
    }
}

enum ItemKind {
    Single((SmallVec<[IndexPage; 1]>, String)),
    Range(
        (SmallVec<[IndexPage; 1]>, String),
        (SmallVec<[IndexPage; 1]>, String),
    ),
}

struct PendingItem {
    cmd: Option<CompactString>,
    kind: ItemKind,
    timestamp: usize,
}

impl From<Record> for PendingItem {
    fn from(value: Record) -> Self {
        PendingItem {
            cmd: value.cmd,
            kind: ItemKind::Single(value.page_tuple),
            timestamp: value.timestamp,
        }
    }
}

impl PendingItem {
    #[inline]
    fn is_single(&self) -> bool {
        matches!(self.kind, ItemKind::Single(_))
    }
    #[inline]
    fn is_range(&self) -> bool {
        matches!(self.kind, ItemKind::Range(_, _))
    }

    #[inline]
    fn start_page(&self) -> &[IndexPage] {
        match &self.kind {
            ItemKind::Single((p, _)) => p,
            ItemKind::Range((s, _), _) => s,
        }
    }

    #[inline]
    fn end_page(&self) -> &[IndexPage] {
        match &self.kind {
            ItemKind::Single((p, _)) => p,
            ItemKind::Range(_, (e, _)) => e,
        }
    }
}

pub struct MergedPagesBuilder<'s, F: Fn(&[IndexPage], &[IndexPage]) -> Ordering> {
    page_cmp: F,
    auto_merge: bool,
    strict: bool,

    input_style: &'s IstInputStyle,
    #[allow(dead_code)]
    output_style: &'s IstOutputStyle,

    open_stack: Vec<Record>,
    items: Vec<PendingItem>,
    reduced: Vec<PendingItem>,
    final_items: Vec<PendingItem>,
}

impl<'s, F: Fn(&[IndexPage], &[IndexPage]) -> Ordering> MergedPagesBuilder<'s, F> {
    pub fn new(
        page_cmp: F,
        auto_merge: bool,
        strict: bool,
        style: (&'s IstInputStyle, &'s IstOutputStyle),
    ) -> Self {
        MergedPagesBuilder {
            page_cmp,
            auto_merge,
            strict,
            input_style: style.0,
            output_style: style.1,
            open_stack: Vec::new(),
            items: Vec::new(),
            reduced: Vec::new(),
            final_items: Vec::new(),
        }
    }

    pub fn build_from_vec(
        &mut self,
        p: Vec<(
            Option<CompactString>,
            RangeKind,
            String,
            SmallVec<[IndexPage; 1]>,
        )>,
    ) -> MergedPages {
        fn is_adjacent(
            page_cmp: impl Fn(&[IndexPage], &[IndexPage]) -> Ordering,
            prev_end: &[IndexPage],
            curr_start: &[IndexPage],
        ) -> bool {
            let cmp = page_cmp(prev_end, curr_start);
            if cmp == Ordering::LessSame {
                if let (Some(pe), Some(cs)) = (prev_end.last(), curr_start.last()) {
                    return pe.as_u64() + 1 == cs.as_u64();
                }
            }
            false
        }

        self.open_stack.clear();
        self.items.clear();
        self.reduced.clear();
        self.final_items.clear();

        let page_cmp = &self.page_cmp;

        for (timestamp, (cmd, kind, raw, pages)) in p.into_iter().enumerate() {
            let page_tuple = (pages, raw);
            match kind {
                RangeKind::None => {
                    self.items.push(PendingItem {
                        cmd,
                        kind: ItemKind::Single(page_tuple),
                        timestamp,
                    });
                }
                RangeKind::Open => {
                    self.open_stack.push(Record {
                        cmd,
                        page_tuple,
                        timestamp,
                    });
                }
                RangeKind::Close => {
                    let matched_idx = if self.strict {
                        self.open_stack.iter().rposition(|o| o.cmd == cmd)
                    } else {
                        if let Some(idx) = self.open_stack.iter().rposition(|o| o.cmd == cmd) {
                            Some(idx)
                        } else if !self.open_stack.is_empty() {
                            Some(self.open_stack.len() - 1)
                        } else {
                            None
                        }
                    };

                    if let Some(idx) = matched_idx {
                        let open = self.open_stack.remove(idx);
                        self.items.push(PendingItem {
                            cmd: open.cmd,
                            kind: ItemKind::Range(open.page_tuple, page_tuple),
                            timestamp: open.timestamp,
                        });
                    } else {
                        let record = Record {
                            cmd,
                            page_tuple,
                            timestamp,
                        };
                        log::error!(target: "cindex",
                            "Unmatched Close with command: {:?}.",
                            record.to_raw(RangeKind::Close, self.input_style)
                        );
                        self.items.push(record.into());
                    }
                }
            }
        }

        for open in self.open_stack.drain(..) {
            log::error!(target: "cindex",
                "Unmatched Open in {}.",
                open.to_raw(RangeKind::Open, self.input_style)
            );
            self.items.push(open.into());
        }

        self.items.sort_by(|a, b| {
            let cmd_cmp = a.cmd.cmp(&b.cmd);
            if cmd_cmp != StdOrdering::Equal {
                return cmd_cmp;
            }

            let cmp = (self.page_cmp)(a.start_page(), b.start_page());
            let std_cmp = match cmp {
                Ordering::LessOther | Ordering::LessSame => StdOrdering::Less,
                Ordering::Equal => StdOrdering::Equal,
                Ordering::GreaterSame | Ordering::GreaterOther => StdOrdering::Greater,
            };

            if std_cmp == StdOrdering::Equal {
                a.timestamp.cmp(&b.timestamp)
            } else {
                std_cmp
            }
        });

        let mut items_iter = self.items.drain(..).peekable();
        while let Some(item) = items_iter.next() {
            self.reduced.push(item);

            loop {
                if self.reduced.len() < 2 {
                    break;
                }
                let len = self.reduced.len();
                let prev = &self.reduced[len - 2];
                let last = &self.reduced[len - 1];

                let cmp_start = (self.page_cmp)(prev.end_page(), last.start_page());
                let is_overlap = matches!(
                    cmp_start,
                    Ordering::Equal | Ordering::GreaterSame | Ordering::GreaterOther
                );

                let is_adj =
                    !is_overlap && is_adjacent(page_cmp, prev.end_page(), last.start_page());

                // 紧邻合并：仅在至少一方是 Range 且允许合并时贪心吞噬；纯 Single 稍后统一判断长度
                let should_merge = is_overlap
                    || (self.auto_merge && is_adj && (prev.is_range() || last.is_range()));

                if should_merge {
                    let last_item = self.reduced.pop().unwrap();
                    let prev_item = self.reduced.pop().unwrap();

                    let cmp_ends = (self.page_cmp)(prev_item.end_page(), last_item.end_page());
                    let is_same_end = cmp_ends == Ordering::Equal;

                    if is_overlap && prev_item.is_single() && last_item.is_single() && is_same_end {
                        // 两者为完全相同的 Single，保留先进入时间的那个，直接丢弃后者去重
                        self.reduced.push(prev_item);
                    } else {
                        let use_last_end =
                            matches!(cmp_ends, Ordering::LessSame | Ordering::LessOther);

                        if use_last_end {
                            // Start 取 prev，End 取 last
                            let start_tuple = match prev_item.kind {
                                ItemKind::Single(p) => p,
                                ItemKind::Range(s, _) => s,
                            };
                            let end_tuple = match last_item.kind {
                                ItemKind::Single(p) => p,
                                ItemKind::Range(_, e) => e,
                            };
                            self.reduced.push(PendingItem {
                                cmd: prev_item.cmd,
                                kind: ItemKind::Range(start_tuple, end_tuple),
                                timestamp: prev_item.timestamp,
                            });
                        } else {
                            // prev 完全包裹了 last（其 end 更大），我们仅需要保留 prev，完全丢弃 last
                            self.reduced.push(prev_item);
                        }
                    }
                } else {
                    break;
                }
            }

            // 当前块（同一个 command）结束，清理连续的纯 Single 并打包为 Range
            let next_cmd_differs = match items_iter.peek() {
                Some(next_item) => next_item.cmd != self.reduced[0].cmd,
                None => true,
            };

            if next_cmd_differs {
                let mut i = 0;
                while i < self.reduced.len() {
                    if !self.reduced[i].is_single() {
                        i += 1;
                        continue;
                    }

                    let mut j = i + 1;
                    while j < self.reduced.len() && self.reduced[j].is_single() {
                        if is_adjacent(
                            page_cmp,
                            self.reduced[j - 1].end_page(),
                            self.reduced[j].start_page(),
                        ) {
                            j += 1;
                        } else {
                            break;
                        }
                    }

                    let run_len = j - i;
                    if self.auto_merge && run_len >= 3 {
                        // drain 利用所有权提取目标序列，中间的 Single 完全被释放
                        let mut segment: Vec<_> = self.reduced.drain(i..j).collect();
                        let first_item = segment.remove(0);
                        let last_item = segment.pop().unwrap();

                        let start_tuple = match first_item.kind {
                            ItemKind::Single(p) => p,
                            _ => unreachable!(),
                        };
                        let end_tuple = match last_item.kind {
                            ItemKind::Single(p) => p,
                            _ => unreachable!(),
                        };

                        self.reduced.insert(
                            i,
                            PendingItem {
                                cmd: first_item.cmd,
                                kind: ItemKind::Range(start_tuple, end_tuple),
                                timestamp: first_item.timestamp,
                            },
                        );
                        i += 1;
                    } else {
                        i = j;
                    }
                }

                // 将处理好的此 group 平移至结果数组
                self.final_items.append(&mut self.reduced);
            }
        }

        self.final_items.sort_by(|a, b| {
            let cmp = (self.page_cmp)(a.start_page(), b.start_page());
            let std_cmp = match cmp {
                Ordering::LessOther | Ordering::LessSame => StdOrdering::Less,
                Ordering::Equal => StdOrdering::Equal,
                Ordering::GreaterSame | Ordering::GreaterOther => StdOrdering::Greater,
            };

            if std_cmp == StdOrdering::Equal {
                a.timestamp.cmp(&b.timestamp)
            } else {
                std_cmp
            }
        });

        let result = self
            .final_items
            .drain(..)
            .map(|item| {
                let page_command = item.cmd;
                match item.kind {
                    ItemKind::Single(page) => MergedPage::Single { page_command, page },
                    ItemKind::Range(start, end) => MergedPage::Range {
                        page_command,
                        start,
                        end,
                    },
                }
            })
            .collect();

        MergedPages(result)
    }
}

pub fn merge_entries(
    from: Vec<IndexEntry>,
    style: (&IstInputStyle, &IstOutputStyle),
    page_cmp: impl Fn(&[IndexPage], &[IndexPage]) -> Ordering,
    auto_merge: bool,
    strict: bool,
) -> Vec<MergedEntry> {
    struct LevelsAsKey(SmallVec<[(Option<String>, String); 1]>);
    impl std::hash::Hash for LevelsAsKey {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            for k in &self.0 {
                u8::MAX.hash(state);
                k.0.as_ref().unwrap_or(&k.1).hash(state);
            }
        }
    }
    impl PartialEq for LevelsAsKey {
        fn eq(&self, other: &Self) -> bool {
            if self.0.len() == other.0.len() {
                for (l, r) in self.0.iter().zip(&other.0) {
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
                true
            } else {
                false
            }
        }
    }
    impl Eq for LevelsAsKey {}

    let mut entries: IndexMap<LevelsAsKey, Vec<_>> = IndexMap::default();

    for entry in from {
        let pages = entries.entry(LevelsAsKey(entry.levels)).or_default();
        pages.push((
            entry.page_commands,
            entry.range,
            entry.pages_raw,
            entry.pages,
        ));
    }

    let mut builder = MergedPagesBuilder::new(page_cmp, auto_merge, strict, style);

    entries
        .into_iter()
        .map(|e| MergedEntry {
            levels: e.0.0,
            pages: builder.build_from_vec(e.1),
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ordering {
    LessOther,
    LessSame,
    Equal,
    GreaterSame,
    GreaterOther,
}

pub fn get_page_cmp(prec_table: Precedence) -> Box<dyn Fn(&[IndexPage], &[IndexPage]) -> Ordering> {
    let Precedence(prec_table) = prec_table;
    let page_into = move |page: &IndexPage| -> (u8, u64) {
        match page.kind {
            IndexPageKind::roman => (prec_table[0], page.value),
            IndexPageKind::Arabic => (prec_table[1], page.value),
            IndexPageKind::alpha => (prec_table[2], page.value),
            IndexPageKind::Roman => (prec_table[3], page.value),
            IndexPageKind::Alpha => (prec_table[4], page.value),
            IndexPageKind::ZhDigits => (prec_table[5], page.value),
            IndexPageKind::ZhNumber => (prec_table[6], page.value),
        }
    };

    Box::new(move |lhs, rhs| {
        match (lhs.is_empty(), rhs.is_empty()) {
            (true, true) => return Ordering::Equal,
            (true, false) => return Ordering::LessOther,
            (false, true) => return Ordering::GreaterOther,
            (false, false) => {}
        }

        for (l, r) in lhs[..lhs.len() - 1].iter().zip(&rhs[..rhs.len() - 1]) {
            let l = page_into(l);
            let r = page_into(r);
            let cmp = if l.0 == r.0 {
                l.1.cmp(&r.1)
            } else {
                l.0.cmp(&r.0)
            };
            match cmp {
                StdOrdering::Less => return Ordering::LessOther,
                StdOrdering::Equal => {}
                StdOrdering::Greater => return Ordering::GreaterOther,
            }
        }
        match lhs.len().cmp(&rhs.len()) {
            StdOrdering::Less => Ordering::LessOther,
            StdOrdering::Equal => {
                let l = page_into(&lhs.last().unwrap());
                let r = page_into(&rhs.last().unwrap());
                match l.cmp(&r) {
                    StdOrdering::Less if l.0 == r.0 => Ordering::LessSame,
                    StdOrdering::Less => Ordering::LessOther,
                    StdOrdering::Equal => Ordering::Equal,
                    StdOrdering::Greater if l.0 == r.0 => Ordering::GreaterSame,
                    StdOrdering::Greater => Ordering::GreaterOther,
                }
            }
            StdOrdering::Greater => Ordering::GreaterOther,
        }
    })
}

pub fn luaopen_cindex_table(lua: &Lua, _: ()) -> LuaResult<LuaTable> {
    let cindex_table = lua.create_table()?;

    let bihua_to_group = lua.create_table_with_capacity(64, 0)?;

    // nil + Symbols + Numbers + A..=Z + 1..=64画 + 1（一）..=214（龠）部
    let len = 1 + 1 + 1 + 26 + 6 + 214;
    let group_order = lua.create_table_with_capacity(len - 1, len + 1)?;
    group_order.raw_set("nil", 0)?;
    group_order.raw_set(0, "nil")?;
    group_order.raw_set("Symbols", 1)?;
    group_order.raw_set(1, "Symbols")?;
    group_order.raw_set("Numbers", 2)?;
    group_order.raw_set(2, "Numbers")?;
    let mut idx = 2;
    let mut s = compact_str::CompactString::new("");
    for c in 'A'..='Z' {
        s.clear();
        s.push(c);
        idx += 1;
        group_order.raw_set(s.as_str(), idx)?;
        group_order.raw_set(idx, s.as_str())?;
    }
    let mut buf = itoa::Buffer::new();
    for n in 1..=64 {
        s.clear();
        s.push_str("BiHua");
        s.push_str(buf.format(n));
        idx += 1;
        group_order.raw_set(s.as_str(), idx)?;
        group_order.raw_set(idx, s.as_str())?;

        bihua_to_group.raw_set(n, s.as_str())?;
    }
    for n in 1..=han::MAX_RADICAL.0 {
        s.clear();
        s.push_str("BuShou");
        s.push_str(buf.format(n));
        idx += 1;
        group_order.raw_set(s.as_str(), idx)?;
        group_order.raw_set(idx, s.as_str())?;
    }
    cindex_table.raw_set("GroupOrder", group_order)?;
    cindex_table.raw_set("BiHuaToGroup", bihua_to_group)?;

    let kx_to_group = lua.create_table()?;
    for n in 0..han::MAX_RADICAL.1 {
        let rad: han::Radical = unsafe { std::mem::transmute(n) };
        s.clear();
        s.push_str("BuShou");
        s.push_str(rad.original().str_repr());
        match han::kangxi_ideograph(rad) {
            (Some(kx), ideo) => {
                kx_to_group.raw_set(kx as u32, s.as_str())?;
                kx_to_group.raw_set(ideo as u32, s.as_str())?;
            }
            (None, ideo) => {
                kx_to_group.raw_set(ideo as u32, s.as_str())?;
            }
        }
    }
    cindex_table.raw_set("BuShouToGroup", kx_to_group)?;

    let len = han::MAX_RADICAL.0 as usize;
    let kx_to_chars = lua.create_table_with_capacity(len, len)?;
    for n in 0..han::MAX_RADICAL.0 {
        s.clear();
        s.push_str("BuShou");
        s.push_str(buf.format(n));

        let char_table = lua.create_table()?;
        for inner_n in 0..han::MAX_RADICAL.1 {
            let inner_rad: han::Radical = unsafe { std::mem::transmute(inner_n) };
            if buf.format(n) == inner_rad.original().str_repr() {
                char_table.raw_push(han::kangxi_ideograph(inner_rad).1 as u32)?;
            }
        }
        kx_to_chars.raw_set(s.as_str(), char_table.clone())?;
        kx_to_chars.raw_push(char_table)?;
    }
    cindex_table.raw_set("BuShouToChars", kx_to_chars)?;

    Ok(cindex_table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PagePrecedenceProvider, StringPrecedence};

    fn get_prec_table(s: &str) -> [u8; 7] {
        StringPrecedence(s).to_precedence().0
    }

    #[test]
    fn prec() {
        assert_eq!(get_prec_table("rnaRA"), [0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(get_prec_table("rnaRA串数"), [0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(get_prec_table("RanAr"), [4, 2, 1, 0, 3, 5, 6]);
        assert_eq!(get_prec_table("数Ran串Ar"), [6, 3, 2, 1, 5, 4, 0]);
    }

    #[test]
    fn merge() {
        let _page_cmp = get_page_cmp("rnaRA".to_precedence());
    }
}
