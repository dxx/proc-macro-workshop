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

    // There will be no error here if the above runs successfully.
    let setter_stream = generate_setters(input).unwrap();

    let build_method_stream = generate_build_method(input);

    let error_struct_stream = error_struct();
    quote! {
        #vis struct #builder_ident {
            #(#optional_field_stream),*
        }
        impl #builder_ident {
            #(#setter_stream)*

            #build_method_stream
        }

        #error_struct_stream
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

/// Generate setters.
/// 
/// fn executable(&mut self, executable: String) -> &mut Self {
///     self.executable = Some(executable);
///     self
/// }
/// fn args(&mut self, args: Vec<String>) -> &mut Self {
///     self.args = Some(args);
///     self
/// }
/// ...
fn generate_setters(input: &DeriveInput) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    parse_fields(input, |field| {
        let vis = &field.vis;
        let ident = &field.ident;
        let ty = if let Some(ty) = parse_generic_type(field, "Option") {
            ty
        } else { field.ty.clone() };
        let attr = parse_field_attr_val(field, "builder", "each");
        if let Err(err) = attr {
            return err.into_compile_error();
        }
        let mut stream = quote!();
        if let Some(val) = attr.unwrap() {
            let name = Ident::new(&val, field.span());
            let ty = parse_generic_type(field, "Vec");
            if ty.is_none() {
                return syn::Error::new_spanned(field, "field type must be Vec<?>").into_compile_error();
            }
            let ty = ty.unwrap();
            stream.extend([
                quote! {
                    #vis fn #name (&mut self, #ident: #ty) -> &mut Self {
                        if self.#ident.is_none() {
                            self.#ident = Some(vec![]);
                        }
                        self.#ident.as_mut().unwrap().push(#ident);
                        self
                    }
                }
            ]);
        } else {
            stream.extend([quote! {
                #vis fn #ident (&mut self, #ident: #ty) -> &mut Self {
                    self.#ident = Some(#ident);
                    self
                }
            }]);
        }
        stream
    })
}

/// Generate build method.
/// pub fn build(&mut self) -> Result<Command, Box<dyn Error>> {
//     ...
//  }
fn generate_build_method(input: &DeriveInput) -> proc_macro2::TokenStream {
    let struct_ident = &input.ident;
    let check_none_stream = parse_fields(input, |field| {
        let ident = &field.ident;
        let ident_str = ident.as_ref().unwrap().to_string();
        if let Some(_ty) = parse_generic_type(field, "Option") {
            return quote!();
        }
        if let Ok(Some(_)) = parse_field_attr_val(field, "builder", "each") {
            return quote!();
        }
        quote! {
            if self.#ident.is_none() {
                return Err(Box::new(BuildError(format!("{} is none", #ident_str))))
            }
        }
    });
    if let Err(err) = check_none_stream {
        return err.into_compile_error();
    }
    let check_none_stream = check_none_stream.unwrap();

    // There will be no error here if the above runs successfully.
    let assignment_stream = parse_fields(input, |field| {
        let ident = &field.ident;
        if let Some(_ty) = parse_generic_type(field, "Option") {
            return quote! {
                #ident: self.#ident.take()
            };
        }
        if let Ok(Some(_)) = parse_field_attr_val(field, "builder", "each") {
            return quote! {
                #ident: self.#ident.take().unwrap_or_default()
            };
        }
        quote! {
            #ident: self.#ident.take().unwrap()
        }
    }).unwrap();
    
    quote! {
        pub fn build(&mut self) -> Result<#struct_ident, Box<dyn std::error::Error>> {
            #(#check_none_stream)*

            Ok(#struct_ident {
                #(#assignment_stream),*
            })
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
        let right = if init {
            quote!(None)
        } else {
            if let Some(_ty) = parse_generic_type(field, "Option") {
                quote!(#ty)
            } else {
                quote!(Option<#ty>)
            }
        };
        quote! {
            #vis #ident: #right
        }
    })
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
                return Err(Error::new_spanned(&fields, "unexpected fields"))
            }
        }
    };
    Err(Error::new_spanned(&input, "unexpected derive input"))
}

/// Parse generic type of field.
/// pub struct Command {
///     executable: String,
///     args: Vec<String>,
///     env: Vec<String>,
///     current_dir: Option<String>,
/// }
/// If ty is current_dir, return String, if ty is env return String.
fn parse_generic_type(field: &syn::Field, ty: &str) -> Option<syn::Type> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path {
            segments,
            ..
        }, ..
    }) = &field.ty {
        if let Some(syn::PathSegment {
            ident,
            arguments: syn::PathArguments::AngleBracketed(
                syn::AngleBracketedGenericArguments {
                    args,
                    ..
                }
            ), 
        }) = segments.last() {
            if ident.to_string() == ty && args.len() > 0 {
                if let syn::GenericArgument::Type(ty) = args.last().unwrap() {
                    return Some(ty.clone());
                }
            }
        }
    }
    None
}

/// Parse the attributes on the field.
fn parse_field_attr_val(
    field: &syn::Field,
    attr_name: &str,
    meta_name: &str,
) -> syn::Result<Option<String>> {
    for attr in field.attrs.iter() {
        let syn::Attribute {
            path,
            ..
        } = attr;
        if let Some(syn::PathSegment { ident, ..}) = path.segments.last() {
            // #[builder(each = "arg")]
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
                                    // each = "arg"
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
                return Err(syn::Error::new_spanned(meta_list.unwrap(), "expected `builder(each = \"...\")`"))
            }
        }
    }
    Ok(None)
}

/// Custom error struct.
fn error_struct() -> proc_macro2::TokenStream {
    quote! {
        pub struct BuildError(String);
        impl std::error::Error for BuildError {
        }

        impl std::fmt::Display for BuildError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "build {} error", self.0)
            }
        }

        impl std::fmt::Debug for BuildError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple("BuildError").field(&self.0).finish()
            }
        }
    }
}
