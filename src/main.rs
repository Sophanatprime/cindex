use std::{
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow, bail};
use cindex::{
    IstFile, PagePrecedenceProvider,
    ffi::lua::prelude::{IntoLua, LuaFunction, LuaNil, LuaString, LuaTable, LuaValue},
    luaopen_cindex_table,
};
use clap::{Parser, Subcommand, ValueEnum};
use compact_str::CompactString;
use rustc_hash::FxHashMap;
use serde::Serialize;

use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn panic_handler(info: &std::panic::PanicHookInfo) {
    match info.payload_as_str() {
        Some(s) => println!("cindex raised an ERROR: {}", s),
        None => println!("cindex raised an ERROR."),
    }
    std::process::exit(1)
}

fn main() {
    std::panic::set_hook(Box::new(panic_handler));

    let timer = std::time::Instant::now();
    let cli = Cli::parse();

    let mut logger = fern::Dispatch::new().format(|out, message, record| {
        out.finish(format_args!("[{}] {}", record.level(), message))
    });
    if cli.command.is_some() {
        if !cli.quiet {
            logger
                .level(log::LevelFilter::Warn)
                .chain(std::io::stderr())
                .apply()
                .unwrap();
        }
    } else {
        if cli.quiet {
            logger = logger.level(log::LevelFilter::Info);
        } else {
            logger = logger
                .level(log::LevelFilter::Info)
                .chain(std::io::stderr());
        }

        if let Some(log_file) = &cli.log_file {
            logger = logger.chain(std::fs::File::create(log_file).unwrap());
        } else if cli.input_files.is_empty() {
            // pass
        } else {
            let mut log_file = Path::new(&cli.input_files[0]);
            log_file = Path::new(log_file.file_name().unwrap());
            logger = logger.chain(std::fs::File::create(log_file.with_extension("ilg")).unwrap());
        };

        logger.apply().unwrap();
    }

    log::info!(
        "This is cindex, version {} {}. Copyright 2026 Wenjian Chern<longaster@163.com>",
        env!("CARGO_PKG_VERSION"),
        "2026-08-31"
    );

    if cli.license {
        print_license();
        std::process::exit(0)
    }

    if let Some(sub) = &cli.command {
        match sub {
            Commands::Query {
                escaped,
                kind,
                text,
            } => {
                if *escaped {
                    process_query(kind, &normalize_unicode_escape(text));
                } else {
                    process_query(kind, text)
                }
            }
        }
    } else {
        if let Err(e) = process_make_index(&cli) {
            log::error!(target: "cindex", "{e}");
        }
    }

    log::info!("Completed in {}ms", timer.elapsed().as_millis());
}

fn normalize_unicode_escape(mut s: &str) -> std::borrow::Cow<'_, str> {
    if !s.contains("\\u") {
        return std::borrow::Cow::Borrowed(s);
    }
    let mut res = String::new();
    while let Some(idx) = memchr::memmem::find(s.as_bytes(), b"\\u") {
        res.push_str(&s[..idx]);
        s = &s[idx + 2..];
        if s.is_empty() {
            res.push_str("\\u");
            break;
        }
        if s.as_bytes()[0] == b'{' {
            match memchr::memchr(b'}', s.as_bytes()) {
                Some(idx) => {
                    let n = u32::from_str_radix(&s[1..idx], 16)
                        .unwrap_or_else(|_| panic!("invalid number: {}", &s[1..idx]));
                    res.push(
                        char::from_u32(n).unwrap_or_else(|| panic!("invalid char: U+{:04X}", n)),
                    );
                    s = &s[idx + 1..];
                }
                None => {
                    res.push_str("\\u");
                    res.push_str(s);
                    break;
                }
            }
        } else {
            let n = u32::from_str_radix(&s[..4.min(s.len())], 16)
                .unwrap_or_else(|_| panic!("invalid number: {}", &s[..4.min(s.len())]));
            res.push(char::from_u32(n).unwrap_or_else(|| panic!("invalid char: U+{:04X}", n)));
            s = &s[4.min(s.len())..];
        }
    }

    std::borrow::Cow::Owned(res)
}

/// cindex – a program similar to LaTeX's makeindex for generating sorted indexes.
///
/// cindex processes raw index data (.idx) files and produces formatted index (.ind) files,
/// optionally applying style definitions and supporting various sorting and grouping methods.
#[derive(Parser, Debug)]
#[command(name = "cindex", version, about, long_about = None)]
pub struct Cli {
    /// Compress leading and trailing spaces around sort keys.
    /// By default, spaces in sort keys are preserved.
    #[arg(short = 'c', long = "compress-spaces")]
    pub compress_spaces: bool,

    /// Read index entries from standard input (stdin).
    /// If this option is used and no output file is given with -o, the sorted index
    /// is written to standard output (stdout).
    #[arg(short = 'i', long = "stdin")]
    pub stdin_input: bool,

    /// Set the output index file to <ind>.
    /// If not specified, the default output file name is the base name of the first input
    /// file <idx0> with the extension .ind.
    #[arg(short = 'o', long = "output", value_name = "ind")]
    pub output: Option<String>,

    /// Disable implicit page range formation.
    /// When set, page ranges must be generated using explicit range operators.
    /// By default, three or more consecutive page numbers are automatically merged
    /// into a range (e.g., 1–5).
    #[arg(short = 'r', long = "no-implicit-ranges")]
    pub no_implicit_ranges: bool,

    /// Set the style file to <sty>. There is no default.
    /// If the provided name has no extension, `.ist` is appended.
    /// cindex first looks for the style file in the current directory,
    /// then uses the kpathsea library to search the TEXMF tree.
    /// If the file is not a `.ist` file, it is treated as a Lua script.
    #[arg(short = 's', long = "style", value_name = "sty")]
    pub style: Option<String>,

    /// Set the log file to <log>. By default, the log file is named after the first
    /// input file <idx0> with the extension .ilg.
    #[arg(short = 't', long = "log", value_name = "log")]
    pub log_file: Option<String>,

    /// Strict mode. Distinguish page numbers that have different embedded commands.
    /// By default, during page range construction, if the command in the left page range
    /// does not match the right one, the left command is used (some LaTeX documents
    /// produce index entries with a missing right range command). With --strict,
    /// the commands on both sides of a range must be strictly identical.
    #[arg(long = "strict")]
    pub strict: bool,

    /// Set index grouping and sorting methods to <sorts>, which can be one or more
    /// comma-separated names (e.g., "stroke,mandarin").
    /// This option can only be specified once.
    #[arg(short = 'z', long = "sorts", verbatim_doc_comment)]
    pub sorts: Option<String>,

    /// Treat the input files as ikv format (cindex-specific).
    #[arg(long = "kv")]
    pub kv_format: bool,

    /// Input index files. At least one file is required unless -i (stdin input) is used.
    /// Multiple files are processed in the given order.
    #[arg(value_name = "idx", required = false, num_args = 0..)]
    pub input_files: Vec<String>,

    #[arg(last = true)]
    pub extras: Vec<String>,

    /// Quiet mode. Suppress progress and informational messages on stderr.
    /// By default, processing messages and errors are output to both stderr and the log file.
    #[arg(short = 'q', long = "quiet", global = true)]
    pub quiet: bool,

    /// Print license information for cindex and all bundled components, then exit.
    #[arg(long = "license", global = true)]
    pub license: bool,

    /// Subcommands for querying character data.
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Query character information such as pronunciation, radical, stroke count, or stroke order.
    Query {
        /// The input string is escaped by \uHHHH or \u{H..}.
        #[arg(short = 'e', long = "escaped")]
        escaped: bool,

        /// The kind of data to retrieve.
        #[arg(value_name = "kind")]
        kind: QueryKind,

        /// The input string (one or more characters) to query.
        #[arg(value_name = "text")]
        text: String,
    },
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
pub enum QueryKind {
    /// Most common Mandarin reading (pinyin).
    #[value(alias = "pinyin")]
    Mandarin,
    /// Most common Cantonese reading (jyutping).
    #[value(aliases = ["yueyu", "jyutping"])]
    Cantonese,
    /// Radical and residual stroke count.
    #[value(alias = "bushou")]
    Radical,
    /// Ordered strokes.
    #[value(alias = "bihua")]
    StrokesName,
    StrokesDigit,
}

fn print_license() {
    const LICENSE_INFO: &str = "\
cindex\tGNU Lesser General Public License v2.1 or later
\thttps://github.com/Sophanatprime/cindex/LICENSE
lpeg-1.1.0\tMIT LICENSE
\thttps://www.inf.puc-rio.br/~roberto/lpeg/#license
argparse-0.7.2\tMIT LICENSE
\thttps://github.com/luarocks/argparse/blob/master/LICENSE
Unicode Data\tUnicode License v3
\thttps://www.unicode.org/license.txt
Unihan Data\tUnicode License v3
\thttps://www.unicode.org/license.txt
ids(ts.txt)\tMIT License
\thttps://github.com/yi-bai/ids/blob/main/LICENSE
StrokeOrder.txt\thttps://github.com/CNMan/UnicodeCJK-WuBi/
";
    print!("{}", LICENSE_INFO);
}

fn process_query(kind: &QueryKind, text: &str) {
    use cindex::han::*;
    use icu_properties::{
        CodePointMapData,
        props::{NamedEnumeratedProperty, Script},
    };

    let cpmap = CodePointMapData::<Script>::new();

    let mut itoa_buf = itoa::Buffer::new();
    for c in text.chars() {
        let info = cjk_info(c);
        print!("U+{:04X} {}: ", c as u32, c);
        if let Some(info) = info {
            match kind {
                QueryKind::Mandarin => {
                    if let Some(v) = info.mandarin() {
                        println!(
                            "{}{}",
                            v.0,
                            if v.1 != 0 { itoa_buf.format(v.1) } else { "" }
                        )
                    } else {
                        println!("N/A[NIL]");
                    }
                }
                QueryKind::Cantonese => {
                    if let Some(v) = info.mandarin() {
                        println!(
                            "{}{}",
                            v.0,
                            if v.1 != 0 { itoa_buf.format(v.1) } else { "" }
                        )
                    } else {
                        println!("N/A[NIL]");
                    }
                }
                QueryKind::Radical => {
                    let rad = info.radical();
                    match kangxi_ideograph(rad) {
                        (Some(kx), ideo) => println!(
                            "{}[kx=U+{:04X},ideo={}].{}",
                            rad.str_repr(),
                            kx as u32,
                            ideo,
                            itoa_buf.format(info.additional_strokes())
                        ),
                        (None, ideo) => println!(
                            "{}[ideo={}].{}",
                            rad.str_repr(),
                            ideo,
                            itoa_buf.format(info.additional_strokes())
                        ),
                    }
                }
                QueryKind::StrokesName | QueryKind::StrokesDigit => {
                    let kts = info.k_total_strokes();
                    let gts = info.glyph_total_strokes();
                    if kts == gts {
                        print!("[g={}]", itoa_buf.format(gts));
                    } else if gts == 0 {
                        print!("[k={},", itoa_buf.format(kts));
                    } else {
                        print!("[k={},", itoa_buf.format(kts));
                        print!("g={}]", itoa_buf.format(gts));
                    }

                    if let Some(strokes) = ordered_strokes(c) {
                        if kind == &QueryKind::StrokesName {
                            for s in strokes.strokes() {
                                print!(
                                    "{}",
                                    match s {
                                        Stroke::S1 => "横",
                                        Stroke::S2 => "竖",
                                        Stroke::S3 => "撇",
                                        Stroke::S4 => "点",
                                        Stroke::S5 => "折",
                                    }
                                );
                            }
                        } else if kind == &QueryKind::StrokesDigit {
                            for s in strokes.strokes() {
                                print!("{}", itoa_buf.format(s as u8));
                            }
                        }
                    }
                    println!();
                }
            }
        } else {
            match cpmap.get(c) {
                Script::Han => println!("N/A[NIL]"),
                s => println!("N/A[Script={}]", s.long_name()),
            }
        }
    }
}

fn process_make_index(args: &Cli) -> anyhow::Result<()> {
    let ist_path = args.style.as_ref().map(|s| {
        let p = Path::new(s);
        if p.extension().is_none() {
            p.with_extension("ist")
        } else {
            p.to_path_buf()
        }
    });
    let ist_path_handle = std::thread::spawn(move || {
        ist_path.map(|p| {
            if p.exists() {
                p
            } else {
                PathBuf::from(
                    cindex::ffi::kpse_find_file(&p, false)
                        .unwrap_or_else(|_| panic!("unable to find style file: {}", p.display()))
                        .unwrap_or_else(|| panic!("unable to find style file: {}", p.display())),
                )
            }
        })
    });

    let mut input = Vec::new();
    let mut input_handles = Vec::new();
    let (tx, rx) = std::sync::mpsc::channel();
    for (idx, p) in args.input_files.iter().enumerate() {
        let tx = tx.clone();
        let path = PathBuf::from(p);
        let handler = std::thread::spawn(move || {
            if path.exists() {
                tx.send((idx, path)).unwrap();
            } else if path.extension().is_none() {
                let real_path = match cindex::ffi::kpse_find_file(&path, false) {
                    Ok(Some(path)) => path,
                    _ => cindex::ffi::kpse_find_file(path.with_extension("idx"), false)
                        .unwrap_or_else(|_| panic!("unable to find input file: {}", path.display()))
                        .unwrap_or_else(|| panic!("unable to find input file: {}", path.display())),
                };
                tx.send((idx, PathBuf::from(real_path))).unwrap();
            } else {
                let real_path = cindex::ffi::kpse_find_file(&path, false)
                    .unwrap_or_else(|_| panic!("unable to find input file: {}", path.display()))
                    .unwrap_or_else(|| panic!("unable to find input file: {}", path.display()));
                tx.send((idx, PathBuf::from(real_path))).unwrap();
            }
        });
        input_handles.push(handler);
    }
    drop(tx);

    let idx_string = if args.stdin_input {
        let mut buf = Vec::new();
        let mut stdin = std::io::stdin();
        stdin.read_to_end(&mut buf).unwrap();
        // BOM
        let mut ignore_len = 0;
        while buf[ignore_len..].starts_with("\u{feff}".as_bytes()) {
            ignore_len += '\u{feff}'.len_utf8();
        }
        if ignore_len != 0 {
            let kpet_len = buf.len() - ignore_len;
            unsafe {
                std::ptr::copy(buf.as_mut_ptr().add(ignore_len), buf.as_mut_ptr(), kpet_len);
            };
            buf.truncate(kpet_len);
        }
        vec![(None, String::from_utf8(buf).unwrap())]
    } else {
        input.resize(args.input_files.len(), PathBuf::new());
        for _ in 0..input_handles.len() {
            let (idx, path) = rx
                .recv()
                .with_context(|| anyhow!("thread raised an error when find input files"))?;
            input[idx] = path;
        }

        input
            .iter()
            .filter_map(|s| {
                std::fs::read_to_string(s)
                    .inspect_err(|e| {
                        log::error!(target: "cindex", "{e}");
                    })
                    .ok()
                    .map(|f| (Some(PathBuf::from(s)), f))
            })
            .collect()
    };

    let output = match &args.output {
        Some(p) => Some(PathBuf::from(p)),
        None if idx_string.is_empty() => bail!("output filename is required"),
        None => match &idx_string[0] {
            (Some(p), _) => Some(Path::new(p.file_name().unwrap()).with_extension("ind")),
            (None, _) => None,
        },
    };
    let output_cloned = output.clone();
    let out_ok_handle = std::thread::spawn(move || match output_cloned {
        Some(p) => cindex::ffi::kpse_out_name_ok(p),
        None => true,
    });

    let (ist, lua_ist) = ist_path_handle
        .join()
        .expect("unable to execute kpsewhich")
        .map(|path| {
            if matches!(path.extension(), Some(p) if matches!(p.to_str(), Some("lua") | Some("cilua") )) {
                (IstFile::default(), Some(path.to_string_lossy().to_string()))
            } else {
                (
                    IstFile::read(&path)
                        .unwrap_or_else(|_| panic!("unable to read ist file: {}", path.display())),
                    None,
                )
            }
        })
        .unwrap_or_default();
    let (ist_input, ist_output) = ist.split();

    let mut api = cindex::ffi::Api::default();
    api.set_ist_input((&ist_input) as *const _)
        .set_ist_output((&ist_output) as *const _);

    let lua = match cindex::ffi::make_lua(lua_ist, |lua| {
        use cindex::ffi::lua::{Integer, IntoLua, Value};

        lua.preload_module("cindex.table", lua.create_function(luaopen_cindex_table)?)?;

        lua.globals().raw_set("__api", &api as *const _ as usize)?;
        let c_funcs_api = lua.create_table()?;
        c_funcs_api.raw_set(
            "indexentry_to_table",
            lua.create_function::<_, Integer, Value>(|lua, arg| {
                if arg == 0 {
                    Ok(Value::Nil)
                } else {
                    let entry = unsafe { &*(arg as usize as *const cindex::IndexEntry) };
                    IntoLua::into_lua(entry, lua)
                }
            })?,
        )?;
        c_funcs_api.raw_set(
            "mergedentry_to_table",
            lua.create_function::<_, Integer, Value>(|lua, arg| {
                if arg == 0 {
                    Ok(Value::Nil)
                } else {
                    let entry = unsafe { &*(arg as usize as *const cindex::MergedEntry) };
                    IntoLua::into_lua(entry, lua)
                }
            })?,
        )?;
        c_funcs_api.raw_set(
            "orderedstrokes_to_string",
            lua.create_function::<_, (Integer, Integer), Value>(|lua, arg| {
                let ptr = arg.0 as *const u8;
                let len = arg.1 as usize;
                let os = unsafe {
                    cindex::han::RawStrokes::from_strokes(std::slice::from_raw_parts(ptr, len))
                };
                let Some(os) = os else {
                    return Ok(LuaNil);
                };
                let mut s = CompactString::with_capacity(os.len());
                for kind in os.strokes() {
                    s.push((kind as u8 + 0x30) as char);
                }
                lua.create_string(s.as_bytes()).map(LuaValue::String)
            })?,
        )?;
        c_funcs_api.raw_set(
            "table_to_json",
            lua.create_function::<_, LuaTable, Value>(|lua, tab| {
                let Ok(json_text) = serde_json::to_string_pretty(&tab) else {
                    return Err(cindex::ffi::lua::Error::SerializeError(
                        "erroneous calling table.to_json".to_string(),
                    ));
                };
                lua.create_string(json_text.as_bytes())
                    .map(LuaValue::String)
            })?,
        )?;
        c_funcs_api.raw_set(
            "value_from_json",
            lua.create_function::<_, LuaString, Value>(|lua, s| {
                let Ok(value) = serde_json::from_str::<'_, serde_json::Value>(&s.to_string_lossy())
                else {
                    return Err(cindex::ffi::lua::Error::SerializeError(
                        "erroneous calling value_from_json".to_string(),
                    ));
                };
                println!("{:?}", value);
                let ser = cindex::ffi::lua::serde::Serializer::new(lua);
                value.serialize(ser)
            })?,
        )?;
        lua.globals().raw_set("__c_function_api", c_funcs_api)?;

        let cli_options = lua.create_table()?;
        cli_options.raw_set("c", args.compress_spaces)?;
        cli_options.raw_set("q", args.quiet)?;
        cli_options.raw_set("r", args.no_implicit_ranges)?;
        cli_options.raw_set("strict", args.strict)?;
        if let Some(sort_options) = &args.sorts {
            cli_options.raw_set("z", sort_options.as_str())?;
        }
        if !args.extras.is_empty() {
            let extras = lua.create_table_with_capacity(args.extras.len(), 0)?;
            args.extras
                .iter()
                .try_for_each(|s| extras.raw_push(s.as_str()))?;
            cli_options.raw_set("extra_options", extras)?;
        }
        lua.globals().raw_set("CliOptions", cli_options)?;

        Ok(())
    }) {
        Ok(lua) => lua,
        Err(e) => bail!("{}", e),
    };

    let mut entries = Vec::<cindex::IndexEntry>::new();
    let read_indices: LuaFunction = lua.globals().raw_get("__cindex_read_indices")?;
    for (p, f) in &idx_string {
        let (p, ext_ikv) = match p {
            Some(p) => {
                let path = p
                    .to_string_lossy()
                    .replace("\\", "/")
                    .into_lua(&lua)
                    .unwrap();
                (
                    path,
                    matches!(p.extension(), Some(k) if k.to_str() == Some("ikv")),
                )
            }
            None => (LuaNil, false),
        };
        let f = cindex::ffi::StrRef::from_str(f);
        read_indices
            .call::<()>((
                p,
                &f as *const _ as i64,
                args.kv_format | ext_ikv,
                &mut entries as *mut _ as i64,
            ))
            .inspect_err(|e| {
                log::error!(target: "cindex", "{e}");
            })
            .ok();
    }

    let raw_len = entries.len();
    if raw_len == 0
        && let Ok(emptiness_fallback) = lua
            .globals()
            .raw_get::<LuaFunction>("__cindex_reading_empty")
            .inspect_err(|e| {
                log::error!(target: "cindex", "{e}");
            })
    {
        emptiness_fallback
            .call::<()>(&mut entries as *mut _ as i64)
            .inspect_err(|e| {
                log::error!(target: "cindex", "{e}");
            })
            .ok();
    }
    let len_after_re_reading = entries.len();

    let page_cmp_prec = if let Ok(val) = lua
        .globals()
        .raw_get::<LuaValue>("__cindex_page_precedence")
    {
        match &val {
            LuaValue::Function(f) => match f.call::<LuaValue>(()).unwrap() {
                LuaValue::String(s) => s.to_string_lossy().to_precedence(),
                LuaValue::Table(t) => {
                    let t = t
                        .sequence_values::<LuaString>()
                        .map(|s| s.unwrap().to_string_lossy());
                    cindex::IteratorPrecedence(t.collect::<Vec<_>>().iter()).to_precedence()
                }
                LuaValue::Nil => ist_output.page_precedence.to_precedence(),
                _ => {
                    log::warn!(target: "cindex", "invalid return value of precedence, got {}", val.type_name());
                    ist_output.page_precedence.to_precedence()
                }
            },
            LuaValue::String(s) => s.to_string_lossy().to_precedence(),
            LuaValue::Table(t) => {
                let t = t
                    .sequence_values::<LuaString>()
                    .map(|s| s.unwrap().to_string_lossy());
                cindex::IteratorPrecedence(t.collect::<Vec<_>>().iter()).to_precedence()
            }
            LuaValue::Nil => "rnaRA".to_precedence(),
            _ => {
                log::warn!(target: "cindex", "invalid type for precedence, got {}", val.type_name());
                ist_output.page_precedence.to_precedence()
            }
        }
    } else {
        ist_output.page_precedence.to_precedence()
    };
    log::info!(target: "cindex", "Page precedence: {}.", page_cmp_prec.display());
    let page_cmp = cindex::get_page_cmp(page_cmp_prec);
    let entries = cindex::merge_entries(
        entries,
        (&ist_input, &ist_output),
        page_cmp,
        !args.no_implicit_ranges,
        args.strict,
    );

    log::info!(
        target: "cindex",
        "Reading from {}, found {} entries. Kept {} entries after merged.",
        if let &[(None, _)] = &*idx_string {
            String::from("stdin")
        } else {
            format!(
                "[ {} ]",
                idx_string.iter()
                    .map(|s| s.0.as_ref().unwrap().display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        },
        if raw_len == 0 {
            format!("{} [emptiness fallback]", len_after_re_reading)
        } else {
            format!("{}", raw_len)
        },
        entries.len(),
    );

    let has_entry = !entries.is_empty();
    if has_entry {
        let detect_groups: LuaFunction = lua.globals().raw_get("__cindex_detect_groups")?;

        // lua.load(r##"
        //     local jit = require("jit");
        //     jit.opt.start("hotloop=10");
        //     require("jit.dump").on("tli");
        // "##).exec().unwrap();

        let mut groups = vec![0u32; entries.len()];
        let group_args = (
            entries.first().map(|e| e as *const _).unwrap_or_default() as i64, // start ptr
            std::mem::size_of::<cindex::MergedEntry>(),                        // ptr offset
            entries.len(),                                                     // len
            groups.as_mut_ptr() as i64,                                        // result ptr
        );
        let groups_table: LuaTable = detect_groups.call(group_args)?;
        let groups_internal: Vec<String> = groups_table
            .sequence_values()
            .map(|n| {
                n.inspect_err(|e| {
                    log::error!(target: "cindex", "{e}");
                })
                .unwrap_or_default()
            })
            .collect();

        let mut group_categories = vec![0usize; groups_internal.len()];
        groups
            .iter()
            .for_each(|&i| group_categories[i as usize] += 1);

        lua.globals()
            .raw_get::<LuaFunction>("__cindex_sort_groups")?
            .call::<()>((groups_table.clone(), group_categories.as_ptr() as i64))?;

        let mut split_entries = FxHashMap::from_iter(
            groups_internal
                .iter()
                .zip(group_categories.iter().map(|len| Vec::with_capacity(*len))),
        );
        entries
            .into_iter()
            .zip(&groups)
            .for_each(|(e, &group_idx)| {
                let key = &groups_internal[group_idx as usize];
                split_entries.get_mut(key).expect("unreachable").push(e);
            });

        let map = lua
            .create_table_with_capacity(0, split_entries.len())
            .unwrap();
        for (k, v) in split_entries.iter_mut() {
            let v_t = lua.create_table_with_capacity(v.len(), 0).unwrap();
            for p in v {
                v_t.raw_push(p as *mut _ as i64).unwrap();
            }
            map.raw_set(k.as_str(), v_t).unwrap();
        }
        lua.globals()
            .raw_get::<LuaFunction>("__cindex_sort_entries")?
            .call::<()>(map.clone())?;

        match out_ok_handle.join() {
            Ok(true) => {}
            _ => bail!("writing to {} is forbidden", output.unwrap().display()),
        }
        let mut write_buf = cindex::ffi::Writer(match &output {
            Some(p) => Box::new(std::io::BufWriter::new(std::fs::File::create(p).unwrap())),
            None => Box::new(std::io::stdout()),
        });
        lua.globals()
            .raw_get::<LuaFunction>("__cindex_write_entries")?
            .call::<()>((&mut write_buf as *mut _ as i64, groups_table, map))?;
    }

    let writing_info = if has_entry {
        "done"
    } else {
        "nothing to write"
    };
    match &output {
        Some(output) => log::info!("Writing to: '{}', {}", output.display(), writing_info),
        None => log::info!("Writing to stdout, {}", writing_info),
    }

    Ok(())
}
