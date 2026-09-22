use cindex::ffi::{lua::prelude::*, LuaModule};

fn main() {
    let lua = unsafe { Lua::unsafe_new() };
    LuaModule::Jits.preload(&lua).unwrap();
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
size_t = ffi.typeof("size_t");
StrRef_t = ffi.typeof("StrRef");
Char_t = ffi.typeof("Char");
    "##;
    lua.load(ffi_decl).exec().unwrap();

    lua.load(
        r#"
        local jit = require("jit")
        jit.opt.start("hotloop=10")
        require("jit.dump").on()

        function bench_1()
            local s = StrRef_t()
            local r

            for i=1,10000000 do
                r = __api.s_to_s_own(s,0,0)
            end

            return tonumber(r.len)
        end

        function bench_2()
            local s = StrRef_t()
            local r = StrRef_t()

            for i=1,10000000 do
                __api.s_to_s_ref(s,0,0,r)
            end

            return tonumber(r.len)
        end
    "#,
    )
    .exec()
    .unwrap();

    let n: i64 = lua.load("bench_2()").eval().unwrap();
    println!("n={n}");
}

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
