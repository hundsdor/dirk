use proc_macro::TokenStream;

use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
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
    let data = data;
    let processor = InfallibleComponentMacroProcessor::new(&data);

    processor.process().map(|items| {
        let expaned = quote! { #(#items)* };
        TokenStream::from(expaned)
    })
}

pub(crate) fn _macro_helper(data: ComponentMacroData) -> ComponentResult<TokenStream> {
    let data = data;
    let processor = ComponentMacroProcessor::new(&data);

    processor.process().map(|items| {
        let expaned = quote! { #(#items)* };
        TokenStream::from(expaned)
    })
}

mod kw {
    syn::custom_keyword!(__inner);
}

#[derive(Debug)]
struct ComponentMacroInput {
    bindings: Punctuated<Binding, Comma>,
    inner: Option<(kw::__inner, Comma)>,
}

impl Parse for ComponentMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let inner = input
            .peek(kw::__inner)
            .then(|| input.parse::<kw::__inner>())
            .map(|r| r.and_then(|kw| input.parse::<Comma>().map(|comma| (kw, comma))))
            .transpose()?;

        let bindings = {
            let mut punctuated = Punctuated::new();

            let mut index = 0;
            loop {
                if input.is_empty() {
                    break;
                }
                let value = Binding::parse(input, index)?;
                punctuated.push_value(value);
                if input.is_empty() {
                    break;
                }
                let punct = input.parse()?;
                punctuated.push_punct(punct);
                index += 1;
            }

            punctuated
        };
        let res = ComponentMacroInput { bindings, inner };

        Ok(res)
    }
}

impl ComponentMacroInput {
    fn inner_marker() -> proc_macro2::TokenStream {
        let kw = kw::__inner {
            span: proc_macro2::Span::call_site(),
        };
        let comma = Comma {
            spans: [Span::call_site()],
        };

        let mut stream: proc_macro2::TokenStream = kw.to_token_stream();
        stream.extend(std::iter::once(comma.to_token_stream()));
        stream
    }
}
