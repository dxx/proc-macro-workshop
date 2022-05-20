use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error, Ident, spanned::Spanned, parse_macro_input};

pub fn token_stream(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    let builder_stream = generate_builder(&derive_input);
    let impl_stream = generate_impl(&derive_input);
    let stream = quote!{
        #builder_stream
        #impl_stream
    };
    stream.into()
}

// #[proc_macro_derive(Builder)]
// pub fn derive(input: TokenStream) -> TokenStream {
//     let derive_input = parse_macro_input!(input as DeriveInput);

//     let builder_stream = generate_builder(&derive_input);
//     let impl_stream = generate_impl(&derive_input);
//     let stream = quote!{
//         #builder_stream
//         #impl_stream
//     };
//     stream.into()
// }

/// Generate builder struct.
/// pub struct CommandBuilder { ... }.
fn generate_builder(input: &DeriveInput) -> proc_macro2::TokenStream {
    let vis = &input.vis;
    let builder_name = format!("{}Builder", input.ident.to_string());
    let builder_ident = Ident::new(&builder_name, input.span());

    let optional_field_stream = parse_optional_fields(input, false);
    if let Err(err) = optional_field_stream {
        return err.into_compile_error();
    }
    let optional_field_stream = optional_field_stream.unwrap();

    quote! {
        #vis struct #builder_ident {
            #(#optional_field_stream),*
        }
    }
}

/// Generate impl of input struct.
/// impl Command { ... }.
fn generate_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    let ident = &input.ident;
    let builder_name = format!("{}Builder", input.ident.to_string());
    let builder_ident = Ident::new(&builder_name, input.span());
    let builder_method_stream = generate_builder_method(input, &builder_ident);
    quote! {
        impl #ident {
            #builder_method_stream
        }
    }
}

///  pub fn builder() -> CommandBuilder {
///      CommandBuilder {
///          executable: None,
///          args: None,
///          env: None,
///          current_dir: None,
///      }
///  }
fn generate_builder_method(input: &DeriveInput, builder_ident: &Ident) -> proc_macro2::TokenStream {
    let optional_field_stream = parse_optional_fields(input, true);
    if let Err(err) = optional_field_stream {
        return err.into_compile_error();
    }
    let optional_field_stream = optional_field_stream.unwrap();
    quote! {
        pub fn builder() -> #builder_ident {
            #builder_ident{
                #(#optional_field_stream),*
            }
        }
    }
}

/// Parse field list.
/// 
/// if init is true:
/// executable: None,
//  args: None,
//  env: None,
//  current_dir: None,
/// 
/// if init is false:
/// executable: Option<String>
//  args: Option<Vec<String>>
//  env: Option<Vec<String>>
//  current_dir: Option<String>
fn parse_optional_fields(
    input: &DeriveInput,
    init: bool
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    parse_fields(input, |field| {
        let vis = &field.vis;
        let ident = &field.ident;
        let ty = &field.ty;
        let right = if init { quote!(None) } else { quote!(Option<#ty>)};
        quote! {
            #vis #ident: #right
        }
    })
}

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
                return Err(Error::new_spanned(&fields, "unexpected fields"))
            }
        }
    };
    Err(Error::new_spanned(&input, "unexpected derive input"))
}
