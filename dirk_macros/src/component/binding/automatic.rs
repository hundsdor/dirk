use syn::{
    bracketed, parenthesized,
    parse::Parse,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Bracket, Comma, Dot, Paren},
    Error, Expr, ExprCall, ExprMethodCall, ExprPath, Ident, Path, PathArguments, PathSegment, Type,
};

use crate::{
    component::error::{ComponentLogicAbort, ComponentResult},
    errors::InfallibleError,
    expectable::TypeExpectable,
    syntax::wrap_type,
    util::{type_arc, type_rc, type_refcell, type_rwlock},
    FACTORY_PREFIX_SCOPED, FACTORY_PREFIX_SINGLETON, FACTORY_PREFIX_STATIC,
};

use super::unwrap_once;

pub(crate) mod kw {
    syn::custom_keyword!(singleton_bind);
    syn::custom_keyword!(scoped_bind);
    syn::custom_keyword!(static_bind);
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum AutomaticBindingKind {
    Singleton {
        kw: kw::singleton_bind,
        ty: Type,
        bracket: Option<Bracket>,
        dependencies: Punctuated<Ident, Comma>,
    },
    Scoped {
        kw: kw::scoped_bind,
        ty: Type,
        bracket: Option<Bracket>,
        dependencies: Punctuated<Ident, Comma>,
    },
    Static {
        kw: kw::static_bind,
        ty: Type,
        bracket: Option<Bracket>,
        dependencies: Punctuated<Ident, Comma>,
    },
}

impl Parse for AutomaticBindingKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = (input).lookahead1();
        let ty;

        let res = if lookahead.peek(kw::singleton_bind) {
            let kw = kw::singleton_bind::parse(input)?;
            parenthesized!(ty in input);
            let ty = ty.parse()?;
            let (bracket, dependencies) = {
                if input.peek(Bracket) {
                    let deps;
                    let bracket = bracketed!(deps in input);
                    let deps = deps.parse_terminated(Ident::parse, Comma)?;
                    (Some(bracket), deps)
                } else {
                    (None, Punctuated::new())
                }
            };
            Self::Singleton {
                kw,
                ty,
                bracket,
                dependencies,
            }
        } else if lookahead.peek(kw::scoped_bind) {
            let kw = kw::scoped_bind::parse(input)?;
            parenthesized!(ty in input);
            let ty = ty.parse()?;
            let (bracket, dependencies) = {
                if input.peek(Bracket) {
                    let deps;
                    let bracket = bracketed!(deps in input);
                    let deps = deps.parse_terminated(Ident::parse, Comma)?;
                    (Some(bracket), deps)
                } else {
                    (None, Punctuated::new())
                }
            };
            Self::Scoped {
                kw,
                ty,
                bracket,
                dependencies,
            }
        } else if lookahead.peek(kw::static_bind) {
            let kw = kw::static_bind::parse(input)?;
            parenthesized!(ty in input);
            let ty = ty.parse()?;
            let (bracket, dependencies) = {
                if input.peek(Bracket) {
                    let deps;
                    let bracket = bracketed!(deps in input);
                    let deps = deps.parse_terminated(Ident::parse, Comma)?;
                    (Some(bracket), deps)
                } else {
                    (None, Punctuated::new())
                }
            };
            Self::Static {
                kw,
                ty,
                bracket,
                dependencies,
            }
        } else {
            return Err(lookahead.error());
        };

        if !ty.is_empty() {
            Err(Error::new(input.span(), "Did not expect further tokens 1"))
        } else {
            Ok(res)
        }
    }
}

impl AutomaticBindingKind {
    pub(crate) fn ty(&self) -> ComponentResult<&Type> {
        let ty = match self {
            Self::Singleton {
                kw: _,
                ty,
                bracket: _,
                dependencies: _,
            } => ty,
            Self::Scoped {
                kw: _,
                ty,
                bracket: _,
                dependencies: _,
            } => ty,
            Self::Static {
                kw: _,
                ty,
                bracket: _,
                dependencies: _,
            } => ty,
        };

        if let Ok(type_impl_trait) = ty.as_impl_trait() {
            Err(ComponentLogicAbort::ImplTraitBinding(
                type_impl_trait.clone(),
            ))?;
        }
        Ok(ty)
    }

    pub(crate) fn wrapped_ty(&self) -> Type {
        match self {
            Self::Singleton {
                kw: _,
                ty,
                bracket: _,
                dependencies: _,
            } => wrap_type(wrap_type(ty.clone(), type_rwlock), type_arc),
            Self::Scoped {
                kw: _,
                ty,
                bracket: _,
                dependencies: _,
            } => wrap_type(wrap_type(ty.clone(), type_refcell), type_rc),
            Self::Static {
                kw: _,
                ty,
                bracket: _,
                dependencies: _,
            } => ty.clone(),
        }
    }

    pub(crate) fn unwrap_ty<'o>(&self, other: &'o Type) -> ComponentResult<&'o Type> {
        match self {
            Self::Singleton {
                kw: _,
                ty: _,
                bracket: _,
                dependencies: _,
            } => {
                let other = unwrap_once(other, "Arc")?;
                let other = unwrap_once(other, "RwLock")?;
                Ok(other)
            }
            Self::Scoped {
                kw: _,
                ty: _,
                bracket: _,
                dependencies: _,
            } => {
                let other = unwrap_once(other, "Rc")?;
                let other = unwrap_once(other, "RefCell")?;
                Ok(other)
            }
            Self::Static {
                kw: _,
                ty: _,
                bracket: _,
                dependencies: _,
            } => Ok(other),
        }
    }

    pub(crate) fn dependencies(&self) -> &Punctuated<Ident, Comma> {
        match self {
            Self::Singleton {
                kw: _,
                ty: _,
                bracket: _,
                dependencies,
            } => dependencies,
            Self::Scoped {
                kw: _,
                ty: _,
                bracket: _,
                dependencies,
            } => dependencies,
            Self::Static {
                kw: _,
                ty: _,
                bracket: _,
                dependencies,
            } => dependencies,
        }
    }

    pub(crate) fn hint(&self) -> &'static str {
        match self {
            Self::Singleton {
                kw: _,
                ty: _,
                bracket: _,
                dependencies: _,
            } => "singleton bindings wrap their type T into a std::sync::Arc<std::sync::RwLock<T>>",
            Self::Scoped {
                kw: _,
                ty: _,
                bracket: _,
                dependencies: _,
            } => "scoped bindings wrap their type T into a std::rc::Rc<std::cell::RefCell<T>>",
            Self::Static {
                kw: _,
                ty: _,
                bracket: _,
                dependencies: _,
            } => "static bindings do not wrap their type T and just return a T",
        }
    }

    pub(crate) fn get_factory_create_call(&self) -> ComponentResult<ExprCall> {
        let path = {
            let ty = self.ty()?;

            let mut segments = ty.as_path()?.path.segments.clone();
            let last = segments
                .last_mut()
                .ok_or_else(|| InfallibleError::EmptyPath(ty.span()))?;
            last.ident = Ident::new(
                &format!("{}{}", self.factory_prefix(), last.ident),
                last.ident.span(),
            );
            last.arguments = PathArguments::None;

            let create = PathSegment {
                ident: Ident::new("create", ty.span()),
                arguments: PathArguments::None,
            };
            segments.push(create);

            Path {
                leading_colon: None,
                segments,
            }
        };
        let expr_path = ExprPath {
            attrs: Vec::new(),
            qself: None,
            path,
        };
        let fun = syn::Expr::Path(expr_path);

        Ok(ExprCall {
            attrs: Vec::new(),
            func: Box::new(fun),
            paren_token: Paren::default(),
            args: self.provider_calls(),
        })
    }

    fn provider_calls(&self) -> Punctuated<Expr, Comma> {
        let mut res = Punctuated::new();

        for dependency in self.dependencies() {
            let provider_ident = Ident::new(&format!("{dependency}_provider"), dependency.span());

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

        res
    }

    fn factory_prefix(&self) -> &'static str {
        match self {
            Self::Singleton {
                kw: _,
                ty: _,
                bracket: _,
                dependencies: _,
            } => FACTORY_PREFIX_SINGLETON,
            Self::Scoped {
                kw: _,
                ty: _,
                bracket: _,
                dependencies: _,
            } => FACTORY_PREFIX_SCOPED,
            Self::Static {
                kw: _,
                ty: _,
                bracket: _,
                dependencies: _,
            } => FACTORY_PREFIX_STATIC,
        }
    }
}
