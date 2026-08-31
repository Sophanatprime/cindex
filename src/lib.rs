pub mod ffi;
pub mod han;
mod ikv;
mod out;
mod page;
mod style;

pub use out::{
    MergedEntry, MergedPage, Ordering, get_page_cmp, luaopen_cindex_table, merge_entries,
};
pub use page::{
    IndexPage, IndexPageKind, IteratorPrecedence, Precedence,
    PrecedenceProvider as PagePrecedenceProvider, StringPrecedence, TablePrecedence,
};
pub use style::{IndexEntry, IstFile};
