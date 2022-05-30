use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

pub fn token_stream(input: TokenStream) -> TokenStream {
    let _derive_input = parse_macro_input!(input as DeriveInput);
    TokenStream::new()
}
