mod smart_pointer;

use proc_macro::TokenStream;
use syn::{ItemStruct, parse_macro_input};

use crate::smart_pointer::{Args, expand_smart_pointer};

#[proc_macro_attribute]
pub fn smart_pointer(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    let item = parse_macro_input!(input as ItemStruct);

    match expand_smart_pointer(item, args) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
