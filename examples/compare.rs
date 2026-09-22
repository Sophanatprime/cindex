use mlua::prelude::*;
use xript_runtime::{ConsoleHandler, HostBindings, RuntimeOptions, create_runtime};

use cindex::ffi::LuaModule as AllowedModule;

fn main() -> anyhow::Result<()> {
    let manifest_json = r#"{
        "xript": "0.7",
        "name": "my-app",
        "bindings": {
            "greet": {
                "description": "Returns a greeting.",
                "params": [{ "name": "name", "type": "string" }]
            }
        }
    }"#;

    let mut bindings = HostBindings::new();
    bindings.add_function("greet", |args: &[serde_json::Value]| {
        let name = args.first().and_then(|v| v.as_str()).unwrap_or("World");
        Ok(serde_json::json!(format!("Hello, {}!", name)))
    });

    let runtime = create_runtime(
        manifest_json,
        RuntimeOptions {
            host_bindings: bindings,
            capabilities: vec![],
            console: ConsoleHandler::default(),
            ..Default::default()
        },
    )?;

    let result = runtime.execute(r#"greet("Rust")"#)?;
    println!("js: {}", result.value);

    let lua = Lua::new_with(
        LuaStdLib::BIT
            | LuaStdLib::MATH
            | LuaStdLib::STRING
            | LuaStdLib::TABLE
            | LuaStdLib::PACKAGE,
        LuaOptions::default(),
    )?;
    {
        AllowedModule::Lpeg.preload(&lua)?;
        // lua.preload_module(
        //     "re",
        //     lua.create_function(|lua, ()| {
        //         lua.load(include_str!(concat!(
        //             env!("CARGO_MANIFEST_DIR"),
        //             "/../luajit/lpeg-1.1.0/re.lua"
        //         )))
        //         .set_name("re.lua")
        //         .eval::<LuaValue>()
        //     })?,
        // )?;

        lua.globals().raw_set("dofile", LuaNil)?;
        lua.globals().raw_set("loadfile", LuaNil)?;

        let package: LuaTable = lua.globals().get("package")?;
        package.raw_set("path", LuaNil)?;
        package.raw_set("cpath", LuaNil)?;
        package.raw_set("loadlib", LuaNil)?;
        let searchers: LuaTable = package.get("loaders")?;
        let preload = searchers.get::<LuaFunction>(1)?;
        let new_searchers = lua.create_table()?;
        new_searchers.set(1, preload)?;
        package.set("loaders", new_searchers)?;
    }

    let greet = lua.create_function(|_, vals: LuaVariadic<String>| {
        let name = vals.get(0).map(|v| v.as_str()).unwrap_or("World");
        Ok(format!("Hello, {}", name))
    })?;

    lua.globals().set("greet", greet)?;
    lua.globals().set("io", lua.create_table()?)?;
    let result = lua
        .load(
            r#"
    local lpeg = require("lpeg");
    print(lpeg);
    return greet("Rust");
    "#,
        )
        .eval::<LuaValue>()?;
    println!("lua: {:?}", result);

    Ok(())
}
