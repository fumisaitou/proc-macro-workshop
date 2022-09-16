extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, FieldsNamed,
    PathSegment, Type, TypePath, Path, ext};

// コンパイラとコード間ではproc_macroのTokenStream
// コード内ではproc_macro2のTokenStream

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    // 構造体とbuild用の名前をゲット
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;
    let builder_name = format_ident!("{}Builder", struct_name);

    // 構造体のフィールドの宣言と初期化
    let fields = builder_fields(&input.data);
    let fields_init = builder_fields_init(&input.data);

    // setters?build?、まだよく分かってない
    // let setters = builder_setters(&input.data);
    // let build = builder_build(&input.data, &struct_name);

    // Build the output, possibly using quasi-quotation
    let expanded = quote!{
        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #fields_init
                }
            }
        }
    };

    // Hand the output tokens back to the compiler
    proc_macro::TokenStream::from(expanded)
}

fn builder_fields(data: &Data) -> TokenStream {
    let fields = extract_fields(data);
    let option_wrapped = fields.named.iter().map(|f| {
        let ty = &f.ty;
        let ident = &f.ident;
        if type_is_option(ty) {
            quote!{
                #ident: #ty
            }
        } else {
            quote!{
                #ident: std::option::Option<#ty>
            }
        }
    });
    quote!{
        #(#option_wrapped),*  // まじ意味わからん
    }
}

fn builder_fields_init(data: &Data) -> TokenStream{
    let fields = extract_fields(data);
    let value_none = fields.named.iter().map(|f| {
        let ident = &f.ident;
        quote!{
            #ident:
        }
    })
}


// ---------------------------------------------------


fn extract_fields(data: &Data) -> &FieldsNamed {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields,
            _ => panic!("all fields must be named."),
        },
        _ => panic!("struct expected, but got other item."),
    }
}

fn type_is_option(ty: &Type) -> bool {
    type_is_contained_by(ty, "Option")
}

fn type_is_contained_by<T: AsRef<str>>(ty: &Type, container_type: T) -> bool {
    let container_type = container_type.as_ref();
    extract_last_path_segment(ty)
        .map(|path_seg| path_seg.ident == container_type)
        .unwrap_or(false)
}

fn extract_last_path_segment(ty: &Type) -> Option<&PathSegment> {
    match ty {
        &Type::Path(TypePath {
            qself: _,
            path: Path {
                segments: ref seg,
                leading_colon: _,
            },
        }) => seg.last(),
        _ => None,
    }
}