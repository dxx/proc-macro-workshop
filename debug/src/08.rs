use std::str::FromStr;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input, parse_quote};

const IMPL_DEBUG_STRUCT: [&str; 1] = ["PhantomData"];

pub fn token_stream(input: TokenStream) -> TokenStream {
    let mut derive_input = parse_macro_input!(input as DeriveInput);
    let debug_impl_stream = generate_debug_impl(&mut derive_input);
    match debug_impl_stream {
        Ok(stream) => stream.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

/// Generate debug impl.
fn generate_debug_impl(input: &mut DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &input.ident;
    let name = ident.to_string();

    let mut new_where_clause: Vec<syn::WherePredicate> = vec![];

    let meta_val = parse_attr_meta_val(&input.attrs, "debug", "bound")?;
    if let Some(val) = meta_val {
        let impl_debug = proc_macro2::TokenStream::from_str(val.as_str()).unwrap();
        new_where_clause.push(parse_quote!(#impl_debug));
    }
    
    let fields = parse_fields(&input, |field| {
        let ident = field.ident.as_ref().unwrap();
        let name = ident.to_string();
        let f_ty = &field.ty;
        
        if let syn::Type::Path(syn::TypePath { path, ..}) = f_ty {
            let ty_ident = &path.segments.last().unwrap().ident;
            if IMPL_DEBUG_STRUCT.contains(&ty_ident.to_string().as_str()) {
                // Add trait bounds.
                // PhantomData<T>: Debug.
                new_where_clause.push(parse_quote!(#f_ty: std::fmt::Debug));
            } else {
                // values: Vec<T::Value>.
                if let Some(syn::PathSegment {
                    ident: _ident,
                    arguments: syn::PathArguments::AngleBracketed(
                        syn::AngleBracketedGenericArguments {
                            args,
                            ..
                        }
                    ),
                }) = &path.segments.last() {
                    if let Some(syn::GenericArgument::Type(
                        syn::Type::Path(
                            syn::TypePath { path, ..}
                        )
                    )) = args.first() {
                        if path.segments.len() > 1 {
                            // The path is `T::Value`.
                            new_where_clause.push(parse_quote!(#path: std::fmt::Debug));
                        }
                    }
                }
            }
        }

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

    let generics = &mut input.generics.clone();

    add_trait_bounds(generics, |type_param| {
        // Filter flag.
        let mut generic_flag  = false;
        let mut type_flag  = false;
        let _ = parse_fields(&input, |field| {
            if let syn::Type::Path(syn::TypePath { path, ..}) = &field.ty {
                if let Some(syn::PathSegment {
                    ident,
                    arguments,
                }) = &path.segments.last() {
                    // T in f: T.
                    if ident.to_string() == type_param.ident.to_string() {
                        type_flag = true;
                    }
                    
                    if IMPL_DEBUG_STRUCT.contains(&ident.to_string().as_str()) {
                        if let syn::PathArguments::AngleBracketed(
                            syn::AngleBracketedGenericArguments {
                                args,
                                ..
                            }
                        ) = arguments {
                            // T in PhantomData<T>.
                            for arg in args {
                                if let syn::GenericArgument::Type(
                                        syn::Type::Path(
                                            syn::TypePath { path, ..}
                                        )
                                    ) = arg {
                                    if let Some(syn::PathSegment { ident, ..}) = path.segments.last() {
                                        if ident.to_string() == type_param.ident.to_string() {
                                            generic_flag = true;
                                        }
                                    }
                                }
                            }
                        }

                    }
                }
            }
            proc_macro2::TokenStream::new()
        });
        return if type_flag {
            false
        } else {
            generic_flag
        }
    });

    let (
        impl_generics,
        ty_generics,
        where_clause
    ) = generics.split_for_impl();
    
    // Init when WhereClause is none.
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
        where_token: <syn::Token![where]>::default(),
        predicates: syn::punctuated::Punctuated::new(),
    });

    where_clause.predicates.extend(new_where_clause);

    let stream = quote! {
        impl #impl_generics std::fmt::Debug for #ident #ty_generics #where_clause {
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
    mut f: impl FnMut(&syn::Field) -> proc_macro2::TokenStream)
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

/// Add a bound `T: std::fmt::Debug` to every type parameter T.
fn add_trait_bounds(generics: &mut syn::Generics, filter: impl Fn(&syn::TypeParam) -> bool) {
    for param in generics.params.iter_mut() {
        if let syn::GenericParam::Type(ref mut type_param) = param {
            // If a generic like Field<T: Trait>, continue the next loop.
            if type_param.bounds.len() > 0 {
                continue;
            }
            // Continue the next loop if true.
            if filter(type_param) {
                continue;
            }
            type_param.bounds.push(parse_quote!(std::fmt::Debug));
        }
    }
}

/// Parse the meta value of attribute.
fn parse_attr_meta_val(
    attrs: &Vec<syn::Attribute>,
    attr_name: &str,
    meta_name: &str,
) -> syn::Result<Option<String>> {
    for attr in attrs {
        let syn::Attribute {
            path,
            ..
        } = attr;
        if let Some(syn::PathSegment { ident, ..}) = path.segments.last() {
            // #[debug(bound = "T::Value: Debug")].
            if ident.to_string() == attr_name {
                let meta_list = attr.parse_meta();
                if let Ok(syn::Meta::List(syn::MetaList {
                    nested, ..
                })) = &meta_list {
                    for nest in nested.iter() {
                        match nest {
                            syn::NestedMeta::Meta(
                                syn::Meta::NameValue(name_value)) => {
                                    let key = name_value.path.segments.last().unwrap().ident.to_string();
                                    // bound = "T::Value: Debug"
                                    if key == meta_name {
                                        if let syn::Lit::Str(lit) = &name_value.lit {
                                            return Ok(Some(lit.value()));
                                        }
                                    }
                            },
                            _ => {}
                        }
                    }
                }
                return Err(syn::Error::new_spanned(meta_list.unwrap(), "expected `debug(bound = \"...\")`"))
            }
        }
    }
    Ok(None)
}
