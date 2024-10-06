use proc_macro2::Span;
use syn::{
    parenthesized, parse::Parse, punctuated::Punctuated, token::Paren, token::PathSep, Error, Expr,
    ExprCall, ExprPath, Ident, Path, PathArguments, PathSegment, Type,
};

use crate::{
    component::error::ComponentResult,
    syntax::wrap_type,
    util::{segments, type_rc, type_refcell},
};

use super::unwrap_once;

pub(crate) mod kw {
    syn::custom_keyword!(cloned_instance_bind);
    syn::custom_keyword!(scoped_instance_bind);
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum ManualBindingKind {
    ClonedInstance {
        kw: kw::cloned_instance_bind,
        ty: Type,
    },
    ScopedInstance {
        kw: kw::scoped_instance_bind,
        ty: Type,
    },
}

impl Parse for ManualBindingKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = (input).lookahead1();
        let ty;

        let res = if lookahead.peek(kw::cloned_instance_bind) {
            let kw = kw::cloned_instance_bind::parse(input)?;
            parenthesized!(ty in input);
            let ty = ty.parse()?;
            Self::ClonedInstance { kw, ty }
        } else if lookahead.peek(kw::scoped_instance_bind) {
            let kw = kw::scoped_instance_bind::parse(input)?;
            parenthesized!(ty in input);
            let ty = ty.parse()?;
            Self::ScopedInstance { kw, ty }
        } else {
            return Err(lookahead.error());
        };

        if !ty.is_empty() {
            Err(Error::new(input.span(), "Did not expect further tokens 2"))
        } else {
            Ok(res)
        }
    }
}

impl ManualBindingKind {
    pub(crate) fn ty(&self) -> &Type {
        match self {
            Self::ScopedInstance { kw: _, ty } => ty,
            Self::ClonedInstance { kw: _, ty } => ty,
        }
    }

    pub(crate) fn wrapped_ty(&self) -> Type {
        match self {
            Self::ScopedInstance { kw: _, ty } => {
                wrap_type(wrap_type(ty.clone(), type_refcell), type_rc)
            }
            Self::ClonedInstance { kw: _, ty } => ty.clone(),
        }
    }

    pub(crate) fn unwrap_ty<'o>(&self, other: &'o Type) -> ComponentResult<&'o Type> {
        match self {
            Self::ScopedInstance { kw: _, ty: _ } => {
                let other = unwrap_once(other, "Rc")?;
                let other = unwrap_once(other, "RefCell")?;
                Ok(other)
            }
            Self::ClonedInstance { kw: _, ty: _ } => Ok(other),
        }
    }

    pub(crate) fn hint(&self) -> &'static str {
        match self {
            Self::ClonedInstance { kw: _, ty:_ } => {
                "cloned instance bindings do not wrap their type T and just return a T"
            }
            Self::ScopedInstance {kw: _, ty:_ } => {
                "scoped instance bindings wrap their type T into a std::rc::Rc<std::cell::RefCell<T>>"
            },
        }
    }

    pub(crate) fn get_new_factory(&self, ident: &Ident) -> Expr {
        let path = match self {
            ManualBindingKind::ClonedInstance { kw: _, ty: _ } => {
                let segments = segments!("dirk", "ClonedInstanceFactory", "new");
                Path {
                    leading_colon: None,
                    segments,
                }
            }
            ManualBindingKind::ScopedInstance { kw: _, ty: _ } => {
                let segments = segments!("dirk", "ScopedInstanceFactory", "new");
                Path {
                    leading_colon: None,
                    segments,
                }
            }
        };

        let expr_path = ExprPath {
            attrs: Vec::new(),
            qself: None,
            path,
        };

        let func = Expr::Path(expr_path);

        let mut args = Punctuated::new();
        let path = Path::from(ident.clone());
        let expr_path = ExprPath {
            attrs: Vec::new(),
            qself: None,
            path,
        };
        args.push(Expr::Path(expr_path));

        let expr_call = ExprCall {
            attrs: Vec::new(),
            func: Box::new(func),
            paren_token: Paren::default(),
            args,
        };
        Expr::Call(expr_call)
    }
}
