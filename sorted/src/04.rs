use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::ToTokens;
use syn::{parse_macro_input, Item, ItemEnum};

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
    if let syn::Item::Enum(item_enum) = item {
        let enum_pairs = get_sorted_pairs(item_enum);
        for i in 0..item_enum.variants.len() {
            let var = &item_enum.variants[i];
            let name = var.ident.to_string();
            let (ref enum_name, ref enum_ident) = enum_pairs[i];
            if !enum_name.eq(&name) {
                return Err(syn::Error::new_spanned(
                    enum_ident, format!("{} should sort before {}", enum_name, name)
                ));
            }
        }

        return Ok(());
    }
    Err(syn::Error::new(
        proc_macro2::Span::call_site(), "expected enum or match expression"
    ))
}

fn get_sorted_pairs(e: &ItemEnum) -> Vec<(String, Ident)> {
    let mut vec: Vec<(String, Ident)> = e.variants
        .iter()
        .map(|var| (var.ident.to_string(), var.ident.clone()))
        .collect();
    vec.sort();
    vec
}
