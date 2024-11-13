use proc_macro::TokenStream;
use syn::parse::{ParseBuffer, ParseStream, Parser};

use syn::Result;

pub(crate) trait ParseWithContext<C>: Sized {
    fn parse_with_context(input: ParseStream<'_>, context: C) -> Result<Self>;
}

#[allow(unused)]
pub(crate) fn parse_with_context<T: ParseWithContext<C>, C>(
    tokens: TokenStream,
    context: C,
) -> Result<T> {
    Parser::parse2(
        |input: ParseStream| T::parse_with_context(input, context),
        proc_macro2::TokenStream::from(tokens),
    )
}

pub(crate) trait ExtensionParseBufferWithContext {
    fn parse_with_context<T: ParseWithContext<C>, C>(&self, context: C) -> Result<T>;
}

impl<'a> ExtensionParseBufferWithContext for ParseBuffer<'a> {
    fn parse_with_context<T: ParseWithContext<C>, C>(&self, context: C) -> Result<T> {
        T::parse_with_context(self, context)
    }
}
