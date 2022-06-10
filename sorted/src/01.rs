use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, Item};

pub fn token_stream(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);
    item.to_token_stream().into()
}
