extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, FieldsNamed,
    Meta, MetaList, PathSegment, Type, TypePath, Path,
    NestedMeta, Lit, MetaNameValue, Error, PathArguments, 
    AngleBracketedGenericArguments, GenericArgument, Ident};

// test1
// 空のderiveマクロを作る

// test2
// CommandBuilder構造体のインスタンスを返すbuilder関数をCommand構造体に実装
// pub struct CommandBuilder {
//     executable: Option<String>,
//     args: Option<Vec<String>>,
//     env: Option<Vec<String>>,
//     current_dir: Option<String>,
// }

// impl Command {
//     pub fn builder() -> CommandBuilder {
//         CommandBuilder {
//             executable: None,
//             args: None,
//             env: None,
//             current_dir: None,
//         }
//     }
// }

// test3
// CommandBuilder構造体の各フィールドに対するsetterメソッドを作る
//     impl CommandBuilder {
//         fn executable(&mut self, executable: String) -> &mut Self {
//             self.executable = Some(executable);
//             self
//         }

// test4
// Command構造体のインスタンスを返すbuildメソッドをCommandBuilder構造体に実装
//   CommandBuilderの各フィールドがNoneの場合はエラーを返すようにする
//     impl CommandBuilder {
//         pub fn build(&mut self) -> Result<Command, Box<dyn Error>> {
//             ...
//         }
//     }

// test5
// CommandBuilder構造体でメソッドチェーンを使えるように


#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;                              //Command
    let builder_name = format_ident!("{}Builder", struct_name); //CommandBuilder
    let struct_vis = input.vis;

    let (idents, types): (Vec<Ident>, Vec<Type>) = match input.data{
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields
                .named
                .into_iter()
                .map(|field| {
                    let ident = field.ident;
                    let ty = field.ty;
                    (ident.unwrap(), ty)
                })
                .unzip(),
            _ => panic!("no unnamed fields are allowed")
        },
        _ => panic!("expects struct"),
    };

    let checks = idents
        .iter()
        .zip(&types)
        .filter(|(_, ty)| !is_option(ty))
        .map(|(ident, _)| {
            let err = format!("Required fiels '{}' is missing", ident.to_string());
            quote!{
                if self.#ident.is_none() {
                    return Err(#err.into())
                }
            }
        });
    
    let builder_fields = idents
        .iter()
        .zip(&types)
        .map(|(ident, ty)| {
            let t = unwrap_option(ty).unwrap_or(ty);
            quote!{
                #ident: Option<#t>
            }
        });

    let struct_fields = idents
        .iter()
        .zip(&types)
        .map(|(ident, ty)| {
            if is_option(ty) {
                quote!{
                    #ident: self.#ident.clone()
                }
            } else {
                quote!{
                    #ident: self.#ident.clone().unwrap()
                }
            }
        });

    let setters = idents
        .iter()
        .zip(&types)
        .map(|(ident, ty)| {
            let t = unwrap_option(ty).unwrap_or(ty);
            quote!{
                pub fn #ident(&mut self, #ident: #t) -> &mut Self {
                    self.#ident = Some(#ident);
                    self
                }
            }
        });

    let expanded = quote!{
        #struct_vis struct #builder_name {
            #(#builder_fields),*
            //#(#idents: Option<#types>),*
        }

        impl #builder_name {
            #(#setters)*
            // #(fn #idents(&mut self, #idents: #types) -> &mut Self {
            //     self.#idents = Some(#idents);
            //     self
            // })*

            pub fn build(&mut self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                #(#checks)*
                Ok(#struct_name {
                    //#(#idents: self.#idents.clone().unwrap()),*
                    #(#struct_fields),*
                })
            }
        }

        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#idents: std::option::Option::None),*
                }
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}


fn is_option(ty: &Type) -> bool {
    match get_last_path_segment(ty) {
        Some(seg) => seg.ident == "Option",
        _ => false,
    }
}

fn unwrap_option(ty: &Type) -> Option<&Type> {
    if !is_option(ty) {
        return None;
    }
    match get_last_path_segment(ty) {
        Some(seg) => match seg.arguments {
            PathArguments::AngleBracketed(ref args) => {
                args.args.first().and_then(|arg| match arg {
                    &GenericArgument:: Type(ref ty) => Some(ty),
                    _ => None,
                })
            }
            _ => None,
        },
        None => None,
    }
}

fn get_last_path_segment(ty: &Type) -> Option<&PathSegment> {
    match ty {
        Type::Path(path) => path.path.segments.last(),
        _ => None,
    }
}

// テスト6
// リファクタリング手前まで