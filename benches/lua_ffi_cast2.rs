use api::*;
use cindex::ffi::lua::prelude::*;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn ffi_none(c: &mut Criterion) {
    let lua = unsafe { Lua::unsafe_new() };
    lua.globals()
        .raw_set("__api", (api::API as *const _) as usize)
        .unwrap();
    let ffi_decl = r##"
local ffi = require("ffi");
ffi.cdef[[
    typedef struct {
        int (*test_add)(int, int);
        int (*cmp_strref)(void*, void*);
    } Api;
]];
__api = ffi.cast("Api*", __api);

function compare(a, b)
    return 0;
end
    "##;

    lua.load(ffi_decl).exec().unwrap();
    let a = black_box("114514");
    let b = black_box("1918");
    let a = StrRef {
        ptr: a.as_ptr(),
        len: a.len(),
    };
    let b = StrRef {
        ptr: b.as_ptr(),
        len: b.len(),
    };
    let cmp_ref: LuaFunction = lua.globals().raw_get("compare").unwrap();
    let key = lua.create_registry_value(cmp_ref).unwrap();
    c.bench_function("ffi none 100_0000", |ben| {
        ben.iter(|| unsafe {
            let val = lua
                .exec_raw::<LuaValue>((), |state| {
                    for _ in 0..black_box(100_0000) {
                        mlua::ffi::lua_rawgeti(state, mlua::ffi::LUA_REGISTRYINDEX, key.id() as _);
                        mlua::ffi::lua_pushlightuserdata(state, &a as *const _ as _);
                        mlua::ffi::lua_pushlightuserdata(state, &b as *const _ as _);
                        mlua::ffi::lua_call(state, 2, 1);
                        mlua::ffi::lua_pop(state, 1);
                    }
                })
                .unwrap();
            black_box(val)
        })
    });
}

fn ffi_raw(c: &mut Criterion) {
    let lua = unsafe { Lua::unsafe_new() };
    lua.globals()
        .raw_set("__api", (api::API as *const _) as usize)
        .unwrap();
    let ffi_decl = r##"
local ffi = require("ffi");
ffi.cdef[[
    typedef struct {
        int (*test_add)(int, int);
        int (*cmp_strref)(void*, void*);
    } Api;
]];
__api = ffi.cast("Api*", __api);

function compare(a, b)
    return __api.cmp_strref(a, b);
end
    "##;

    lua.load(ffi_decl).exec().unwrap();
    let a = black_box("114514");
    let b = black_box("1918");
    let a = StrRef {
        ptr: a.as_ptr(),
        len: a.len(),
    };
    let b = StrRef {
        ptr: b.as_ptr(),
        len: b.len(),
    };
    let cmp_ref: LuaFunction = lua.globals().raw_get("compare").unwrap();
    let key = lua.create_registry_value(cmp_ref).unwrap();
    c.bench_function("ffi raw 100_0000", |ben| {
        ben.iter(|| unsafe {
            let val = lua
                .exec_raw::<LuaValue>((), |state| {
                    for _ in 0..black_box(100_0000) {
                        mlua::ffi::lua_rawgeti(state, mlua::ffi::LUA_REGISTRYINDEX, key.id() as _);
                        mlua::ffi::lua_pushlightuserdata(state, &a as *const _ as _);
                        mlua::ffi::lua_pushlightuserdata(state, &b as *const _ as _);
                        mlua::ffi::lua_call(state, 2, 1);
                        mlua::ffi::lua_pop(state, 1);
                    }
                })
                .unwrap();
            black_box(val)
        })
    });
}

fn ffi_cast(c: &mut Criterion) {
    let lua = unsafe { Lua::unsafe_new() };
    lua.globals()
        .raw_set("__api", (api::API as *const _) as usize)
        .unwrap();
    let ffi_decl = r##"
local ffi = require("ffi");
ffi.cdef[[
    typedef struct {
        uint8_t* ptr;
        size_t len;
    } StrRef;
    typedef struct {
        int (*test_add)(int, int);
        int (*cmp_strref)(const StrRef*, const StrRef*);
    } Api;
]];
__api = ffi.cast("Api*", __api);

local StrRefPtr = ffi.typeof("const StrRef*")
function compare(a, b)
    local a = ffi.cast(StrRefPtr, a);
    local b = ffi.cast(StrRefPtr, b);
    return __api.cmp_strref(a, b);
end
    "##;

    lua.load(ffi_decl).exec().unwrap();
    let a = black_box("114514");
    let b = black_box("1918");
    let a = StrRef {
        ptr: a.as_ptr(),
        len: a.len(),
    };
    let b = StrRef {
        ptr: b.as_ptr(),
        len: b.len(),
    };
    let cmp_ref: LuaFunction = lua.globals().raw_get("compare").unwrap();
    let key = lua.create_registry_value(cmp_ref).unwrap();
    c.bench_function("ffi cast 100_0000", |ben| {
        ben.iter(|| unsafe {
            let val = lua
                .exec_raw::<LuaValue>((), |state| {
                    for _ in 0..black_box(100_0000) {
                        mlua::ffi::lua_rawgeti(state, mlua::ffi::LUA_REGISTRYINDEX, key.id() as _);
                        mlua::ffi::lua_pushlightuserdata(state, &a as *const _ as _);
                        mlua::ffi::lua_pushlightuserdata(state, &b as *const _ as _);
                        mlua::ffi::lua_call(state, 2, 1);
                        mlua::ffi::lua_pop(state, 1);
                    }
                })
                .unwrap();
            black_box(val)
        })
    });
}

fn ffi_type(c: &mut Criterion) {
    let lua = unsafe { Lua::unsafe_new() };
    lua.globals()
        .raw_set("__api", (api::API as *const _) as usize)
        .unwrap();
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
    return __api.cmp_strref(a, b);
end
    "##;

    lua.load(ffi_decl).exec().unwrap();
    let a = black_box("114514");
    let b = black_box("1918");
    let a = StrRef {
        ptr: a.as_ptr(),
        len: a.len(),
    };
    let b = StrRef {
        ptr: b.as_ptr(),
        len: b.len(),
    };
    let cmp_ref: LuaFunction = lua.globals().raw_get("compare").unwrap();
    let key = lua.create_registry_value(cmp_ref).unwrap();
    c.bench_function("ffi type 100_0000", |ben| {
        ben.iter(|| unsafe {
            let val = lua
                .exec_raw::<LuaValue>((), |state| {
                    for _ in 0..black_box(100_0000) {
                        mlua::ffi::lua_rawgeti(state, mlua::ffi::LUA_REGISTRYINDEX, key.id() as _);
                        mlua::ffi::lua_pushlightuserdata(state, &a as *const _ as _);
                        mlua::ffi::lua_pushlightuserdata(state, &b as *const _ as _);
                        mlua::ffi::lua_call(state, 2, 1);
                        mlua::ffi::lua_pop(state, 1);
                    }
                })
                .unwrap();
            black_box(val)
        })
    });
}

criterion_group!(lua_ffi_cast2, ffi_none, ffi_raw, ffi_cast, ffi_type);
criterion_main!(lua_ffi_cast2);

mod api {
    use bstr::BStr;
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
