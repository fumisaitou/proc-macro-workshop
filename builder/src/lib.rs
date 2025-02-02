extern crate proc_macro;

use proc_macro2::{TokenStream, Span};
use quote::{quote, format_ident};
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, FieldsNamed,
    Meta, MetaList, PathSegment, Type, TypePath, Path,
    NestedMeta, Lit, MetaNameValue, Error, PathArguments, 
    AngleBracketedGenericArguments, GenericArgument, Ident, Visibility};


enum LitOrError {
    Lit(String),
    Error(syn::Error),
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;                              //Command
    let builder_name = format_ident!("{}Builder", struct_name); //CommandBuilder
    let struct_vis = input.vis;

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields,
            _ => panic!("no unnamed fields are allowed."),
        },
        _ => panic!("this macro can be applied only to struct.")
    };

    let builder_struct = build_builder_struct(&fields, &builder_name, &struct_vis);
    let builder_impl = build_builder_impl(&fields, &builder_name, &struct_name);
    let struct_impl = build_struct_impl(&fields, &builder_name, &struct_name);

    let expanded = quote!{
        #builder_struct
        #builder_impl
        #struct_impl
    };

    proc_macro::TokenStream::from(expanded)
}


fn build_builder_struct(
    fields: &FieldsNamed,
    builder_name: &Ident,
    visibility: &Visibility,
) -> TokenStream {
    let struct_fields = fields
        .named
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref();
            let ty = unwrap_option(&field.ty).unwrap_or(&field.ty);
            (ident.unwrap(), ty)
        })
        .map(|(ident, ty)| {
            if is_vector(&ty) {
                quote!{
                    #ident: #ty
                }
            } else {
                quote!{
                    #ident: std::option::Option<#ty>
                }
            }
        });

    quote! {
        #visibility struct #builder_name {
            #(#struct_fields),*
        }
    }
}

fn build_builder_impl(
    fields: &FieldsNamed,
    builder_name: &Ident,
    struct_name: &Ident,
) -> TokenStream {
    let checks = fields
        .named
        .iter()
        .filter(|field| !is_option(&field.ty))
        .filter(|field| !is_vector(&field.ty))
        .map(|field| {
            let ident = field.ident.as_ref();
            let err = format!("Required field '{}' is missing", ident.unwrap().to_string());
            quote! {
                if self.#ident.is_none() {
                    return Err(#err.into());
                }
            }
        });

    let setters = fields.named
        .iter()
        .map(|field| {
            let ident_each_name = field
                .attrs
                .first()
                .map(|attr| match attr.parse_meta() {
                    Ok(Meta::List(list)) => match list.nested.first() {
                        Some(NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                            ref path,
                            eq_token: _,
                            lit: Lit::Str(ref str),
                        }))) => {
                            if let Some(name) = path.segments.first() {
                                if name.ident.to_string() != "each" {
                                    return Some(LitOrError::Error(syn::Error::new_spanned(
                                        list, 
                                        "expected `builder(each = \"...\")`",
                                    )));
                                }
                            }
                            Some(LitOrError::Lit(str.value()))
                        }
                        _ => None,
                    },
                    _ => None,
                })
                .flatten();

            let ident = &field.ident.as_ref();
            let ty = unwrap_option(&field.ty).unwrap_or(&field.ty);
            match ident_each_name {
                Some(LitOrError::Lit(name)) => {
                    let ty_each = unwrap_vector(ty).unwrap();
                    let ident_each = Ident::new(name.as_str(), Span::call_site());
                    if ident.unwrap().to_string() == name {
                        quote!{
                            pub fn #ident_each(&mut self, #ident_each: #ty_each) -> &mut Self {
                                self.#ident.push(#ident_each);
                                self
                            }
                        }
                    } else {
                        quote!{
                            pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                                self.#ident = #ident;
                                self
                            }
                            pub fn #ident_each(&mut self, #ident_each: #ty_each) -> &mut Self {
                                self.#ident.push(#ident_each);
                                self
                            }
                        }
                    }
                },
                Some(LitOrError::Error(err)) => err.to_compile_error().into(),
                None => {
                    if is_vector(&ty) {
                        quote!{
                            pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                                self.#ident = #ident;
                                self
                            }
                        }
                    } else {
                        quote!{
                            pub fn #ident(&mut self, #ident: #ty) -> &mut Self {
                                self.#ident = std::option::Option::Some(#ident);
                                self
                            }
                        }
                    }
                }
            }
        });

    let struct_fields = fields.named.iter().map(|field| {
        let ident = field.ident.as_ref();
        if is_option(&field.ty) || is_vector(&field.ty) {
            quote! {
                #ident: self.#ident.clone()
            }
        } else {
            quote! {
                #ident: self.#ident.clone().unwrap()
            }
        }
    });

    quote! {
        impl #builder_name {
            #(#setters)*

            pub fn build(&mut self) -> std::result::Result<#struct_name, std::boxed::Box<dyn std::error::Error>> {
                #(#checks)*
                Ok(#struct_name {
                    #(#struct_fields),*
                })
            }
        }
    }
}

fn build_struct_impl(
    fields: &FieldsNamed,
    builder_name: &Ident,
    struct_name: &Ident,
) -> TokenStream {
    let field_defaults = fields
        .named
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref();
            let ty = &field.ty;
            if is_vector(&ty) {
                quote!{
                    #ident: Vec::new()
                }
            } else {
                quote!{
                    #ident: None
                }
            }
        });

    quote! {
        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#field_defaults),*
                }
            }
        }
    }
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
    
    unwrap_generic_type(ty)
}

fn is_vector(ty: &Type) -> bool {
    match get_last_path_segment(ty) {
        Some(seg) => seg.ident == "Vec",
        _ => false,
    }
}

fn unwrap_vector(ty: &Type) -> Option<&Type> {
    if !is_vector(ty) {
        return None;
    }

    unwrap_generic_type(ty)
}

fn unwrap_generic_type(ty: &Type) -> Option<&Type> {
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
