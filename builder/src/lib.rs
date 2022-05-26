use proc_macro::TokenStream;

#[allow(dead_code)]
#[path = "./01.rs"]
mod _01;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    _01::token_stream(input)
}

