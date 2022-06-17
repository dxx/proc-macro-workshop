use proc_macro::TokenStream;

#[allow(dead_code)]
#[path ="./01.rs"]
mod _01;

#[allow(dead_code)]
#[path ="./02.rs"]
mod _02;

#[allow(dead_code)]
#[path ="./03.rs"]
mod _03;

#[allow(dead_code)]
#[path ="./04.rs"]
mod _04;

#[allow(dead_code)]
#[path ="./05.rs"]
mod _05;

#[allow(dead_code)]
#[path ="./06.rs"]
mod _06;

#[allow(dead_code)]
#[path ="./07.rs"]
mod _07;

#[allow(dead_code)]
#[path ="./08.rs"]
mod _08;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    // _01::token_stream(args, input)
    // _02::token_stream(args, input)
    // _03::token_stream(args, input)
    // _04::token_stream(args, input)
    // _05::token_stream(args, input)
    // _06::token_stream(args, input)
    // _07::token_stream(args, input)
    _08::token_stream(args, input)
}

#[proc_macro_attribute]
pub fn check(args: TokenStream, input: TokenStream) -> TokenStream {
    // _05::check_token_stream(args, input)
    // _06::check_token_stream(args, input)
    // _07::check_token_stream(args, input)
    _08::check_token_stream(args, input)
}
