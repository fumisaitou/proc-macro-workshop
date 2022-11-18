extern crate proc_macro;

use std::path::Path;

use quote::quote;
use syn::{parse_macro_input, Item, FnArg, GenericArgument, Signature, Type, PathArguments, PathSegment};
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
                let span = get_last_path_segment(&*pat.ty).unwrap().ident.span();
                let arg_type = get_seg(&*pat.ty, &span);
                // let arg_type = match get_last_path_segment(&*pat.ty) {
                //     Some(path) => get_seg(ty, span),
                //     // Some(path) => type_translate(&path.ident.clone()),
                //     None => panic!("only type pattern path at input"),
                // };

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
            Some(path) => type_translate(&path.ident.clone()),
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

fn type_translate(ty: &Ident) -> Option<Ident> {
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

fn type_script(ty: &Type, span: &proc_macro2::Span) -> Vec<Option<Ident>> {
    let mut type_vec: Vec<Option<Ident>> = Vec::new();

    match ty {
        Type::Tuple(tup) => {
            type_vec.push(Some(Ident::new("Tuple", span)));

            let args = tup.elems
                .iter()
                .map(|arg_type| {

                });
        },
        Type::Path(path) => {
            let type_name = &path.path.segments.first().unwrap().ident;
            
            match &path.path.segments.first().unwrap().arguments {
                PathArguments::None => type_translate(&path.path.segments.first().unwrap().ident),
                PathArguments::AngleBracketed(ang) => {
                    type_vec.push(Some(*type_name));

                    for data in ang.args.iter() {
                        match data {
                            GenericArgument::Type(gene_type) => {
                                
                            }
                            _ => panic!("miss at generic"),
                        }
                    }
                }
            }

        }
    }
}

fn get_seg(ty: &Type, span: &proc_macro2::Span) -> Option<Ident> {
    match ty {
        Type::Tuple(tup) => {
            let args = tup.elems
            .iter()
            .map(|arg_type| {
                get_seg(arg_type, span).unwrap()
            });

            let mut whole_args = String::from("");
            for (i, data) in args.enumerate() {
                if i==0 { whole_args = format!("{}", data); }
                else    { whole_args = format!("{} {}", whole_args, data); }
            };
            whole_args = format!("[{}]", whole_args);

            Some(syn::Ident::new(&whole_args, *span))
        },
        Type::Path(path) => {
            let type_name = &path.path.segments.first().unwrap().ident;
            let type_name_str = format!("{}", &type_name);

            match &path.path.segments.first().unwrap().arguments {
                // not generic type (eg BigInt)
                PathArguments::None => type_translate(&path.path.segments.first().unwrap().ident),
                // generic type (vec, option, result)
                PathArguments::AngleBracketed(ang) => {
                    let args = ang.args
                        .iter()
                        .map(|a| match a {
                            GenericArgument::Type(gene_type) => {
                                get_seg(gene_type, span).unwrap()
                            },
                            _ => panic!("miss"),
                        });
                    
                    let mut whole_args = String::from("");
                    for (i, data) in args.enumerate() {
                        if i==0 { whole_args = format!("{}", data); }
                        else    { whole_args = format!("{} {}", whole_args, data); }
                    };
/////////////////////////////////////////////////////////////////////////////////////////////////////////

                    match &*type_name_str {
                        "Vec" => {
                            let vec_str = format!("'({})", whole_args);
                            Some(Ident::new(&vec_str, *span))
                        },
                        "Option" => {
                            let opt_str = "aaaa";
                            Some(Ident::new(opt_str, *span))
                        },
                        "Result" => {
                            let res_str = format!("(Result {})", whole_args);
                            Some(Ident::new(&res_str, *span))
                        }
/////////////////////////////////////////////////////////////////////////////////////////////////////////

                        _ => panic!("no types"),
                    }
                },
                _ => panic!("miss2")
            }
        }

        _ => None,
    }
}