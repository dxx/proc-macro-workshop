use proc_macro::TokenStream;

#[allow(dead_code)]
#[path = "./01.rs"]
mod _01;

#[allow(dead_code)]
#[path = "./02.rs"]
mod _02;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    // _01::token_stream(input)
    _02::token_stream(input)
}
