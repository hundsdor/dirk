use syn::ExprPath;
use syn::{
    bracketed, parenthesized,
    parse::Parse,
    punctuated::Punctuated,
    token::{Bracket, Comma, Dot, Paren},
    Error, Expr, ExprMethodCall, Ident, Path, PathArguments, PathSegment, Type,
};

use crate::{
    component::error::{ComponentLogicAbort, ComponentResult},
    expectable::TypeExpectable,
    syntax::wrap_type,
    util::{type_arc, type_rc, type_refcell, type_rwlock},
    FACTORY_PREFIX_SCOPED, FACTORY_PREFIX_SINGLETON, FACTORY_PREFIX_STATIC,
};

use super::{bindable::Bindable, bindable::FactoryBindable, unwrap_once};

pub(crate) mod kw {
    syn::custom_keyword!(singleton_bind);
    syn::custom_keyword!(scoped_bind);
    syn::custom_keyword!(static_bind);
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum AutomaticBindingKind {
    Singleton {
        kw: kw::singleton_bind,
        paren: Paren,
        ty: Type,
        bracket: Option<Bracket>,
        dependencies: Punctuated<Ident, Comma>,
    },
    Scoped {
        kw: kw::scoped_bind,
        paren: Paren,
        ty: Type,
        bracket: Option<Bracket>,
        dependencies: Punctuated<Ident, Comma>,
    },
    Static {
        kw: kw::static_bind,
        paren: Paren,
        ty: Type,
        bracket: Option<Bracket>,
        dependencies: Punctuated<Ident, Comma>,
    },
}

impl Parse for AutomaticBindingKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = (input).lookahead1();

        if lookahead.peek(kw::singleton_bind) {
            let ty_buf;
            let kw = kw::singleton_bind::parse(input)?;
            let paren = parenthesized!(ty_buf in input);
            let ty = ty_buf.parse()?;
            let (bracket, dependencies) = {
                if input.peek(Bracket) {
                    let deps_buf;
                    let bracket = bracketed!(deps_buf in input);
                    let deps = deps_buf.parse_terminated(Ident::parse, Comma)?;
                    (Some(bracket), deps)
                } else {
                    (None, Punctuated::new())
                }
            };

            if !ty_buf.is_empty() {
                Err(Error::new(input.span(), "Did not expect further tokens"))?;
            }

            return Ok(Self::Singleton {
                kw,
                ty,
                paren,
                bracket,
                dependencies,
            });
        }

        if lookahead.peek(kw::scoped_bind) {
            let ty_buf;
            let kw = kw::scoped_bind::parse(input)?;
            let paren = parenthesized!(ty_buf in input);
            let ty = ty_buf.parse()?;
            let (bracket, dependencies) = {
                if input.peek(Bracket) {
                    let deps_buf;
                    let bracket = bracketed!(deps_buf in input);
                    let deps = deps_buf.parse_terminated(Ident::parse, Comma)?;
                    (Some(bracket), deps)
                } else {
                    (None, Punctuated::new())
                }
            };

            if !ty_buf.is_empty() {
                Err(Error::new(input.span(), "Did not expect further tokens"))?;
            }

            return Ok(Self::Scoped {
                kw,
                paren,
                ty,
                bracket,
                dependencies,
            });
        }

        if lookahead.peek(kw::static_bind) {
            let ty_buf;
            let kw = kw::static_bind::parse(input)?;
            let paren = parenthesized!(ty_buf in input);
            let ty = ty_buf.parse()?;
            let (bracket, dependencies) = {
                if input.peek(Bracket) {
                    let deps_buf;
                    let bracket = bracketed!(deps_buf in input);
                    let deps = deps_buf.parse_terminated(Ident::parse, Comma)?;
                    (Some(bracket), deps)
                } else {
                    (None, Punctuated::new())
                }
            };

            if !ty_buf.is_empty() {
                Err(Error::new(input.span(), "Did not expect further tokens"))?;
            }

            return Ok(Self::Static {
                kw,
                paren,
                ty,
                bracket,
                dependencies,
            });
        }

        Err(lookahead.error())
    }
}

impl Bindable for AutomaticBindingKind {
    fn ty(&self) -> ComponentResult<Type> {
        if let Self::Singleton { dependencies, .. } = self {
            if !dependencies.is_empty() {
                return Err(ComponentLogicAbort::UnexpectedDependencies(
                    dependencies.clone(),
                ))?;
            }
        }

        let ty = match self {
            Self::Singleton { ty, .. } | Self::Scoped { ty, .. } | Self::Static { ty, .. } => {
                ty.clone()
            }
        };

        if let Ok(type_impl_trait) = ty.as_impl_trait() {
            Err(ComponentLogicAbort::ImplTraitBinding(
                type_impl_trait.clone(),
            ))?;
        }

        Ok(ty)
    }

    fn wrapped_ty(&self) -> ComponentResult<Type> {
        match self {
            Self::Singleton { .. } => self
                .ty()
                .map(|ty| wrap_type(wrap_type(ty, type_rwlock), type_arc)),
            Self::Scoped { .. } => self
                .ty()
                .map(|ty| wrap_type(wrap_type(ty, type_refcell), type_rc)),
            Self::Static { .. } => self.ty(),
        }
    }

    fn unwrap_ty<'o>(&self, other: &'o Type) -> ComponentResult<&'o Type> {
        match self {
            Self::Singleton { ty: _, .. } => {
                let other = unwrap_once(other, "Arc")?;
                let other = unwrap_once(other, "RwLock")?;
                Ok(other)
            }
            Self::Scoped { ty: _, .. } => {
                let other = unwrap_once(other, "Rc")?;
                let other = unwrap_once(other, "RefCell")?;
                Ok(other)
            }
            Self::Static { ty: _, .. } => Ok(other),
        }
    }

    fn hint(&self) -> &'static str {
        match self {
            Self::Singleton { .. } => {
                "singleton bindings wrap their type T into a std::sync::Arc<std::sync::RwLock<T>>"
            }
            Self::Scoped { .. } => {
                "scoped bindings wrap their type T into a std::rc::Rc<std::cell::RefCell<T>>"
            }
            Self::Static { .. } => "static bindings do not wrap their type T and just return a T",
        }
    }
}

impl FactoryBindable for AutomaticBindingKind {
    fn provider_calls(&self) -> Punctuated<Expr, Comma> {
        let mut res = Punctuated::new();

        if let Some(dependencies) = self.dependencies() {
            for dependency in dependencies {
                let provider_ident =
                    Ident::new(&format!("{dependency}_provider"), dependency.span());

                let mut segments = Punctuated::new();
                let segment = PathSegment {
                    ident: provider_ident,
                    arguments: PathArguments::None,
                };
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
                let receiver = Expr::Path(expr_path);

                let method = Ident::new("clone", dependency.span());
                let expr_method_call = ExprMethodCall {
                    attrs: Vec::new(),
                    receiver: Box::new(receiver),
                    dot_token: Dot::default(),
                    method,
                    turbofish: None,
                    paren_token: Paren::default(),
                    args: Punctuated::new(),
                };

                let expr = Expr::MethodCall(expr_method_call);
                res.push(expr);
            }
        }

        res
    }

    fn factory_prefix(&self) -> &'static str {
        match self {
            Self::Singleton { .. } => FACTORY_PREFIX_SINGLETON,
            Self::Scoped { .. } => FACTORY_PREFIX_SCOPED,
            Self::Static { .. } => FACTORY_PREFIX_STATIC,
        }
    }
}

impl AutomaticBindingKind {
    pub(crate) fn dependencies(&self) -> Option<&Punctuated<Ident, Comma>> {
        match self {
            Self::Singleton {
                dependencies: _, ..
            } => None,
            Self::Scoped { dependencies, .. } => Some(dependencies),
            Self::Static { dependencies, .. } => Some(dependencies),
        }
    }
}
