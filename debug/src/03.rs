use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn token_stream(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let debug_impl_stream = generate_debug_impl(&derive_input);
    match debug_impl_stream {
        Ok(stream) => stream.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

/// Generate debug impl.
fn generate_debug_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &input.ident;
    let name = ident.to_string();
    let fields = parse_fields(&input, |field| {
        let ident = field.ident.as_ref().unwrap();
        let name = ident.to_string();
        if let Some(fmt) = parse_field_attr_val(field, "debug") {
            return quote! {
                builder.field(#name, &format_args!(#fmt, &self.#ident));
            };
        }
        quote! {
            builder.field(#name, &self.#ident);
        }
    });
    let fields = fields?;
    let stream = quote! {
        impl std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut builder = f.debug_struct(#name);
                #(#fields)*
                builder.finish()
            }
        }
    };
    Ok(stream)
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

/// Parse the attributes on the field.
fn parse_field_attr_val(
    field: &syn::Field,
    attr_name: &str,
) -> Option<String> {
    for attr in field.attrs.iter() {
        let syn::Attribute {
            path,
            ..
        } = attr;
        if let Some(syn::PathSegment { ident, ..}) = path.segments.last() {
            // #[debug = "0b{:08b}"].
            if ident.to_string() == attr_name {
                let meta_list = attr.parse_meta();
                if let Ok(syn::Meta::NameValue(name_value)) = &meta_list {
                    // debug = "0b{:08b}".
                    if let syn::Lit::Str(lit) = &name_value.lit {
                        return Some(lit.value());
                    }
                }
            }
        }
    }
    None
}
