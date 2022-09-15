use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let indent = input.ident;

    // Build the output, possibly using quasi-quotation
    let expanded = quote!{
        
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
