use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, Item};

pub fn token_stream(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);
    if let Err(err) = parse_item(&item) {
        let mut err_token_stream = err.to_compile_error();
        err_token_stream.extend(item.into_token_stream());
        return err_token_stream.into();
    }
    item.to_token_stream().into()
}

fn parse_item(item: &Item) -> syn::Result<()> {
    if let syn::Item::Enum(_) = item {
        return Ok(());
    }
    Err(syn::Error::new(
        proc_macro2::Span::call_site(), "expected enum or match expression"
    ))
}
