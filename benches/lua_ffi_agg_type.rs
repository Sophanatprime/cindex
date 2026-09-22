use cindex::ffi::lua::prelude::*;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn agg_types(c: &mut Criterion) {
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
    typedef struct { uint32_t inner; } Char;
    typedef struct {
        uint32_t (*s_to_char_1)(StrRef);
        uint32_t (*s_to_char_2)(const StrRef*);
        Char (*s_to_char_3)(StrRef);
        Char (*s_to_char_4)(const StrRef*);
        StrRef (*s_to_s_own)(StrRef, size_t, size_t);
        void (*s_to_s_ref)(const StrRef*, size_t, size_t, StrRef*);
    } Api;
]];
__api = ffi.cast("Api*", __api);
local size_t = ffi.typeof("size_t");
local StrRef_t = ffi.typeof("StrRef");
local Char_t = ffi.typeof("Char");

function f1()
    local s = StrRef_t();
    return tonumber(__api.s_to_char_1(s));
end

function f2()
    local s = StrRef_t();
    return tonumber(__api.s_to_char_2(s));
end

function f3()
    local s = StrRef_t();
    return tonumber(__api.s_to_char_3(s).inner);
end

function f4()
    local s = StrRef_t();
    return tonumber(__api.s_to_char_4(s).inner);
end

function s1()
    local s = StrRef_t();
    return __api.s_to_s_own(s, ffi.cast(size_t, 0), ffi.cast(size_t, 0));
end

function s2()
    local s_in = StrRef_t();
    local s_out = StrRef_t();
    __api.s_to_s_ref(s_in, ffi.cast(size_t, 0), ffi.cast(size_t, 0), s_out);
    return s_out;
end

function b1()
    for i = 1,1000000 do
        local n = f1();
    end
end

function b2()
    for i = 1,1000000 do
        local n = f2();
    end
end

function b3()
    for i = 1,1000000 do
        local n = f3();
    end
end

function b4()
    for i = 1,1000000 do
        local n = f4();
    end
end
    "##;

    lua.load(ffi_decl).exec().unwrap();
    let f1 = {
        let f: LuaFunction = lua.globals().get("f1").unwrap();
        lua.create_registry_value(f).unwrap()
    };
    let f2 = {
        let f: LuaFunction = lua.globals().get("f2").unwrap();
        lua.create_registry_value(f).unwrap()
    };
    let f3 = {
        let f: LuaFunction = lua.globals().get("f3").unwrap();
        lua.create_registry_value(f).unwrap()
    };
    let f4 = {
        let f: LuaFunction = lua.globals().get("f4").unwrap();
        lua.create_registry_value(f).unwrap()
    };

    c.bench_function("own/u32", |b| {
        b.iter(|| unsafe {
            let val = lua
                .exec_raw::<LuaValue>((), |state| {
                    for _ in 0..black_box(100_0000) {
                        mlua::ffi::lua_rawgeti(state, mlua::ffi::LUA_REGISTRYINDEX, f1.id() as _);
                        mlua::ffi::lua_call(state, 0, 1);
                        mlua::ffi::lua_pop(state, 1);
                    }
                })
                .unwrap();
            black_box(val)
        });
    });

    c.bench_function("ref/u32", |b| {
        b.iter(|| unsafe {
            let val = lua
                .exec_raw::<LuaValue>((), |state| {
                    for _ in 0..black_box(100_0000) {
                        mlua::ffi::lua_rawgeti(state, mlua::ffi::LUA_REGISTRYINDEX, f2.id() as _);
                        mlua::ffi::lua_call(state, 0, 1);
                        mlua::ffi::lua_pop(state, 1);
                    }
                })
                .unwrap();
            black_box(val)
        });
    });

    c.bench_function("own/Char", |b| {
        b.iter(|| unsafe {
            let val = lua
                .exec_raw::<LuaValue>((), |state| {
                    for _ in 0..black_box(100_0000) {
                        mlua::ffi::lua_rawgeti(state, mlua::ffi::LUA_REGISTRYINDEX, f3.id() as _);
                        mlua::ffi::lua_call(state, 0, 1);
                        mlua::ffi::lua_pop(state, 1);
                    }
                })
                .unwrap();
            black_box(val)
        });
    });

    c.bench_function("ref/Char", |b| {
        b.iter(|| unsafe {
            let val = lua
                .exec_raw::<LuaValue>((), |state| {
                    for _ in 0..black_box(100_0000) {
                        mlua::ffi::lua_rawgeti(state, mlua::ffi::LUA_REGISTRYINDEX, f4.id() as _);
                        mlua::ffi::lua_call(state, 0, 1);
                        mlua::ffi::lua_pop(state, 1);
                    }
                })
                .unwrap();
            black_box(val)
        });
    });

    let s1 = {
        let f: LuaFunction = lua.globals().get("s1").unwrap();
        lua.create_registry_value(f).unwrap()
    };
    let s2 = {
        let f: LuaFunction = lua.globals().get("s2").unwrap();
        lua.create_registry_value(f).unwrap()
    };
    c.bench_function("own-str", |b| {
        b.iter(|| unsafe {
            let val = lua
                .exec_raw::<LuaValue>((), |state| {
                    for _ in 0..black_box(100_0000) {
                        mlua::ffi::lua_rawgeti(state, mlua::ffi::LUA_REGISTRYINDEX, s1.id() as _);
                        mlua::ffi::lua_call(state, 0, 1);
                        mlua::ffi::lua_pop(state, 1);
                    }
                })
                .unwrap();
            black_box(val)
        });
    });
    c.bench_function("ref-str", |b| {
        b.iter(|| unsafe {
            let val = lua
                .exec_raw::<LuaValue>((), |state| {
                    for _ in 0..black_box(100_0000) {
                        mlua::ffi::lua_rawgeti(state, mlua::ffi::LUA_REGISTRYINDEX, s2.id() as _);
                        mlua::ffi::lua_call(state, 0, 1);
                        mlua::ffi::lua_pop(state, 1);
                    }
                })
                .unwrap();
            black_box(val)
        });
    });

    let f1: LuaFunction = lua.globals().get("b1").unwrap();
    let f2: LuaFunction = lua.globals().get("b2").unwrap();
    let f3: LuaFunction = lua.globals().get("b3").unwrap();
    let f4: LuaFunction = lua.globals().get("b4").unwrap();
    c.bench_function("lua(own/u32)", |b| {
        b.iter(|| {
            let n: LuaValue = black_box(f1.clone()).call(()).unwrap();
            black_box(n)
        });
    });
    c.bench_function("lua(ref/u32)", |b| {
        b.iter(|| {
            let n: LuaValue = black_box(f2.clone()).call(()).unwrap();
            black_box(n)
        });
    });
    c.bench_function("lua(own/Chr)", |b| {
        b.iter(|| {
            let n: LuaValue = black_box(f3.clone()).call(()).unwrap();
            black_box(n)
        });
    });
    c.bench_function("lua(ref/Chr)", |b| {
        b.iter(|| {
            let n: LuaValue = black_box(f4.clone()).call(()).unwrap();
            black_box(n)
        });
    });
}

criterion_group!(lua_ffi_agg_type, agg_types);
criterion_main!(lua_ffi_agg_type);

mod api {
    #[repr(C)]
    #[derive(Clone)]
    pub struct StrRef {
        ptr: *const u8,
        len: usize,
    }

    pub const API: &Api = &Api {
        s_to_char_1: s_to_char_own,
        s_to_char_2: s_to_char_ref,
        s_to_char_3: s_to_char_own,
        s_to_char_4: s_to_char_ref,
        s_to_s_own: s_to_s_own,
        s_to_s_ref: s_to_s_ref,
    };

    #[allow(dead_code)]
    pub struct Api {
        pub s_to_char_1: unsafe extern "C" fn(StrRef) -> u32,
        pub s_to_char_2: unsafe extern "C" fn(*const StrRef) -> u32,
        pub s_to_char_3: unsafe extern "C" fn(StrRef) -> u32,
        pub s_to_char_4: unsafe extern "C" fn(*const StrRef) -> u32,
        pub s_to_s_own: unsafe extern "C" fn(StrRef, usize, usize) -> StrRef,
        pub s_to_s_ref: unsafe extern "C" fn(*const StrRef, usize, usize, *mut StrRef),
    }

    unsafe extern "C" fn s_to_char_own(_s: StrRef) -> u32 {
        0
    }
    unsafe extern "C" fn s_to_char_ref(_s: *const StrRef) -> u32 {
        0
    }
    unsafe extern "C" fn s_to_s_own(s: StrRef, _: usize, _: usize) -> StrRef {
        s
    }
    unsafe extern "C" fn s_to_s_ref(s: *const StrRef, _: usize, _: usize, out: *mut StrRef) {
        unsafe {
            *out = (&*s).clone();
        }
    }
}
