use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    parse::Parse, token::Dot, Expr, ExprField, ExprPath, Ident, Member, Path, PathArguments, Type,
};

use crate::{
    syntax::wrap_type,
    util::{
        path_arc_new, path_rc_new, path_refcell_new, path_rwlock_new, type_arc, type_rc,
        type_refcell, type_rwlock,
    },
    FACTORY_PREFIX_SCOPED, FACTORY_PREFIX_SINGLETON, FACTORY_PREFIX_STATIC,
};

use quote::quote;
use syn::{punctuated::Punctuated, PathSegment};

use error::ProvidesResult;

use self::{
    processor::{ProvidesMacroData, ProvidesMacroProcessor},
    syntax::wrap_call,
};

mod error;
mod processor;
mod syntax;

pub(crate) fn _macro(attr: TokenStream, item: TokenStream) -> ProvidesResult<TokenStream> {
    let data = ProvidesMacroData::new(attr, item);
    let processor = ProvidesMacroProcessor::new(&data);

    processor.process().map(|items| {
        let expaned = quote! { #(#items)* };
        TokenStream::from(expaned)
    })
}

mod kw {
    syn::custom_keyword!(singleton_inject);
    syn::custom_keyword!(scoped_inject);
    syn::custom_keyword!(static_inject);
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ProvidesMacroInput {
    Scoped(kw::scoped_inject),
    Singleton(kw::singleton_inject),
    Static(kw::static_inject),
}

impl Default for ProvidesMacroInput {
    fn default() -> Self {
        Self::Static(kw::static_inject {
            span: Span::call_site(),
        })
    }
}

impl Parse for ProvidesMacroInput {
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

impl ProvidesMacroInput {
    fn wrap_type(&self, injectable_ty: Type) -> Type {
        match self {
            ProvidesMacroInput::Static(_) => injectable_ty,
            ProvidesMacroInput::Scoped(_) => {
                let injectable_ty = wrap_type(injectable_ty, type_refcell);
                wrap_type(injectable_ty, type_rc)
            }
            ProvidesMacroInput::Singleton(_) => {
                let injectable_ty = wrap_type(injectable_ty, type_rwlock);
                wrap_type(injectable_ty, type_arc)
            }
        }
    }

    fn wrap_call(&self, constructor_call: Expr) -> Expr {
        match self {
            ProvidesMacroInput::Static(_) => constructor_call,
            ProvidesMacroInput::Scoped(_) => {
                let constructor_call = wrap_call(constructor_call, path_refcell_new);
                wrap_call(constructor_call, path_rc_new)
            }
            ProvidesMacroInput::Singleton(_) => {
                let constructor_call = wrap_call(constructor_call, path_rwlock_new);
                wrap_call(constructor_call, path_arc_new)
            }
        }
    }

    fn receiver(&self, ident: Ident) -> Expr {
        match self {
            ProvidesMacroInput::Static(_) => {
                let mut segments = Punctuated::new();

                let self_ident = Ident::new("self", ident.span());
                segments.push(PathSegment {
                    ident: self_ident,
                    arguments: PathArguments::None,
                });

                let path = Path {
                    leading_colon: None,
                    segments,
                };

                let self_expr = ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path,
                };
                let member = Member::Named(ident);

                let expr_field = ExprField {
                    attrs: Vec::new(),
                    base: Box::new(Expr::Path(self_expr)),
                    dot_token: Dot::default(),
                    member,
                };
                Expr::Field(expr_field)
            }
            ProvidesMacroInput::Scoped(_) | ProvidesMacroInput::Singleton(_) => {
                let segment = PathSegment {
                    ident,
                    arguments: PathArguments::None,
                };

                let mut segments = Punctuated::new();
                segments.push(segment);

                let path = Path {
                    leading_colon: None,
                    segments,
                };

                let expr_path = ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path,
                };

                Expr::Path(expr_path)
            }
        }
    }

    pub(crate) fn factory_prefix(&self) -> &'static str {
        match self {
            ProvidesMacroInput::Static(_) => FACTORY_PREFIX_STATIC,
            ProvidesMacroInput::Scoped(_) => FACTORY_PREFIX_SCOPED,
            ProvidesMacroInput::Singleton(_) => FACTORY_PREFIX_SINGLETON,
        }
    }
}
