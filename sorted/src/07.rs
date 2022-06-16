use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::ToTokens;
use syn::{parse_macro_input, Item, ItemEnum, ItemFn, visit_mut::{self, VisitMut}};

pub fn token_stream(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);
    if let Err(err) = parse_item(&item) {
        let mut err_token_stream = err.to_compile_error();
        err_token_stream.extend(item.into_token_stream());
        return err_token_stream.into();
    }
    item.to_token_stream().into()
}

pub fn check_token_stream(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_fn = parse_macro_input!(input as ItemFn);
    
    let mut check_visitor = CheckVisitor { error: Ok(()) };
    check_visitor.visit_item_fn_mut(&mut item_fn);

    if let Err(err) = check_visitor.error {
        let mut err_token_stream = err.to_compile_error();
        err_token_stream.extend(item_fn.into_token_stream());
        return err_token_stream.into();
    }

    item_fn.to_token_stream().into()
}

fn parse_item(item: &Item) -> syn::Result<()> {
    if let syn::Item::Enum(item_enum) = item {
        let enum_pairs = get_sorted_pairs(item_enum);
        for i in 0..item_enum.variants.len() {
            let var = &item_enum.variants[i];
            let name = var.ident.to_string();
            let (ref enum_name, ref enum_ident) = enum_pairs[i];
            if !enum_name.eq(&name) {
                return Err(syn::Error::new_spanned(
                    enum_ident, format!("{} should sort before {}", enum_name, name)
                ));
            }
        }

        return Ok(());
    }
    Err(syn::Error::new(
        proc_macro2::Span::call_site(), "expected enum or match expression"
    ))
}

struct CheckVisitor {
    error: syn::Result<()>,
}

impl VisitMut for CheckVisitor {
    fn visit_expr_match_mut(&mut self, node: &mut syn::ExprMatch) {
        let mut index = -1;
        for i in 0..node.attrs.len() {
            let ref attr = node.attrs[i];
            if get_path_name(&attr.path) == "sorted" {
                index = i as i32;
                break;
            }
        }
        if index < 0 {
            // Delegate to the default impl to visit nested expressions.
            visit_mut::visit_expr_match_mut(self, node);
            return;
        }
        // remove #[sorted]
        node.attrs.remove(index as usize);

        let arms = &node.arms;
        let match_pairs = get_match_sorted_pairs(arms);
        for i in 0..arms.len() {
            let arm = &arms[i];
            match &arm.pat {
                // Io(e)
                // Error::Io(e)
                syn::Pat::TupleStruct(syn::PatTupleStruct { path, .. }) |
                syn::Pat::Struct(syn::PatStruct { path, .. }) |
                syn::Pat::Path(syn::PatPath { path, .. }) => {   
                    let name = get_path_name(path);
                    let (ref match_name, ref match_ident) = match_pairs[i];
                    if !match_name.eq(&name) {
                        self.error = Err(syn::Error::new_spanned(
                            match_ident, format!("{} should sort before {}", match_name, name)
                        ));
                        return;
                    }
                }
                syn::Pat::Slice(pat_slice) => {
                    if pat_slice.elems.len() == 0 {
                        self.error = Err(syn::Error::new_spanned(
                            pat_slice, "unsupported by #[sorted]"
                        ));
                        return;
                    }
                }
                _ => {}
            }
        }
    }
}

fn get_sorted_pairs(e: &ItemEnum) -> Vec<(String, Ident)> {
    let mut vec: Vec<(String, Ident)> = e.variants
        .iter()
        .map(|var| (var.ident.to_string(), var.ident.clone()))
        .collect();
    vec.sort_by(|a, b| a.0.cmp(&b.0));
    vec
}

fn get_match_sorted_pairs(arms: &Vec<syn::Arm>) -> Vec<(String, syn::Path)> {
    let mut vec: Vec<(String, syn::Path)> = arms
        .iter()
        .filter(|arm| {
            return match arm.pat {
                // Io(e)
                // Error::Io(e)
                syn::Pat::TupleStruct(_) |
                syn::Pat::Struct(_) |
                syn::Pat::Path(_) => true,
                _ => false,
            }
        })
        .map(|arm| {
            return match &arm.pat {
                syn::Pat::TupleStruct(syn::PatTupleStruct { path, .. }) |
                syn::Pat::Struct(syn::PatStruct { path, .. }) |
                syn::Pat::Path(syn::PatPath { path, .. }) => (get_path_name(path), path.clone()),
                _ => unreachable!(),
            }
        })
        .collect();
    vec.sort_by(|a, b| a.0.cmp(&b.0));
    vec
}

fn get_path_name(path: &syn::Path) -> String {
    let mut names = vec![];
    for p in path.segments.iter() {
        names.push(p.ident.to_string())
    }
    names.join("::")
}
