use proc_macro::TokenStream;

#[allow(dead_code)]
#[path = "./01.rs"]
mod _01;

#[allow(dead_code)]
#[path = "./02.rs"]
mod _02;

#[allow(dead_code)]
#[path = "./03.rs"]
mod _03;

#[allow(dead_code)]
#[path = "./04.rs"]
mod _04;

#[allow(dead_code)]
#[path = "./05.rs"]
mod _05;

#[allow(dead_code)]
#[path = "./06.rs"]
mod _06;

#[allow(dead_code)]
#[path = "./07.rs"]
mod _07;

#[allow(dead_code)]
#[path = "./08.rs"]
mod _08;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    // _01::token_stream(input)
    // _02::token_stream(input)
    // _03::token_stream(input)
    // _04::token_stream(input)
    // _05::token_stream(input)
    // _06::token_stream(input)
    // _07::token_stream(input)
    _08::token_stream(input)
}
