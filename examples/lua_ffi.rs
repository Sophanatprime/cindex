use cindex::ffi::lua::prelude::*;

fn main() -> anyhow::Result<()> {
    let lua = unsafe { Lua::unsafe_new() };
    lua.globals()
        .raw_set("__api", (api::API as *const _) as usize)?;
    let ffi_decl = r##"
local ffi = require("ffi");
ffi.cdef[[
    typedef struct {
        int (*test_add)(int, int);
    } Api;
]];
api = ffi.cast("Api*", __api);
__api = nil;
    "##;

    lua.load(ffi_decl).exec()?;
    let lua_code = r###"
return api.test_add(114000, 514);
    "###;
    let res: LuaValue = lua.load(lua_code).eval()?;
    println!("return {:?}", res);
    Ok(())
}

mod api {
    use std::ffi::c_int;

    #[repr(C)]
    pub struct Api {
        test_add: unsafe extern "C" fn(c_int, c_int) -> c_int,
    }

    pub static API: &Api = &Api { test_add };

    #[unsafe(no_mangle)]
    unsafe extern "C" fn test_add(a: c_int, b: c_int) -> c_int {
        a + b
    }
}
