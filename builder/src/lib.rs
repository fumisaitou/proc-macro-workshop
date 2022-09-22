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

    let checks = idents.iter().map(|ident| {
        let err = format!("Required fiels '{}' is missing", ident.to_string());
        quote!{
            if self.#ident.is_none() {
                return Err(#err.into())
            }
        }
    });


    let expanded = quote!{
        #struct_vis struct #builder_name {
            #(#idents: Option<#types>),*
        }

        impl #builder_name {
            #(fn #idents(&mut self, #idents: #types) -> &mut Self {
                self.#idents = Some(#idents);
                self
            })*

            pub fn build(&mut self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                #(#checks)*
                Ok(#struct_name {
                    #(#idents: self.#idents.clone().unwrap()),*
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