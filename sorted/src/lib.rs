use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Pair, Ident, Item, ItemConst, ItemFn, FnArg, Pat, Stmt, Type};


#[proc_macro_attribute]
pub fn sorted(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ty = parse_macro_input!(input as Item);

    let item_fn = match ty {
        Item::Fn(ref n) => n,
        _ => panic!("function is only allowed."),
    };


    let fn_name = item_fn.ident.clone();

    let fn_name_str = String::from(stringify!(fn_naem));

    let inputs_parse = &item_fn.decl.inputs
        .iter()
        .map(|var| {
            // 引数の型は明記されなければならない
            match var {
                FnArg::Captured(cap) => {
                    let _arg_name = match &cap.pat {
                        Pat::Ident(name) => name.ident.clone(),
                        _ => panic!("Now only Ident pattern"),
                    };

                    let arg_type = match &cap.ty {
                        Type::Path(p) => p.path.segments
                            .last()
                            .map(|seg| {
                                if let Pair::Punctuated(typ, _) = seg {
                                    typ.ident.clone()
                                } else {
                                    panic!("pair variant error")
                                }
                            }),
                        _ => panic!("only type pattern path"),
                    };
                    arg_type.unwrap()
                },
                _ => panic!("Need an explicitly typed pattern "),
            }
        });

    // proc_macro::TokenStream::from(quote!{})


    // let fn_output_ty = match &item_fn.decl.output {
    //     syn::ReturnType::Default => panic!("return type is necessary"),
    //     syn::ReturnType::Type(_, ty) => match &**ty {
    //         Type::Path(pt) => pt.path.segments
    //             .last()
    //             .map(|seg| {
    //                 if let Pair::Punctuated(typ, _) = seg {
    //                     typ.ident.clone()
    //                 } else {
    //                     panic!("pair variant error")
    //                 }
    //             }),
    //         _ => panic!("only return type pattern path"),
    //     },
    // };

    // proc_macro::TokenStream::from(quote!{})

    // let mut inputs_str = String::from("");
    // for (i, t) in inputs_parse.enumerate() {
    //     if (i == 0) {
    //         inputs_str = format!("{}", t);
    //     } else  {
    //         inputs_str = format!("{} {}", inputs_str, t);
    //     }
    // };

    // let output_str = String::from(stringify!(fn_output_ty.unwrap()));

    // let fn_body = format!("'(extern {} (-> ({}) {}))'", fn_name, inputs_str, output_str);

    // let expanded = quote!{
    //     const #fn_name_str: String = #fn_body;
    // };

    // proc_macro::TokenStream::from(expanded)


    // // dbg!(proc_macro::TokenStream::from(expanded));

}
