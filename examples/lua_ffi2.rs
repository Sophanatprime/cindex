use bstr::BStr;
use cindex::ffi::lua::prelude::*;

use crate::api::StrRef;

fn main() -> anyhow::Result<()> {
    let lua = unsafe { Lua::unsafe_new() };
    lua.globals()
        .raw_set("__api", (api::API as *const _) as usize)?;
    let ffi_decl = r##"
local ffi = require("ffi");
ffi.cdef[[
    typedef struct {
        struct {
            uint8_t* ptr;
            size_t len;
        }* inner;
    } StrRefPtr;
    typedef struct {
        int (*test_add)(int, int);
        int (*cmp_strref)(StrRefPtr, StrRefPtr);
    } Api;
]];
__api = ffi.cast("Api*", __api);

strref = ffi.metatype("StrRefPtr", {
    __tostring = function(s) return ffi.string(s.inner.ptr, s.inner.len) end,
    __len = function(s) return s.inner.len end,
});

function compare(a, b)
    local a = strref(a);
    local b = strref(b);
    print(a);
    return __api.cmp_strref(a, b);
end
    "##;

    lua.load(ffi_decl).exec()?;
    let res: LuaValue = unsafe {
        let a = "114514";
        let b = "1918";
        let a = StrRef {
            ptr: a.as_ptr(),
            len: a.len(),
        };
        let b = StrRef {
            ptr: b.as_ptr(),
            len: b.len(),
        };
        let cmp_ref: LuaFunction = lua.globals().raw_get("compare")?;
        lua.exec_raw::<LuaValue>((cmp_ref,), |state| {
            mlua::ffi::lua_pushlightuserdata(state, &a as *const _ as _);
            mlua::ffi::lua_pushlightuserdata(state, &b as *const _ as _);
            mlua::ffi::lua_call(state, 2, 1);
            // let n = mlua::ffi::lua_tonumber(state, -1);
            // println!("n: {}", n);
            // mlua::ffi::lua_pop(state, -1);
        })?
    };
    println!("res: {:?}", res);
    Ok(())
}

mod api {
    use super::BStr;
    use std::{cmp::Ordering, ffi::c_int};

    #[repr(C)]
    pub struct StrRef {
        pub(super) ptr: *const u8,
        pub(super) len: usize,
    }

    #[repr(C)]
    pub struct Api {
        test_add: unsafe extern "C" fn(c_int, c_int) -> c_int,
        cmp_strref: unsafe extern "C" fn(*const StrRef, *const StrRef) -> c_int,
    }

    pub static API: &Api = &Api {
        test_add,
        cmp_strref,
    };

    #[unsafe(no_mangle)]
    unsafe extern "C" fn test_add(a: c_int, b: c_int) -> c_int {
        a + b
    }

    #[unsafe(no_mangle)]
    unsafe extern "C" fn cmp_strref(a: *const StrRef, b: *const StrRef) -> c_int {
        let a = unsafe { std::slice::from_raw_parts((*a).ptr, (*a).len) };
        let b = unsafe { std::slice::from_raw_parts((*b).ptr, (*b).len) };
        let a = BStr::new(a);
        let b = BStr::new(b);
        match a.cmp(b) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }
}
