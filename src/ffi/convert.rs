use compact_str::ToCompactString;
use smallvec::SmallVec;

use crate::page::IndexPage;
use crate::style::{IndexEntry, RangeKind};
use crate::{MergedEntry, MergedPage, ffi::*};

impl FromLua for IndexEntry {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        let type_name = value.type_name();
        let err_fn = |s| {
            Err(LuaError::FromLuaConversionError {
                from: type_name,
                to: "cindex::IndexEntry".to_string(),
                message: Some(s),
            })
        };

        let LuaValue::Table(table) = value else {
            return err_fn(String::from("expect a table"));
        };

        let mut levels_val = LuaValue::Nil;
        let mut range_val = LuaValue::Nil;
        let mut page_commands = LuaValue::Nil;
        let mut pages_val = LuaValue::Nil;
        let mut pages_raw_val = LuaValue::Nil;
        for pair in table.pairs::<LuaValue, LuaValue>() {
            let (k, v) = pair?;
            if let LuaValue::String(s) = k {
                match s.to_str()?.as_ref() {
                    "levels" => levels_val = v,
                    "range" => range_val = v,
                    "page_commands" | "page-commands" => page_commands = v,
                    "pages" => pages_val = v,
                    "pages_raw" | "pages-raw" => pages_raw_val = v,
                    s => return err_fn(format!("unknown key '{}'", s)),
                }
            } else {
                return err_fn(format!("unknown key '{:?}'", k));
            }
        }

        let mut levels = SmallVec::new();
        if let LuaValue::Table(levels_tbl) = levels_val {
            let len = levels_tbl.raw_len() as usize;
            if len > 0 {
                levels.reserve(len);
                for i in 1..=len {
                    let tup: LuaTable = levels_tbl.get(i)?;
                    let tup_len = tup.raw_len();
                    if tup_len == 1 || tup_len == 2 {
                        if tup_len == 1 {
                            levels.push((None, tup.get(1)?));
                        } else if tup_len >= 2 {
                            levels.push((Some(tup.get(1)?), tup.get(2)?));
                        }
                    } else {
                        return err_fn(format!(
                            "length of tables must be 1 or 2 in levels, got: {}",
                            tup_len
                        ));
                    };
                }
            }
        } else {
            return err_fn(format!("expect table for levels"));
        }

        let range = if range_val == LuaNil {
            RangeKind::None
        } else {
            let mut s = String::from_lua(range_val, lua)?;
            s.make_ascii_lowercase();
            match s.as_str() {
                "open" | "-1" => RangeKind::Open,
                "none" | "0" => RangeKind::None,
                "close" | "1" => RangeKind::Close,
                _ => {
                    return err_fn(format!(
                        "invalid value for range: '{}', available value: [Open,None,Close]",
                        s
                    ));
                }
            }
        };

        let page_commands = if page_commands == LuaNil {
            None
        } else if let LuaValue::String(s) = page_commands {
            Some(s.to_string_lossy().to_compact_string())
        } else {
            return err_fn(format!("expect string for page_commands"));
        };

        let mut pages = SmallVec::new();
        if let LuaValue::Table(table) = pages_val {
            let len = table.raw_len();
            if len > 0 {
                pages.reserve(len);
                for tup in table.sequence_values::<LuaTable>() {
                    let mut kind = None;
                    let mut value = None;

                    for pair in tup?.pairs::<LuaValue, LuaValue>() {
                        let (k, v) = pair?;
                        if let LuaValue::String(s) = k {
                            match s.to_str()?.as_ref() {
                                "kind" => {
                                    if kind.is_some() {
                                        return err_fn(format!(
                                            "duplicated field 'kind' of a page"
                                        ));
                                    }
                                    if let LuaValue::String(kind_val) = v {
                                        kind = Some(kind_val);
                                    } else {
                                        return err_fn(format!(
                                            "expect string for field 'kind' of a page"
                                        ));
                                    }
                                }
                                "value" => {
                                    if value.is_some() {
                                        return err_fn(format!(
                                            "duplicated field 'value' of a page"
                                        ));
                                    }
                                    if let LuaValue::Integer(value_val) = v {
                                        value = Some(value_val as u64);
                                    } else if let LuaValue::Number(value_val) = v {
                                        value = Some(value_val as u64);
                                    } else {
                                        return err_fn(format!(
                                            "expect integer for field 'value' of a page"
                                        ));
                                    }
                                }
                                _ => return err_fn(format!("unknown field for a page")),
                            }
                        }
                    }

                    match (kind, value) {
                        (Some(kind), Some(num)) => match kind.to_str()?.as_ref() {
                            "Arabic" => pages.push(IndexPage::Arabic(num)),
                            "Alpha" => pages.push(IndexPage::Alpha(num)),
                            "alpha" => pages.push(IndexPage::alpha(num)),
                            "roman" => pages.push(IndexPage::roman(num)),
                            "Roman" => pages.push(IndexPage::Roman(num)),
                            "ZhDigits" => pages.push(IndexPage::ZhDigits(num)),
                            "ZhNumber" => pages.push(IndexPage::ZhNumber(num)),
                            s => return err_fn(format!("unknown kind of a page: '{}', ", s)),
                        },
                        (None, Some(_)) => {
                            return err_fn(format!("missing field 'kind' of a page"));
                        }
                        (Some(_), None) => {
                            return err_fn(format!("missing field 'value' of a page"));
                        }
                        (None, None) => {
                            return err_fn(format!("missing fields 'kind' and 'value' of a page"));
                        }
                    }
                }
            }
        } else {
            return err_fn(format!("expect table for pages"));
        };

        let pages_raw = if let LuaValue::String(s) = pages_raw_val {
            s.to_string_lossy()
        } else if LuaNil == pages_raw_val {
            String::new()
        } else {
            return err_fn(format!("expect string for pages_raw"));
        };

        Ok(IndexEntry {
            levels,
            range,
            page_commands,
            pages,
            pages_raw,
        })
    }
}

impl IntoLua for &IndexEntry {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table_with_capacity(0, 4)?;

        if !self.levels.is_empty() {
            let levels_tbl = lua.create_table_with_capacity(self.levels.len(), 0)?;

            for (i, (key_opt, text)) in self.levels.iter().enumerate() {
                let tup = if let Some(key) = key_opt {
                    let t = lua.create_table_with_capacity(2, 0)?;
                    t.raw_set(1, key.as_str())?;
                    t.raw_set(2, text.as_str())?;
                    t
                } else {
                    let t = lua.create_table_with_capacity(1, 0)?;
                    t.raw_set(1, text.as_str())?;
                    t
                };
                levels_tbl.raw_set(i + 1, tup)?;
            }
            table.raw_set("levels", levels_tbl)?;
        }

        if self.range != RangeKind::None {
            table.raw_set(
                "range",
                match self.range {
                    RangeKind::Open => "Open",
                    RangeKind::None => "None",
                    RangeKind::Close => "Close",
                },
            )?;
        }

        if let Some(cmd) = &self.page_commands {
            table.raw_set("page_commands", cmd.as_str())?;
        }

        if !self.pages.is_empty() {
            let pages_tbl = lua.create_table_with_capacity(self.pages.len(), 0)?;

            for page in self.pages.iter() {
                let t = lua.create_table_with_capacity(0, 2)?;

                let (k, v) = page.as_kind();
                t.raw_set("kind", k)?;
                t.raw_set("value", v)?;

                pages_tbl.raw_push(t)?;
            }
            table.raw_set("pages", pages_tbl)?;
        }

        if !self.pages_raw.is_empty() {
            table.raw_set("pages_raw", self.pages_raw.as_str())?;
        }

        Ok(LuaValue::Table(table))
    }
}

impl IntoLua for &MergedEntry {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table_with_capacity(0, 2)?;

        if !self.levels.is_empty() {
            let levels = lua.create_table_with_capacity(self.levels.len(), 0)?;
            for (key_opt, text) in &self.levels {
                let t = lua.create_table()?;
                match key_opt {
                    Some(key) => {
                        t.raw_set(1, key.as_str())?;
                        t.raw_set(2, text.as_str())?;
                    }
                    None => {
                        t.raw_set(1, text.as_str())?;
                    }
                }
            }
            table.raw_set("levels", levels)?;
        }

        if !self.pages.is_empty() {
            let pages = lua.create_table_with_capacity(self.pages.len(), 0)?;
            for page in self.pages.iter() {
                let t = lua.create_table()?;
                match page {
                    MergedPage::Single { page_command, page } => {
                        t.raw_set("kind", "single")?;
                        t.raw_set(
                            "page_command",
                            page_command.as_ref().map_or(LuaNil, |s| {
                                LuaValue::String(
                                    lua.create_string(s).expect("unable to create string"),
                                )
                            }),
                        )?;
                        let pg = lua.create_table_with_capacity(page.0.len(), 0)?;
                        for p in page.0.iter() {
                            let pg_item = lua.create_table()?;
                            let (k, v) = p.as_kind();
                            pg_item.raw_set("kind", k)?;
                            pg_item.raw_set("value", v)?;
                            pg.raw_push(pg_item)?;
                        }
                        t.raw_set("pages", pg)?;
                        t.raw_set("pages_raw", page.1.as_str())?;
                    }
                    MergedPage::Range {
                        page_command,
                        start,
                        end,
                    } => {
                        t.raw_set("kind", "single")?;
                        t.raw_set(
                            "page_command",
                            page_command.as_ref().map_or(LuaNil, |s| {
                                LuaValue::String(
                                    lua.create_string(s).expect("unable to create string"),
                                )
                            }),
                        )?;
                        let st = lua.create_table_with_capacity(start.0.len(), 0)?;
                        for p in start.0.iter() {
                            let pg_item = lua.create_table()?;
                            let (k, v) = p.as_kind();
                            pg_item.raw_set("kind", k)?;
                            pg_item.raw_set("value", v)?;
                            st.raw_push(pg_item)?;
                        }
                        t.raw_set("start", st)?;
                        t.raw_set("start_raw", start.1.as_str())?;
                        let ed = lua.create_table_with_capacity(end.0.len(), 0)?;
                        for p in end.0.iter() {
                            let pg_item = lua.create_table()?;
                            let (k, v) = p.as_kind();
                            pg_item.raw_set("kind", k)?;
                            pg_item.raw_set("value", v)?;
                            ed.raw_push(pg_item)?;
                        }
                        t.raw_set("end", ed)?;
                        t.raw_set("end_raw", end.1.as_str())?;
                    }
                }
            }
            table.raw_set("pages", pages)?;
        }

        Ok(LuaValue::Table(table))
    }
}
