use std::{
    collections::{BTreeSet, HashMap},
    fs::read_to_string,
    io::{BufWriter, Write},
    path::PathBuf,
    rc::Rc,
};

use modular_bitfield::{
    bitfield,
    specifiers::{B3, B5, B10, B11},
};

fn main() {
    println!("cargo::rerun-if-changed=UniData");
    make_han();
}

fn make_han() {
    let out_dir = std::env::var("OUT_DIR").expect("unable to get OUT_DIR var");
    let mut out_file = PathBuf::from(out_dir);

    out_file.push("generated_han.rs");
    let generated_han =
        std::fs::File::create(&out_file).expect("unable to create generated_han.rs");
    let mut generated_han = BufWriter::new(generated_han);
    write_radical(&mut generated_han);
    write_cjk_info(&mut generated_han);

    out_file.set_file_name("han_stroke_order.rs");
    let han_stroke_order =
        std::fs::File::create(&out_file).expect("unable to create han_stroke_order.rs");
    let mut han_stroke_order = BufWriter::new(han_stroke_order);
    write_han_order(&mut han_stroke_order);
}

fn write_radical(generated_han: &mut impl Write) {
    let radicals =
        read_to_string("UniData/CJKRadicals.txt").expect("unable to read CJKRadicals.txt");
    let mut radical_kinds = vec![];
    let mut kx_ideo = vec![];
    for line in radicals.lines() {
        let line = line.trim_ascii();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut fields = line.split(";");

        let kind = fields
            .next()
            .expect("illegal CJKRadicals data")
            .trim_ascii();
        let kind = match kind.bytes().position(|b| b == b'\'') {
            Some(i) => (
                u8::from_str_radix(&kind[..i], 10).expect("illegal CJKRadicals data"),
                (kind.len() - i) as u8,
            ),
            None => (
                u8::from_str_radix(kind, 10).expect("illegal CJKRadicals data"),
                0,
            ),
        };
        radical_kinds.push(kind);

        let kx = fields
            .next()
            .expect("illegal CJKRadicals data")
            .trim_ascii();
        let kx = if kx.is_empty() {
            None
        } else {
            let kx = u32::from_str_radix(kx, 16).expect("illegal CJKRadicals data");
            Some(char::from_u32(kx).expect("illegal CJKRadicals data"))
        };
        let ideo = fields
            .next()
            .expect("illegal CJKRadicals data")
            .trim_ascii();
        let ideo = u32::from_str_radix(ideo, 16)
            .ok()
            .and_then(|ideo| char::from_u32(ideo))
            .expect("illegal CJKRadicals data");
        kx_ideo.push((kx, ideo));
    }

    let push_radical = |buf: &mut String, kind: u8, a: u8| {
        buf.push('R');
        let mut itoa_buf = itoa::Buffer::new();
        buf.push_str(itoa_buf.format(kind));
        if a > 0 {
            buf.push('_');
            buf.push_str(itoa_buf.format(a));
        }
    };

    let mut tmp_str = String::new();
    let mut itoa_buf = itoa::Buffer::new();

    let len_rad_kinds = radical_kinds.iter().map(|i| i.0).max().unwrap_or(0);
    let len_rad_all_kinds = radical_kinds.len();
    generated_han
        .write_all(
            format!(
                "pub const MAX_RADICAL: (u{}, u{}) = ({}, {});\n",
                num_bit_len(len_rad_kinds as _),
                num_bit_len(len_rad_all_kinds as _),
                len_rad_kinds,
                len_rad_all_kinds,
            )
            .as_bytes(),
        )
        .expect("unable to write generated_han.rs");

    generated_han
        .write_all(
            b"#[repr(u8)]\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum Radical {\n",
        )
        .expect("unable to write generated_han.rs");
    for (kind, c) in radical_kinds.iter().zip(&kx_ideo) {
        tmp_str.clear();
        tmp_str.push_str("/// ");
        if let Some(kx) = c.0 {
            tmp_str.push_str(&format!("KangXi: {}(U+{:04X}), ", kx, kx as u32));
        }
        tmp_str.push_str(&format!("Ideograph: {}(U+{:04X}).", c.1, c.1 as u32));
        tmp_str.push('\n');
        generated_han
            .write_all(tmp_str.as_bytes())
            .expect("unable to write generated_han.rs");

        tmp_str.clear();
        push_radical(&mut tmp_str, kind.0, kind.1);
        tmp_str.push_str(",\n");
        generated_han
            .write_all(tmp_str.as_bytes())
            .expect("unable to write generated_han.rs");
    }
    generated_han
        .write_all(b"}\n")
        .expect("unable to write generated_han.rs");

    generated_han
        .write_all(b"impl Radical {\n")
        .expect("unable to write generated_han.rs");
    // impl Radical::str_repr
    generated_han
        .write_all(b"pub const fn str_repr(&self) -> &'static str {\nmatch self {\n")
        .expect("unable to write generated_han.rs");
    for kind in &radical_kinds {
        tmp_str.clear();
        push_radical(&mut tmp_str, kind.0, kind.1);
        writeln!(
            generated_han,
            "Radical::{} => \"{}{}\",",
            tmp_str,
            itoa_buf.format(kind.0),
            "\'".repeat(kind.1 as _)
        )
        .expect("unable to write generated_han.rs");
    }
    generated_han
        .write_all(b"}\n}\n")
        .expect("unable to write generated_han.rs");
    // impl Radical::original
    generated_han
        .write_all(b"pub const fn original(&self) -> Self {\nmatch self {\n")
        .expect("unable to write generated_han.rs");
    for kind in &radical_kinds {
        if kind.1 == 0 {
            continue;
        }
        tmp_str.clear();
        push_radical(&mut tmp_str, kind.0, kind.1);
        writeln!(
            generated_han,
            "Radical::{} => Radical::{},",
            tmp_str,
            &tmp_str[..tmp_str.len() - 2]
        )
        .expect("unable to write generated_han.rs");
    }
    generated_han
        .write_all(b"_ => *self,\n}\n}\n")
        .expect("unable to write generated_han.rs");
    // impl Radical::to_index
    generated_han
        .write_all(b"pub const fn to_index(&self) -> (u8, u8) {\nmatch self {\n")
        .expect("unable to write generated_han.rs");
    for kind in &radical_kinds {
        tmp_str.clear();
        push_radical(&mut tmp_str, kind.0, kind.1);
        writeln!(
            generated_han,
            "Radical::{} => ({}, {}),",
            &tmp_str, kind.0, kind.1,
        )
        .expect("unable to write generated_han.rs");
    }
    generated_han
        .write_all(b"}\n}\n")
        .expect("unable to write generated_han.rs");
    // end of impl Radical
    generated_han
        .write_all(b"}\n")
        .expect("unable to write generated_han.rs");

    generated_han
        .write_all(b"pub(super) static STR_RADICAL: phf::OrderedMap<&'static str, Radical> = phf::phf_ordered_map! {\n")
        .expect("unable to write generated_han.rs");
    for kind in radical_kinds.iter() {
        tmp_str.clear();
        tmp_str.push('\"');
        tmp_str.push_str(itoa_buf.format(kind.0));
        (0..kind.1).for_each(|_| tmp_str.push('\''));
        tmp_str.push_str("\" => Radical::");
        push_radical(&mut tmp_str, kind.0, kind.1);
        tmp_str.push_str(",\n");
        generated_han
            .write_all(tmp_str.as_bytes())
            .expect("unable to write generated_han.rs");
    }
    generated_han
        .write_all(b"};\n")
        .expect("unable to write generated_han.rs");

    generated_han
        .write_all(b"pub(super) static KX_IDEOS: &[(char, char)] = &[\n")
        .expect("unable to write generated_han.rs");
    for &(kx, ideo) in &kx_ideo {
        writeln!(
            generated_han,
            "('\\u{{{:04X}}}', '\\u{{{:04X}}}'),",
            kx.map(|v| v as u32).unwrap_or(0),
            ideo as u32
        )
        .expect("unable to write generated_han.rs");
    }
    generated_han
        .write_all(b"];\n")
        .expect("unable to write generated_han.rs");
}

fn write_cjk_info(generated_han: &mut impl Write) {
    #[derive(Clone, Copy, Default)]
    struct CjkStroke {
        radical: (u8, u8),
        addi_strokes: i8,
        k_total_strokes: u8,
        glyph_total_strokes: u8,
        extra: CjkExtra,
    }

    #[bitfield]
    #[derive(Clone, Copy, Default)]
    struct CjkExtra {
        mandarin: B10,
        mandarin_tone: B3,
        cantonese: B11,
        cantonese_tone: B3,
        #[skip]
        extra: B5,
    }
    let _ = CjkExtra::new(); // make lint happy!

    let mut cjk = [
        vec![CjkStroke::default(); 0x4DBF - 0x3400 + 1], // 3400..=4DBF; Ext A
        vec![CjkStroke::default(); 0x9FFF - 0x4E00 + 1], // 4E00..=9FFF; Uni
        vec![CjkStroke::default(); 0xFAFF - 0xF900 + 1], // F900..=FAFF; Comp
        // 20000..=2A6DF; Ext B
        vec![CjkStroke::default(); 0x2A6DF - 0x20000 + 1],
        // 2A700..=2B73F;2B740..=2B81F;2B820..=2CEAF;2CEB0..=2EBEF;2EBF0..=2EE5F; C,D,E,F,I
        vec![CjkStroke::default(); 0x2EE5F - 0x2A700 + 1],
        vec![CjkStroke::default(); 0x2FA1F - 0x2F800 + 1], // 2F800..=2FA1F; Comp Supp
        // 30000..=3134F;31350..=323AF;323B0..=3347F; Ext G,H,J
        vec![CjkStroke::default(); 0x3347F - 0x30000 + 1],
    ];

    let strokes = read_to_string("UniData/Unihan_IRGSources.txt")
        .expect("unable to read Unihan_IRGSources.txt");
    for line in strokes.lines() {
        let line = line.trim_ascii();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut entry = line.split("\t");
        let slot = entry.next().expect("invalid Unihan_IRGSources Data");
        let slot = u32::from_str_radix(&slot[2..], 16).expect("invalid Unihan_IRGSources Data");
        let field = entry.next().expect("invalid Unihan_IRGSources Data");
        if field.eq("kRSUnicode") {
            let mut rs = entry
                .next()
                .expect("invalid Unihan_IRGSources Data")
                .split(|b| b == '.' || b == ' ');
            let kind = rs.next().expect("invalid Unihan_IRGSources Data");
            let kind = match kind.bytes().position(|b| b == b'\'') {
                Some(i) => (
                    u8::from_str_radix(&kind[..i], 10).expect("illegal CJKRadicals data"),
                    (kind.len() - i) as u8,
                ),
                None => (
                    u8::from_str_radix(kind, 10).expect("illegal CJKRadicals data"),
                    0,
                ),
            };
            let addi = rs
                .next()
                .and_then(|s| i8::from_str_radix(s, 10).ok())
                .expect("invalid Unihan_IRGSources Data");
            vec_item_of(&mut cjk, slot).radical = kind;
            vec_item_of(&mut cjk, slot).addi_strokes = addi;
        } else if field.eq("kTotalStrokes") {
            let total = entry
                .next()
                .and_then(|s| u8::from_str_radix(s, 10).ok())
                .expect("invalid Unihan_IRGSources Data");
            vec_item_of(&mut cjk, slot).k_total_strokes = total;
        }
    }

    let ts = read_to_string("UniData/ts.txt").expect("unable to read ts.txt");
    for line in ts.lines() {
        let line = line.trim_ascii();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut entry = line.split("\t");
        let c = {
            let mut cs = entry.next().expect("invalid ts.txt Data").chars();
            let c = cs.next().expect("invalid ts.txt Data");
            if cs.next().is_some() {
                panic!("invalid ts.txt Data");
            }
            c
        };
        let stroke_n = entry
            .next()
            .expect("invalid ts.txt Data")
            .split(|c: char| !c.is_ascii_digit())
            .next()
            .expect("invalid ts.txt Data");
        let n = u8::from_str_radix(stroke_n, 10).expect("invalid ts.txt Data");
        vec_item_of(&mut cjk, c as u32).glyph_total_strokes = n;
    }

    let readings =
        read_to_string("UniData/Unihan_Readings.txt").expect("unable to read Unihan_Readings.txt");
    let mut mandarin_getter = MandarinReader::new();
    let mut cantonese: BTreeSet<&str> = BTreeSet::new();
    cantonese.insert("");
    let mut mandarin: BTreeSet<Rc<String>> = BTreeSet::new();
    mandarin.insert(Rc::new(String::new()));
    for line in readings.lines() {
        let line = line.trim_ascii();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut entry = line.split("\t");
        let _slot = entry.next().expect("invalid Unihan_Readings Data");
        // let slot = u32::from_str_radix(&slot[2..], 16).expect("invalid Unihan_Readings Data");
        let field = entry.next().expect("invalid Unihan_Readings Data");
        if field.eq("kCantonese") {
            let reading = entry.next().expect("invalid Unihan_Readings Data");
            let r = if reading.ends_with(|c| matches!(c, '1'..='6')) {
                let r = &reading[..reading.len() - 1];
                assert!(r.bytes().all(|b| b.is_ascii_alphabetic()));
                r
            } else {
                assert!(reading.bytes().all(|b| b.is_ascii_alphabetic()));
                reading
            };
            if !cantonese.contains(r) {
                cantonese.insert(r);
            }
        } else if field.eq("kMandarin") {
            let mut reading = entry
                .next()
                .expect("invalid Unihan_Readings Data")
                .split(" ");
            let simp = reading.next().expect("invalid Unihan_Readings Data");
            let (n_simp, _) = mandarin_getter.get_madarin_reading(simp);
            if !mandarin.contains(&n_simp) {
                mandarin.insert(n_simp);
            }
        }
    }

    let cantonese = Vec::from_iter(cantonese);
    let mandarin = Vec::from_iter(mandarin);
    for line in readings.lines() {
        let line = line.trim_ascii();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut entry = line.split("\t");
        let slot = entry.next().expect("invalid Unihan_Readings Data");
        let slot = u32::from_str_radix(&slot[2..], 16).expect("invalid Unihan_Readings Data");
        let field = entry.next().expect("invalid Unihan_Readings Data");
        let data_extra = &mut vec_item_of(&mut cjk, slot).extra;
        if field.eq("kCantonese") {
            let reading = entry.next().expect("invalid Unihan_Readings Data");
            let (r, tone) = if reading.ends_with(|c| matches!(c, '1'..='6')) {
                let r = &reading[..reading.len() - 1];
                assert!(r.bytes().all(|b| b.is_ascii_alphabetic()));
                (r, reading.as_bytes().last().unwrap() - b'0')
            } else {
                assert!(reading.bytes().all(|b| b.is_ascii_alphabetic()));
                (reading, 0)
            };
            data_extra
                .set_cantonese_checked(cantonese.binary_search(&r).unwrap() as _)
                .unwrap();
            data_extra.set_cantonese_tone_checked(tone).unwrap();
        } else if field.eq("kMandarin") {
            let mut reading = entry
                .next()
                .expect("invalid Unihan_Readings Data")
                .split(" ");
            let simp = reading.next().expect("invalid Unihan_Readings Data");
            let (n_simp, tone) = mandarin_getter.get_madarin_reading(simp);
            data_extra
                .set_mandarin_checked(mandarin.binary_search(&n_simp).unwrap() as _)
                .unwrap();
            data_extra.set_mandarin_tone_checked(tone).unwrap();
        }
    }

    generated_han
        .write_all(
            b"pub(super) static CANTONESE_READINGS: phf::OrderedSet<&'static str> = phf::phf_ordered_set! [\n",
        )
        .expect("unable to write generated_han.rs");
    for reading in cantonese.iter() {
        generated_han
            .write_all(format!("\"{}\",\n", reading).as_bytes())
            .expect("unable to write generated_han.rs");
    }
    generated_han
        .write_all(b"];\n")
        .expect("unable to write generated_han.rs");
    generated_han
        .write_all(
            b"pub(super) static MANDARIN_READINGS: phf::OrderedSet<&'static str> = phf::phf_ordered_set! [\n",
        )
        .expect("unable to write generated_han.rs");
    for reading in mandarin.iter() {
        generated_han
            .write_all(format!("\"{}\",\n", reading).as_bytes())
            .expect("unable to write generated_han.rs");
    }
    generated_han
        .write_all(b"];\n")
        .expect("unable to write generated_han.rs");

    let mut rad_map: HashMap<(u8, u8), String> = HashMap::new();
    for (data, name) in cjk.iter().zip(NAMES) {
        writeln!(
            generated_han,
            "pub(super) static INFO_{}: &[Option<CjkInfo>] = &[",
            name
        )
        .expect("unable to write generated_han.rs");
        for entry in data {
            let s = if entry.radical == (0, 0) {
                String::from("None,\n")
            } else {
                let rad = match rad_map.get(&entry.radical) {
                    Some(v) => v,
                    None => {
                        let mut buf = itoa::Buffer::new();
                        let mut s = format!("R{}", buf.format(entry.radical.0));
                        if entry.radical.1 > 0 {
                            s.push('_');
                            s.push_str(buf.format(entry.radical.1));
                        }
                        rad_map.insert(entry.radical, s);
                        rad_map.get(&entry.radical).unwrap()
                    }
                };
                format!(
                    "Some(CjkInfo::new(Radical::{}, {}, {}, {}, {}, {}, {}, {}, {})),\n",
                    rad,
                    entry.addi_strokes,
                    entry.k_total_strokes,
                    entry.glyph_total_strokes,
                    entry.extra.mandarin(),
                    entry.extra.mandarin_tone(),
                    entry.extra.cantonese(),
                    entry.extra.cantonese_tone(),
                    0,
                )
            };
            generated_han
                .write_all(s.as_bytes())
                .expect("unable to write generated_han.rs");
        }
        generated_han
            .write_all("];\n".as_bytes())
            .expect("unable to write generated_han.rs");
    }
}

fn write_han_order(han_stroke_order: &mut impl Write) {
    let orders = read_to_string("UniData/StrokeOrder.txt").expect("unable to read StrokeOrder.txt");

    let mut cjk: [Vec<u8>; 7] = [
        vec![], // 3400..=4DBF; Ext A
        vec![], // 4E00..=9FFF; Uni
        vec![], // F900..=FAFF; Comp
        // 20000..=2A6DF; Ext B
        vec![],
        // 2A700..=2B73F;2B740..=2B81F;2B820..=2CEAF;2CEB0..=2EBEF;2EBF0..=2EE5F; C,D,E,F,I
        vec![],
        vec![], // 2F800..=2FA1F; Comp Supp
        // 30000..=3134F;31350..=323AF;323B0..=3347F; Ext G,H,J
        vec![],
    ];
    let mut stroke_indices = [
        vec![0usize; 0x4DBF - 0x3400 + 1 + 1],   // Ext A
        vec![0usize; 0x9FFF - 0x4E00 + 1 + 1],   // Uni
        vec![0usize; 0xFAFF - 0xF900 + 1 + 1],   // Comp
        vec![0usize; 0x2A6DF - 0x20000 + 1 + 1], // Ext B
        vec![0usize; 0x2EBF0 - 0x2A700 + 1 + 1], // C,D,E,F,I
        vec![0usize; 0x2FA1F - 0x2F800 + 1 + 1], // Comp Supp
        vec![0usize; 0x3347F - 0x30000 + 1 + 1], // Ext G,H,J
    ];
    let mut indices = [0usize; 7];

    for line in orders.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut entry = line.split("\t");
        let slot = entry.next().expect("invalid StrokeOrder Data");
        let slot = u32::from_str_radix(&slot[2..], 16).expect("invalid StrokeOrder Data");
        let full_stroke = entry.next().expect("invalid StrokeOrder Data");
        assert!(
            full_stroke.bytes().all(|b| matches!(b, b'1'..=b'5')),
            "invalid StrokeOrder Data: U+{:04X}",
            slot
        );

        for bi_stroke in full_stroke.as_bytes().chunks(2) {
            let first = bi_stroke[0] - b'0';
            let second = bi_stroke.get(1).copied().unwrap_or(b'0') - b'0';
            data_of(&mut cjk, slot).push((first << 4) | second);
        }
        let index = data_of(&mut indices, slot);
        *vec_item_of(&mut stroke_indices, slot) = *index;
        *index += (full_stroke.len() + 1) / 2;
    }

    for (si, i) in stroke_indices.iter_mut().zip(indices) {
        for ii in si.iter_mut().rev() {
            if *ii == 0 {
                *ii = i;
            } else {
                break;
            }
        }
    }

    for (data, name) in cjk.iter().zip(NAMES) {
        let mut tmp_str = String::new();
        writeln!(
            han_stroke_order,
            "pub(super) static ORDER_{}: &[u8] = &[",
            name
        )
        .expect("unable to write han_stroke_order.rs");
        for bi_stroke in data.chunks(16) {
            tmp_str.clear();
            for b in bi_stroke {
                tmp_str.push_str(&format!("0x{:02X},", b));
            }
            tmp_str.push('\n');
            han_stroke_order
                .write_all(tmp_str.as_bytes())
                .expect("unable to write han_stroke_order.rs");
        }
        han_stroke_order
            .write_all(b"];\n")
            .expect("unable to write han_stroke_order.rs");
    }

    for (data, name) in stroke_indices.iter().zip(NAMES) {
        let mut itoa_buf = itoa::Buffer::new();
        writeln!(
            han_stroke_order,
            "pub(super) static ORDER_IDX_{}: &[u{}] = &[",
            name,
            num_bit_len(*data.last().unwrap())
        )
        .expect("unable to write han_stroke_order.rs");
        for i in data {
            writeln!(han_stroke_order, "{},", itoa_buf.format(*i))
                .expect("unable to write han_stroke_order.rs");
        }
        han_stroke_order
            .write_all(b"];\n")
            .expect("unable to write han_stroke_order.rs");
    }
}

fn num_bit_len(n: usize) -> i32 {
    let lz = n.leading_zeros();
    let ge32 = (lz >= 32) as usize;
    let ge48 = (lz >= 48) as usize;
    let ge56 = (lz >= 56) as usize;
    let shift = 3 - ge32 - ge48 - ge56;
    8 << shift
}

const NAMES: [&str; 7] = [
    "CJK_EXT_A",
    "CJK_UNI",
    "CJK_COMP",
    "CJK_EXT_B",
    "CJK_EXT_CDEFI",
    "CJK_COMP_SUPP",
    "CJK_EXT_GHJ",
];

fn data_of<T>(data: &mut [T; 7], slot: u32) -> &mut T {
    match slot {
        0x3400..=0x4DBF => &mut data[0],
        0x4E00..=0x9FFF => &mut data[1],
        0xF900..=0xFAFF => &mut data[2],
        0x20000..=0x2A6DF => &mut data[3],
        0x2A700..=0x2EBF0 => &mut data[4],
        0x2F800..=0x2FA1F => &mut data[5],
        0x30000..=0x3347F => &mut data[6],
        _ => panic!("invalid Han slot: {:04X}", slot),
    }
}

fn vec_item_of<I>(cjk: &mut [Vec<I>; 7], slot: u32) -> &mut I {
    match slot {
        0x3400..=0x4DBF => &mut cjk[0][(slot - 0x3400) as usize],
        0x4E00..=0x9FFF => &mut cjk[1][(slot - 0x4E00) as usize],
        0xF900..=0xFAFF => &mut cjk[2][(slot - 0xF900) as usize],
        0x20000..=0x2A6DF => &mut cjk[3][(slot - 0x20000) as usize],
        0x2A700..=0x2B73F => &mut cjk[4][(slot - 0x2A700) as usize],
        0x2B740..=0x2B81F => &mut cjk[4][(slot - 0x2A700) as usize],
        0x2B820..=0x2CEAF => &mut cjk[4][(slot - 0x2A700) as usize],
        0x2CEB0..=0x2EBEF => &mut cjk[4][(slot - 0x2A700) as usize],
        0x2EBF0..=0x2EE5F => &mut cjk[4][(slot - 0x2A700) as usize],
        0x2F800..=0x2FA1F => &mut cjk[5][(slot - 0x2F800) as usize],
        0x30000..=0x3134F => &mut cjk[6][(slot - 0x30000) as usize],
        0x31350..=0x323AF => &mut cjk[6][(slot - 0x30000) as usize],
        0x323B0..=0x3347F => &mut cjk[6][(slot - 0x30000) as usize],
        _ => panic!("invalid Han slot: U+{:04X}", slot),
    }
}

struct MandarinReader<'m> {
    madarin_nfd: HashMap<&'m str, (Rc<String>, u8)>,
    nfc: icu_normalizer::ComposingNormalizerBorrowed<'static>,
    nfd: icu_normalizer::DecomposingNormalizerBorrowed<'static>,
}

impl<'m> MandarinReader<'m> {
    fn new() -> Self {
        Self {
            madarin_nfd: HashMap::new(),
            nfc: icu_normalizer::ComposingNormalizerBorrowed::new_nfc(),
            nfd: icu_normalizer::DecomposingNormalizerBorrowed::new_nfd(),
        }
    }

    fn get_madarin_reading<'s: 'm>(&mut self, reading: &'s str) -> (Rc<String>, u8) {
        match self.madarin_nfd.entry(reading) {
            std::collections::hash_map::Entry::Occupied(e) => {
                let v = e.into_mut();
                (v.0.clone(), v.1)
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                let nfd_reading = self.nfd.normalize(reading).to_string();
                let (base, tone) = match nfd_reading
                    .chars()
                    .position(|c| matches!(c, '\u{300}' | '\u{301}' | '\u{304}' | '\u{30C}'))
                {
                    Some(i) => {
                        let mut base = String::with_capacity(nfd_reading.len());
                        let mut chars = nfd_reading.chars();
                        for _ in 0..i {
                            base.push(chars.next().unwrap());
                        }
                        let tone_char = chars.next().unwrap();
                        let tone = match tone_char {
                            '\u{300}' => 4,
                            '\u{301}' => 2,
                            '\u{304}' => 1,
                            '\u{30C}' => 3,
                            _ => unreachable!(),
                        };
                        base.extend(chars);
                        (base, tone)
                    }
                    None => (nfd_reading, 0),
                };
                let normalized_base = self.nfc.normalize(&base).to_string();
                let v = e.insert((Rc::new(normalized_base), tone));
                (v.0.clone(), v.1)
            }
        }
    }
}
