extern crate proc_macro;

use quote::{quote, format_ident};
use syn::{parse_macro_input, Item, FnArg, Type};


#[proc_macro_attribute]
pub fn sorted(_args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ty = parse_macro_input!(input as Item);

    let item_fn = match ty {
        Item::Fn(ref n) => n,
        _ => panic!("function is only allowed."),
    };


    let fn_name = &item_fn.sig.ident.clone();

    // let fn_name_str = format!("{}", fn_name);

    let fn_data = &item_fn.sig;

    let inputs_parse = fn_data
        .inputs
        .iter()
        .map(|var| {
            // 引数の型は明記されなければならない
            match var {
                FnArg::Typed(cap) => {
                    // let _arg_name = match &cap.pat {
                    //     Pat::Ident(name) => name.ident.clone(),
                    //     _ => panic!("Now only Ident pattern"),
                    // };

                    let arg_type = match &*cap.ty {
                        Type::Path(p) => p.path.segments
                            .iter()
                            .last()
                            .map(|seg| {
                                seg.ident.clone()
                            }),
                        _ => panic!("only type pattern path at input"),
                    };

                    let un_arg_type = arg_type.unwrap();
                    quote!{
                        #un_arg_type
                    }
                },
                _ => panic!("Need an explicitly typed input pattern "),
            }
        });

    let inputs_token = quote!{
        #(#inputs_parse) *
    };


    let fn_output_ty = match &fn_data.output {
        syn::ReturnType::Default => panic!("return type is necessary"),
        syn::ReturnType::Type(_, ty) => match &**ty {
            Type::Path(pt) => pt.path.segments
                .iter()
                .last()
                .map(|seg| {
                    seg.ident.clone()
                }),
            _ => panic!("only return type pattern path"),
        },
    };

    // let mut inputs_str = String::from("");
    // for (i, t) in inputs_parse.enumerate() {
    //     if (i == 0) {
    //         inputs_str = format!("{}", t);
    //     } else  {
    //         inputs_str = format!("{} {}", inputs_str, t);
    //     }
    // };

    
    
    let output_token = quote!{fn_output_ty.unwrap()};

    let fn_body = format!("'(extern {} (-> ({}) {}))'", fn_name, inputs_token, output_token);
    // let fn_body = quote!{
    //     "(extern #fn_name (-> (#inputs_token) output_token))"
    // };

    let expanded = quote!{
        const #fn_name: &str = #fn_body;
    };

    // let re = dbg!(expanded);
    // proc_macro::TokenStream::from(re)
    
    proc_macro::TokenStream::from(expanded)

    // proc_macro::TokenStream::from(quote!{})
    

}

// let inputs_parse = fn_data
//         .inputs
//         .iter()
//         .map(|var| {
//             // 引数の型は明記されなければならない
//             match var {
//                 FnArg::Captured(cap) => {
//                     let _arg_name = match &cap.pat {
//                         Pat::Ident(name) => name.ident.clone(),
//                         _ => panic!("Now only Ident pattern"),
//                     };

//                     let arg_type = match &cap.ty {
//                         Type::Path(p) => p.path.segments
//                             .iter()
//                             .last()
//                             .map(|seg| {
//                                 seg.ident.clone()
//                             }),
//                         _ => panic!("only type pattern path at input"),
//                     };

//                     let un_arg_type = arg_type.unwrap();
//                     quote!{
//                         #un_arg_type
//                     }
//                 },
//                 _ => panic!("Need an explicitly typed input pattern "),
//             }
//         });

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