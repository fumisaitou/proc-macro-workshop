extern crate proc_macro;

use quote::quote;
use syn::{parse_macro_input, Item, FnArg, Signature, Type, PathSegment};
use proc_macro2::{Ident, TokenStream};


#[proc_macro_attribute]
pub fn sorted(_args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut out = input.clone();
    
    let ty = parse_macro_input!(input as Item);
    let item_fn = match ty {
        Item::Fn(ref n) => n,
        _ => panic!("function is only allowed."),
    };

    let fn_name = &item_fn.sig.ident.clone();
    let fn_name_cap = {
        let mut temp = format!("{}", fn_name);
        temp = temp.to_ascii_uppercase();
        syn::Ident::new(&temp, proc_macro2::Ident::span(fn_name))
    };

    let fn_data = &item_fn.sig;
    let inputs_parse = inputs_type(fn_data);
    let output_ty = output_type(fn_data);

    let fn_body = format!("(extern {} (-> ({}) {}))", fn_name, inputs_parse, output_ty);
    let expanded = quote!{
        const #fn_name_cap: &str = #fn_body;
    };

    out.extend(proc_macro::TokenStream::from(expanded));
    out
}

fn inputs_type(data: &Signature) -> TokenStream {
    let ret = data.inputs.iter().map(|arg| {
        match arg {
            FnArg::Typed(pat) => {
                let arg_type = match get_last_path_segment(&*pat.ty) {
                    Some(path) => type_check(&path.ident.clone()),
                    None => panic!("only type pattern path at input"),
                };

                if let Some(res) = arg_type {
                    quote!{#res}
                } else {
                    panic!("one of input types is undeclared.")
                }
            },
            _ => panic!("Need an explicitly typed input pattern "),
        }
    });

    quote!{
        #(#ret) *
    }
}

fn output_type(data: &Signature) -> TokenStream {
    let ret = match &data.output {
        syn::ReturnType::Default => panic!("return type is necessary"),
        syn::ReturnType::Type(_, ty) => match get_last_path_segment(&*ty) {
            Some(path) => type_check(&path.ident.clone()),
            None => panic!("only return type pattern path"),
        }
    };

    if let Some(res) = ret {
        quote!{#res}
    } else {
        panic!("output type is undeclared.")
    }
}

fn get_last_path_segment(ty: &Type) -> Option<&PathSegment> {
    match ty {
        Type::Path(path) => path.path.segments.last(),
        _ => None,
    }
}

fn type_check(ty: &Ident) -> Option<Ident> {
    let span = Ident::span(&ty);
    let ty_str = format!("{}", &ty);
    match &*ty_str {
        "BigInt"             => Some(Ident::new("Int", span)),
        "char"               => Some(Ident::new("Char", span)),
        "String" | "&str"    => Some(Ident::new("String", span)),
        "bool"               => Some(Ident::new("Bool", span)),
        _ => None,
    }
}

