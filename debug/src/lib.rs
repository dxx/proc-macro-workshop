use proc_macro::TokenStream;

#[allow(dead_code)]
#[path = "./01.rs"]
mod _01;

#[allow(dead_code)]
#[path = "./02.rs"]
mod _02;

#[path = "./03.rs"]
mod _03;


#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    // _01::token_stream(input)
    // _02::token_stream(input)
    _03::token_stream(input)
}
