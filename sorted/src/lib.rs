use proc_macro::TokenStream;

#[path ="./01.rs"]
mod _01;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    _01::token_stream(args, input)
}
