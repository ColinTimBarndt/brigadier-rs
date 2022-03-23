pub mod arguments;
pub mod command;
pub mod context;
pub mod errors;
mod string_reader;
pub mod suggestion;
pub mod tree;

pub use string_reader::*;

macro_rules! async_fn_type {
    (($($Arg:ty),*) -> $Out:ty) => {
        fn($($Arg),*) -> Pin<Box<dyn Future<Output = $Out>>>
    };
}
pub(crate) use async_fn_type;

pub trait CommandSource: Clone + Sync {}
