extern crate proc_macro;

use pm2::TokenStream;
use quote::ToTokens;
use syn::{ImplItem, ItemImpl};
use {proc_macro as pm1, proc_macro2 as pm2};

#[proc_macro_attribute]
pub fn ricq_event_converter(
    input: pm1::TokenStream,
    annotated: pm1::TokenStream,
) -> pm1::TokenStream {
    let ast = syn::parse_macro_input!(input as ItemImpl);
    if !annotated.is_empty() {
        return syn::Error::new_spanned(TokenStream::from(annotated), "Unexpected macro argument")
            .to_compile_error()
            .into();
    }
    match expand(ast) {
        Ok(stream) => stream.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

type MacroResult = syn::Result<TokenStream>;

fn expand(ast: ItemImpl) -> MacroResult {
    todo!()
}
