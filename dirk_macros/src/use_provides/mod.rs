use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse::Parse, Item, ItemUse, UseTree};

use crate::{FACTORY_PREFIX_SCOPED, FACTORY_PREFIX_SINGLETON, FACTORY_PREFIX_STATIC};

use self::error::{UseInjectableLogicError, UseInjectableResult, UseInjectableSyntaxError};

mod error;

pub(crate) fn _macro(attr: TokenStream, item: TokenStream) -> UseInjectableResult<TokenStream> {
    let input = syn::parse::<UseInjectMacroInput>(attr)
        .map_err(UseInjectableSyntaxError::FailedToParseInput)?;
    let input_use = syn::parse::<ItemUse>(item).map_err(UseInjectableSyntaxError::ExpectedUse)?;

    let mut use_factories = input_use.clone();

    use_factories.attrs = Vec::new();
    input.convert_use_tree(&mut use_factories.tree)?;

    let items = vec![Item::Use(input_use), Item::Use(use_factories)];

    let expanded = quote! { #(#items)* };
    Ok(TokenStream::from(expanded))
}

mod kw {
    syn::custom_keyword!(singleton_inject);
    syn::custom_keyword!(scoped_inject);
    syn::custom_keyword!(static_inject);
}

#[allow(dead_code)]
#[derive(Debug)]
enum UseInjectMacroInput {
    Scoped(kw::scoped_inject),
    Singleton(kw::singleton_inject),
    Static(kw::static_inject),
}

impl Default for UseInjectMacroInput {
    fn default() -> Self {
        Self::Static(kw::static_inject {
            span: Span::call_site(),
        })
    }
}

impl Parse for UseInjectMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let lookahead = input.lookahead1();
        let res = if lookahead.peek(kw::singleton_inject) {
            let kw = kw::singleton_inject::parse(input)?;
            Self::Singleton(kw)
        } else if lookahead.peek(kw::scoped_inject) {
            let kw = kw::scoped_inject::parse(input)?;
            Self::Scoped(kw)
        } else if lookahead.peek(kw::static_inject) {
            let kw = kw::static_inject::parse(input)?;
            Self::Static(kw)
        } else {
            return Err(lookahead.error());
        };

        Ok(res)
    }
}

impl UseInjectMacroInput {
    fn factory_prefix(&self) -> &'static str {
        match self {
            UseInjectMacroInput::Singleton(_) => FACTORY_PREFIX_SINGLETON,
            UseInjectMacroInput::Scoped(_) => FACTORY_PREFIX_SCOPED,
            UseInjectMacroInput::Static(_) => FACTORY_PREFIX_STATIC,
        }
    }

    fn convert_use_tree(&self, tree: &mut UseTree) -> UseInjectableResult<()> {
        match tree {
            UseTree::Path(path) => self.convert_use_tree(&mut path.tree),
            UseTree::Group(g) => g
                .items
                .iter_mut()
                .try_for_each(|i| self.convert_use_tree(i)),
            UseTree::Name(name) => {
                let ident = &name.ident;
                name.ident = Ident::new(&format!("{}{ident}", self.factory_prefix()), ident.span());
                Ok(())
            }
            UseTree::Rename(use_rename) => {
                let ident = &use_rename.ident;
                use_rename.ident =
                    Ident::new(&format!("{}{ident}", self.factory_prefix()), ident.span());

                let rename = &use_rename.rename;
                use_rename.rename =
                    Ident::new(&format!("{}{rename}", self.factory_prefix()), rename.span());

                Ok(())
            }
            UseTree::Glob(g) => Err(UseInjectableLogicError::FoundGlob(g.clone()))?,
        }
    }
}
