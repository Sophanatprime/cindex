use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let artifacts = luajit_src::Build::new().lua52compat(false).build();

    cc::Build::new()
        .files([
            "lpeg-1.1.0/lpcap.c",
            "lpeg-1.1.0/lpcode.c",
            "lpeg-1.1.0/lpcset.c",
            "lpeg-1.1.0/lpprint.c",
            "lpeg-1.1.0/lptree.c",
            "lpeg-1.1.0/lpvm.c",
        ])
        .include(artifacts.include_dir())
        .include("lpeg-1.1.0")
        .compile("lpeg");

    build_jit_modules(
        std::env::var_os("OUT_DIR")
            .expect("unable to get OUT_DIR")
            .as_ref(),
    );
}

fn build_jit_modules(out_dir: &Path) {
    let src_dir = out_dir.join("luajit-build").join("src");

    let jit_dir = src_dir.join("jit");

    assert!(src_dir.join("luajit.h").is_file());
    assert!(jit_dir.is_dir());

    let luajit = src_dir.join(if cfg!(windows) {
        "luajit.exe"
    } else {
        "luajit"
    });

    assert!(
        luajit.is_file(),
        "LuaJIT executable not found: {}",
        luajit.display()
    );

    let generated_dir = out_dir.join("jit-generated");
    fs::create_dir_all(&generated_dir).unwrap();

    let mut modules = Vec::new();

    generate(&jit_dir, &jit_dir, &generated_dir, &luajit, &mut modules);

    let mut rust = String::new();

    for (module, symbol, _) in &modules {
        rust.push_str(&format!(
            "unsafe extern \"C-unwind\" {{ \
                pub unsafe fn {symbol}(L: *mut ::core::ffi::c_void) -> ::core::ffi::c_int;\
            }}\n"
        ));

        let _ = module;
    }

    fs::write(out_dir.join("jit_ffi.rs"), rust).unwrap();

    let mut cc = cc::Build::new();

    cc.include(&src_dir);

    for (_, _, c_file) in &modules {
        cc.file(c_file);
    }

    cc.compile("luajit_jit_bytecode");
}

fn generate(
    dir: &Path,
    root: &Path,
    out: &Path,
    luajit: &Path,
    modules: &mut Vec<(String, String, PathBuf)>,
) {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();

    entries.sort();

    for path in entries {
        if path.is_dir() {
            generate(&path, root, out, luajit, modules);
            continue;
        }

        if path.extension().and_then(|x| x.to_str()) != Some("lua") {
            continue;
        }

        let rel = path.strip_prefix(root).unwrap();

        let tail = rel
            .with_extension("")
            .to_string_lossy()
            .replace(|c| c == '\\' || c == '/', ".");

        let module = format!("jit.{tail}");

        /*
         * jit.lua       -> luaopen_jit
         * jit.dump.lua  -> luaopen_jit_dump
         * ...
         */
        let symbol_tail = module.strip_prefix("jit.").unwrap_or("");

        let symbol = if symbol_tail.is_empty() {
            "luaopen_jit".to_string()
        } else {
            format!("luaopen_jit_{}", symbol_tail.replace('.', "_"))
        };

        let out_c_file = out.join(format!("{}.c", symbol));

        let status = Command::new(luajit)
            .current_dir(luajit.parent().unwrap())
            .arg("-b")
            .arg("-n")
            .arg(&module)
            .arg(&path)
            .arg(&out_c_file)
            .status()
            .expect("Failed to execute luajit");

        assert!(
            status.success(),
            "LuaJIT failed to compile bytecode for: {}",
            path.display()
        );

        let symbol_bc = format!("luaJIT_BC_{}", module.replace('.', "_"));
        let c_wrapper = format!(
            "\n\
            #include \"lua.h\"\n\
            #include \"lauxlib.h\"\n\
            \n\
            int {symbol}(lua_State *L) {{\n\
            \tif (luaL_loadbuffer(L, (const char *){symbol_bc}, sizeof({symbol_bc}), \"={module}\") == 0) {{\n\
            \t\tlua_call(L, 0, 1);\n\
            \t\treturn 1;\n\
            \t}}\n\
            \treturn lua_error(L);\n\
            }}\n"
        );

        use std::io::Write;
        let mut c_file = std::fs::OpenOptions::new()
            .append(true)
            .open(&out_c_file)
            .unwrap();

        c_file.write_all(c_wrapper.as_bytes()).unwrap();

        modules.push((module, symbol, out_c_file));
    }
}
