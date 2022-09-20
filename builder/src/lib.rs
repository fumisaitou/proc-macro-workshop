extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, FieldsNamed,
    Meta, MetaList, PathSegment, Type, TypePath, Path,
    NestedMeta, Lit, MetaNameValue, Error, PathArguments, 
    AngleBracketedGenericArguments, GenericArgument, Ident};

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
    let setters = builder_setters(&input.data);
    let build = builder_build(&input.data, &struct_name);

    // Build the output, possibly using quasi-quotation
    let expanded = quote!{
        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #fields_init
                }
            }
        }

        pub struct #builder_name {
            #fields
        }

        impl #builder_name {
            #setters

            pub fn build(&mut self) -> std::result::Result<#struct_name, std::boxed::Box<dyn std::error::Error>> {
                #build
            }
        }
    };

    // Hand the output tokens back to the compiler
    proc_macro::TokenStream::from(expanded)
}


// DataはStruct, Enum, Unionのenum
fn builder_fields(data: &Data) -> TokenStream {
    let fields = extract_fields(data);

    // FieldsNamedにあるnamed(Punctuated)をiterateして
    // 各要素にまたいで変数名identと型をゲット
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

    // option_wrappedがiterなので各要素を繰り返しで記述してる
    // アスタリスクの前の文字がセパレータとして使われます。
    quote!{
        #(#option_wrapped),*
    }
}

// DataはStruct, Enum, Unionのenum
fn builder_fields_init(data: &Data) -> TokenStream{
    let fields = extract_fields(data);
    let value_none = fields.named.iter().map(|f| {
        let ident = &f.ident;
        quote!{
            #ident: std::option::Option::None
        }
    });
    quote!{
        #(#value_none),*
    }
}


fn builder_setters(data: &Data) -> TokenStream {
    let fields = extract_fields(data);
    let setters = fields.named.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        let attrs = &f.attrs;

        enum AttrParseResult {
            Value(String),
            InvalidKey(Meta),
        };

        let each_lit = attrs.iter().find_map(|attr| match attr.parse_meta() {
            Ok(meta) => match meta {
                Meta::List(MetaList {
                    ref path,
                    paren_token: _,
                    ref nested,
                }) => {
                    path.get_ident().map(|i| i == "builder")?;
                    nested.first().and_then(|nm| match nm {
                        NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                            ref path,
                            eq_token: _,
                            lit: Lit::Str(ref litstr),
                        })) => {
                            if !path.get_ident().map(|i| i == "each").unwrap_or(false) {
                                return Some(AttrParseResult::InvalidKey(meta.clone()));
                            }
                            Some(AttrParseResult::Value(litstr.value()))
                        }
                        _ => None,
                    })
                }
                _ => None,
            },
            _ => None,
        });

        if let Some(AttrParseResult::InvalidKey(ref meta)) = each_lit {
            return Error::new_spanned(meta, "expected `builder(each = \"...\")`")
                .to_compile_error();
        }

        if !type_is_vec(unwrap_option(ty)) && each_lit.is_some() {
            return Error::new_spanned(ty, "barr").to_compile_error();
        }

        // ********************************************
        // line 125
        match each_lit {
            Some(AttrParseResult::Value(ref lit))
                if ident.as_ref().map(|i| i == lit).unwrap_or(false) =>
            {
                let option_vec_unwrapped = unwrap_option_vec(ty);
                quote!{
                    fn #ident(&mut self, #ident: #option_vec_unwrapped) -> &mut Self {
                        match self.#ident {
                            std::option::Option::Some(ref mut v) => v.push(#ident),
                            std::option::Option::None => {
                                self.#ident = std::option::Option::Some(vec![#ident]);
                            }
                        }
                        self
                    }
                }
            }
            Some(AttrParseResult::Value(ref lit)) => {
                let option_vec_unwrapped = unwrap_option_vec(ty);
                let option_unwrapped = unwrap_option(ty);
                let lit = format_ident!("{}",lit);
                quote!{
                    fn #ident(&mut self, #ident: #option_unwrapped) -> &mut Self {
                        self.#ident = std::option::Option::Some(#ident);
                        self
                    }

                    fn #lit(&mut self, #lit: #option_vec_unwrapped) -> &mut Self {
                        match self.#ident {
                            std::option::Option::Some(ref mut v) => v.push(#lit),
                            std::option::Option::None => {
                                self.#ident = std::option::Option::Some(vec![#lit]);
                            }
                        }
                        self
                    }
                }
            }
            Some(AttrParseResult::InvalidKey(_)) => unreachable!(),
            None => {
                let option_unwrapped = unwrap_option(ty);
                quote!{
                    fn #ident(&mut self, #ident: #option_unwrapped) -> &mut Self {
                        self.#ident = std::option::Option::Some(#ident);
                        self
                    }
                }
                
            }
        }
    });
    quote!{
        #(#setters),*
    }
}

fn builder_build(data: &Data, struct_name: &Ident) -> TokenStream {
    let fields = extract_fields(data);
    let set_check = fields.named.iter().filter_map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        if type_is_option(ty) || type_is_vec(ty) {
            return None;
        }
        let err = format!("field `{}` is not set.", ident.as_ref().unwrap());
        Some(quote!{
            if self.#ident.is_none() {
                return Err(#err.into())
            }
        })
    });
    let build_values = fields.named.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        if type_is_option(ty) {
            quote!{
                #ident: self.#ident.clone()
            }
        } else if type_is_vec(ty) {
            quote!{
                #ident: self.#ident.clone().unwrap_or_else(std::vec::Vec::new)
            }
        } else {
            quote!{
                #ident: self.#ident.clone().unwrap()
            }
        }
    });

    quote!{
            #(#set_check)*

            Ok(#struct_name {
                #(#build_values),*
            })
    }
}

// ---------------------------------------------------

// DataからStructのみを抽出
// Structのfields(Fields)からNamed(FieldsNamed)をゲット
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

fn type_is_vec(ty: &Type) -> bool {
    type_is_contained_by(ty, "Vec")
}

fn type_is_contained_by<T: AsRef<str>>(ty: &Type, container_type: T) -> bool {
    let container_type = container_type.as_ref();
    extract_last_path_segment(ty)
        .map(|path_seg| path_seg.ident == container_type)
        .unwrap_or(false)
}

fn unwrap_option(ty: &Type) -> &Type {
    unwrap_generic_type(ty, "Option")
}

fn unwrap_vec(ty: &Type) -> &Type {
    unwrap_generic_type(ty, "Vec")
}

fn unwrap_option_vec(ty: &Type) -> &Type {
    unwrap_vec(unwrap_option(ty))
}

fn unwrap_generic_type<T: AsRef<str>>(ty: &Type, container_type: T) -> &Type {
    let container_type = container_type.as_ref();
    extract_last_path_segment(ty)
        .and_then(|path_seg| {
            if path_seg.ident != container_type {
                return None;
            }
            match path_seg.arguments {
                PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    colon2_token: _,
                    lt_token: _,
                    ref args,
                    gt_token,
                }) => args.first().and_then(|a| match a {
                    &GenericArgument::Type(ref inner_ty) => Some(inner_ty),
                    _ => None,
                }),
                _ => None,
            }
        })
        .unwrap_or(ty)
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