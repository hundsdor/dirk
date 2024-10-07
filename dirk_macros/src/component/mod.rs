use proc_macro::TokenStream;

use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Bracket, Comma},
};

use crate::errors::InfallibleResult;

use self::{
    binding::Binding,
    error::{ComponentResult, ComponentSyntaxError},
    processor::{ComponentMacroData, ComponentMacroProcessor, InfallibleComponentMacroProcessor},
};

mod error;
mod processor;
mod syntax;

mod binding;

pub(crate) fn _macro(
    attr: TokenStream,
    item: TokenStream,
) -> InfallibleResult<TokenStream, ComponentSyntaxError> {
    let data = ComponentMacroData::new(attr, item);
    let processor = InfallibleComponentMacroProcessor::new(&data);

    processor.process().map(|items| {
        let expaned = quote! { #(#items)* };
        TokenStream::from(expaned)
    })
}

pub(crate) fn _macro_helper(attr: TokenStream, item: TokenStream) -> ComponentResult<TokenStream> {
    let data = ComponentMacroData::new(attr, item);
    let processor = ComponentMacroProcessor::new(&data);

    processor.process().map(|items| {
        let expaned = quote! { #(#items)* };
        TokenStream::from(expaned)
    })
}

#[derive(Debug)]
struct ComponentMacroInput {
    _bracket: Bracket,
    bindings: Punctuated<Binding, Comma>,
}

impl Parse for ComponentMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let binds;

        let bracket = bracketed!(binds in input);
        let bindings = binds.parse_terminated(Binding::parse, Comma)?;

        let res = ComponentMacroInput {
            _bracket: bracket,
            bindings,
        };

        Ok(res)
    }
}
