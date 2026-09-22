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

    build_lua_modules(
        std::env::var_os("OUT_DIR")
            .expect("unable to get OUT_DIR")
            .as_ref(),
        ["argparse"],
    );
}

fn build_lua_modules<P>(out_dir: &Path, other_paths: P)
where
    P: IntoIterator,
    P::Item: AsRef<Path>,
{
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
        "LuaJIT executable is not found: {}",
        luajit.display()
    );

    let generated_dir = out_dir.join("modules-generated");
    fs::create_dir_all(&generated_dir).unwrap();

    let mut modules = Vec::new();

    // generate(&jit_dir, &jit_dir, &generated_dir, &luajit, &mut modules, other_paths);
    generate2(&generated_dir, &luajit, Some(&jit_dir), &mut modules);
    generate2(&generated_dir, &luajit, other_paths, &mut modules);

    let mut rust = String::new();

    for (module, symbol, _) in &modules {
        rust.push_str(&format!(
            "unsafe extern \"C-unwind\" {{ \
                pub unsafe fn {symbol}(L: *mut ::core::ffi::c_void) -> ::core::ffi::c_int;\
            }}\n"
        ));

        let _ = module;
    }

    fs::write(out_dir.join("lua_modules.rs"), rust).unwrap();

    let mut cc = cc::Build::new();

    cc.include(&src_dir);

    for (_, _, c_file) in &modules {
        cc.file(c_file);
    }

    cc.compile("luajit_modules_bytecode");
}

fn generate2<P>(out: &Path, luajit: &Path, paths: P, modules: &mut Vec<(String, String, PathBuf)>)
where
    P: IntoIterator,
    P::Item: AsRef<Path>,
{
    for path in paths.into_iter() {
        let path = path.as_ref();
        let path = if path.is_relative() {
            Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
        } else {
            path.to_path_buf()
        };
        if path.is_dir() {
            let parent_name = path.file_name().unwrap().to_string_lossy();
            let parent_name = parent_name
                .find('-')
                .map_or(parent_name.to_string(), |idx| {
                    parent_name[..idx].to_string()
                });
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap().path();
                generate_module(luajit, out, &parent_name, &entry, modules);
            }
        } else if path.is_file() {
            let parent_name = path.file_stem().unwrap().to_string_lossy();
            generate_module(luajit, out, &parent_name, &path, modules);
        } else {
            panic!("invalid module path: {}", path.display());
        }
    }
}

fn generate_module(
    luajit: &Path,
    out: &Path,
    root: &str,
    path: &Path,
    modules: &mut Vec<(String, String, PathBuf)>,
) {
    if !path.exists() || path.is_dir() {
        panic!("invalid module file: {}", path.display());
    }
    if path.extension().and_then(|x| x.to_str()) != Some("lua") {
        return;
    }

    let rel = path.file_stem().unwrap().to_string_lossy();
    let module = if rel == root {
        rel.to_string()
    } else {
        format!("{}.{}", root, rel)
    };
    let symbol = format!("luaopen_{}", module.replace('.', "_"));

    let out_c_file = out.join(format!("{}.c", &symbol));
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
