use proc_macro::TokenStream;

#[allow(dead_code)]
#[path ="./01.rs"]
mod _01;

#[path ="./02.rs"]
mod _02;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    // _01::token_stream(args, input)
    _02::token_stream(args, input)
}
