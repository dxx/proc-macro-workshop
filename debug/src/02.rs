use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn token_stream(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let debug_impl_stream = generate_debug_impl(&derive_input);
    debug_impl_stream.into()
}

/// Generate debug impl.
fn generate_debug_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    let ident = &input.ident;
    let name = ident.to_string();
    let fields = parse_fields(&input, |field| {
        let ident = field.ident.as_ref().unwrap();
        let name = ident.to_string();
        quote! {
            builder.field(#name, &self.#ident);
        }
    });
    if let Err(err) = fields {
        return err.into_compile_error();
    }
    let fields = fields.unwrap();
    quote! {
        impl std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut builder = f.debug_struct(#name);
                #(#fields)*
                builder.finish()
            }
        }
    }
}

/// Parse field of struct. Call f function in iteration.
fn parse_fields(
    input: &DeriveInput,
    f: impl Fn(&syn::Field) -> proc_macro2::TokenStream)
-> syn::Result<Vec<proc_macro2::TokenStream>> {
    if let syn::Data::Struct(syn::DataStruct{fields, ..}) = &input.data {
        match fields {
            syn::Fields::Named(syn::FieldsNamed {named, ..}) => {
                return Ok(named.iter().map(|field| {
                    f(field)
                }).collect());
            },
            _ => {
                return Err(syn::Error::new_spanned(&fields, "unexpected fields"))
            }
        }
    };
    Err(syn::Error::new_spanned(&input, "unexpected derive input"))
}
