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

pub(crate) mod error;
pub(crate) mod processor;
mod syntax;

mod binding;

pub(crate) fn _macro(
    data: ComponentMacroData,
) -> InfallibleResult<TokenStream, ComponentSyntaxError> {
    let processor = InfallibleComponentMacroProcessor::new(&data);

    processor.process().map(|items| {
        let expaned = quote! { #(#items)* };
        TokenStream::from(expaned)
    })
}

pub(crate) fn _macro_helper(data: ComponentMacroData) -> ComponentResult<TokenStream> {
    let processor = ComponentMacroProcessor::new(&data);

    processor.process().map(|items| {
        let expaned = quote! { #(#items)* };
        TokenStream::from(expaned)
    })
}

mod kw {
    syn::custom_keyword!(inner);
}

#[derive(Debug)]
struct ComponentMacroInput {
    _bracket: Bracket,
    bindings: Punctuated<Binding, Comma>,
    inner: Option<kw::inner>,
}

impl Parse for ComponentMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let binds;

        let bracket = bracketed!(binds in input);
        let bindings = binds.parse_terminated(Binding::parse, Comma)?;
        let inner = if input.is_empty() {
            None
        } else {
            Some(input.parse::<kw::inner>()?)
        };
        let res = ComponentMacroInput {
            _bracket: bracket,
            bindings,
            inner,
        };

        Ok(res)
    }
}

impl ComponentMacroInput {
    fn inner_marker() -> kw::inner {
        kw::inner {
            span: proc_macro2::Span::call_site(),
        }
    }
}
