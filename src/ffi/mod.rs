use anyhow::Result;
use std::ffi::{OsStr, OsString};

pub use mlua as lua;
use mlua::prelude::*;
mod exports;
pub use exports::{Api, StrRef, Writer};
mod convert;
pub use pcre2_sys;

pub fn make_lua<S: AsRef<str>>(
    user_script: Option<S>,
    f: impl Fn(&Lua) -> Result<()>,
) -> Result<Lua> {
    let lua = unsafe { Lua::unsafe_new() };
    LuaModule::Lpeg.preload(&lua)?;
    LuaModule::IcuTable.preload(&lua)?;
    LuaModule::Jits.preload(&lua)?;
    f(&lua)?;
    let func = lua.load(include_str!("cindex.lua")).into_function()?;
    match user_script {
        Some(p) => func.call::<()>(p.as_ref())?,
        None => func.call::<()>(LuaNil)?,
    }
    Ok(lua)
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LuaModule {
    Lpeg,
    IcuTable,
    Jits,
}

impl LuaModule {
    pub fn preload(&self, lua: &Lua) -> Result<()> {
        unsafe {
            match self {
                LuaModule::Lpeg => lua.preload_module(
                    "lpeg",
                    lua.create_c_function(std::mem::transmute(
                        luajit_modules::luaopen_lpeg as *const (),
                    ))?,
                )?,
                LuaModule::IcuTable => {
                    lua.preload_module("icu.table", lua.create_function(luaopen_icu_table)?)?
                }
                LuaModule::Jits => {
                    for (name, func) in luajit_modules::JIT_MODULES {
                        lua.preload_module(
                            *name,
                            lua.create_c_function(std::mem::transmute(*func))?,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn kpse_find_file(file: impl AsRef<OsStr>, all: bool) -> Result<Option<OsString>> {
    let which = std::env::var("KPSEWHICH_EXE_FILE").unwrap_or(String::from("kpsewhich"));
    let mut cmd = std::process::Command::new(&which);
    if all {
        cmd.arg("--all");
    }
    cmd.arg(file);
    let res = cmd.output()?;
    if res.status.success() {
        let mut bytes = res.stdout;
        bytes.truncate(bytes.trim_ascii_end().len());
        if bytes.is_empty() {
            return Ok(None);
        } else {
            let file = unsafe { OsString::from_encoded_bytes_unchecked(bytes) };
            Ok(Some(file))
        }
    } else {
        let stderr = unsafe { OsString::from_encoded_bytes_unchecked(res.stderr) };
        anyhow::bail!("Error: {}", stderr.display());
    }
}

pub fn kpse_in_name_ok(file: impl AsRef<OsStr>) -> bool {
    let which = std::env::var("KPSEWHICH_EXE_FILE").unwrap_or(String::from("kpsewhich"));
    let mut cmd = std::process::Command::new(&which);
    cmd.arg("--safe-in-name")
        .arg(file.as_ref())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn kpse_out_name_ok(file: impl AsRef<OsStr>) -> bool {
    let which = std::env::var("KPSEWHICH_EXE_FILE").unwrap_or(String::from("kpsewhich"));
    let mut cmd = std::process::Command::new(&which);
    cmd.arg("--safe-out-name")
        .arg(file.as_ref())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn luaopen_icu_table(lua: &Lua, _: ()) -> LuaResult<LuaTable> {
    use icu_collections::codepointtrie::TrieValue;
    use icu_properties::props::{GeneralCategory, NamedEnumeratedProperty, Script};

    let icu = lua.create_table()?;
    let name_table = lua.create_table()?;

    let icu_script = lua.create_table()?;
    #[allow(deprecated)]
    for script in Script::ALL_VALUES {
        icu_script.raw_set(script.long_name(), script.to_u32())?;
        icu_script.raw_set(script.to_u32(), script.long_name())?;
        name_table.raw_set(script.long_name(), script.short_name())?;
    }
    icu.raw_set("Script", icu_script)?;

    let icu_gc = lua.create_table()?;
    #[allow(deprecated)]
    for cat in GeneralCategory::ALL_VALUES {
        icu_gc.raw_set(cat.long_name(), *cat as u8)?;
        icu_gc.raw_set(*cat as u8, cat.long_name())?;
        name_table.raw_set(cat.long_name(), cat.short_name())?;
    }
    icu.raw_set("GeneralCategory", icu_gc)?;

    let emj_group = lua.create_table()?;
    for group in emojis::Group::iter() {
        let name = match group {
            emojis::Group::Activities => "Activities",
            emojis::Group::AnimalsAndNature => "Animals And Nature",
            emojis::Group::Flags => "Flags",
            emojis::Group::FoodAndDrink => "Food And Drink",
            emojis::Group::Objects => "Objects",
            emojis::Group::PeopleAndBody => "People And Body",
            emojis::Group::SmileysAndEmotion => "Smileys And Emotion",
            emojis::Group::Symbols => "Symbols",
            emojis::Group::TravelAndPlaces => "Travel And Places",
        };
        emj_group.raw_set(name, group as u8)?;
        emj_group.raw_set(group as u8, name)?;
    }
    icu.raw_set("EmojiGroup", emj_group)?;

    icu.raw_set("PropertyShortNames", name_table)?;
    Ok(icu)
}
