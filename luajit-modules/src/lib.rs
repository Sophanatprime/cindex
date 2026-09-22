#![no_std]
use core::ffi::{c_int, c_void};

include!(concat!(env!("OUT_DIR"), "/lua_modules.rs"));

pub static JIT_MODULES: &[(
    &'static str,
    unsafe extern "C-unwind" fn(*mut c_void) -> c_int,
)] = &[
    ("jit.bc", luaopen_jit_bc),
    ("jit.bcsave", luaopen_jit_bcsave),
    ("jit.dis_arm", luaopen_jit_dis_arm),
    ("jit.dis_arm64", luaopen_jit_dis_arm64),
    ("jit.dis_arm64be", luaopen_jit_dis_arm64be),
    ("jit.dis_mips", luaopen_jit_dis_mips),
    ("jit.dis_mips64", luaopen_jit_dis_mips64),
    ("jit.dis_mips64el", luaopen_jit_dis_mips64el),
    ("jit.dis_mips64r6", luaopen_jit_dis_mips64r6),
    ("jit.dis_mips64r6el", luaopen_jit_dis_mips64r6el),
    ("jit.dis_mipsel", luaopen_jit_dis_mipsel),
    ("jit.dis_ppc", luaopen_jit_dis_ppc),
    ("jit.dis_x64", luaopen_jit_dis_x64),
    ("jit.dis_x86", luaopen_jit_dis_x86),
    ("jit.dump", luaopen_jit_dump),
    ("jit.p", luaopen_jit_p),
    ("jit.v", luaopen_jit_v),
    ("jit.vmdef", luaopen_jit_vmdef),
    ("jit.zone", luaopen_jit_zone),
];

unsafe extern "C-unwind" {
    pub unsafe fn luaopen_lpeg(L: *mut c_void) -> c_int;
}
