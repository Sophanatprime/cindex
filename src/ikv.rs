use std::str::FromStr;

use anyhow::{Context, Result, anyhow, bail};
use compact_str::CompactString;
use smallvec::SmallVec;

use crate::{IndexPage, style::*};

pub fn from_ikv<T: AsRef<str>>(
    lines: &mut impl Iterator<Item = T>,
    style: &IstInputStyle,
) -> Result<Option<IndexEntry>> {
    let Some(line) = lines.next() else {
        return Ok(None);
    };

    let line = line.as_ref().trim_ascii_start();
    // 我们允许原始的 \indexentry .. 这种形式存在，并且优先使用它。
    if line.starts_with(&style.keyword) {
        return IndexEntry::parse_index_line(style, line);
    }

    let mut line_part = line.split(&style.separator);

    let page_raw = line_part
        .next()
        .with_context(|| anyhow!("missing page in {line}"))?;
    let mut pages = SmallVec::<[IndexPage; 1]>::new();
    for p in page_raw.split(&style.page_compositor) {
        pages.push(IndexPage::from_str(p)?);
    }

    let commands = line_part
        .next()
        .with_context(|| anyhow!("missing range and commands in {line}"))?;
    let (range, page_commands) = if commands.starts_with(style.range_open) {
        (
            RangeKind::Open,
            commands
                .len()
                .gt(&1)
                .then_some(CompactString::new(&commands[1..])),
        )
    } else if commands.starts_with(style.range_close) {
        (
            RangeKind::Close,
            commands
                .len()
                .gt(&1)
                .then_some(CompactString::new(&commands[1..])),
        )
    } else {
        (
            RangeKind::None,
            commands
                .len()
                .gt(&0)
                .then_some(CompactString::new(commands)),
        )
    };

    let length = line_part
        .next()
        .with_context(|| anyhow!("missing length of levels in {line}"))?;
    let length = usize::from_str_radix(length, 10)?;
    let mut levels = SmallVec::<[(Option<String>, String); 1]>::new();

    if line_part.next().is_some() {
        bail!("extra string after length of levels in {line}");
    }

    for i in 0..length {
        let key = lines
            .next()
            .with_context(|| anyhow!("missing the sort key {}", i + 1))?;
        let val = lines
            .next()
            .with_context(|| anyhow!("missing the display {}", i + 1))?;

        let key = key.as_ref().trim_ascii_start();
        let val = val.as_ref().trim_ascii_start();
        if val.is_empty() {
            bail!("empty display");
        }
        if key.is_empty() || key == val {
            levels.push((None, val.into()));
        } else {
            levels.push((Some(key.into()), val.into()));
        }
    }

    Ok(Some(IndexEntry {
        levels,
        range,
        page_commands,
        pages,
        pages_raw: page_raw.into(),
    }))
}
