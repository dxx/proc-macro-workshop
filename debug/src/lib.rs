use proc_macro::TokenStream;

#[path = "./01.rs"]
mod _01;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    _01::token_stream(input)
}
