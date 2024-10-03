use std::{collections::HashMap, iter::zip};

use proc_macro2::Ident;

use syn::{
    bracketed, parenthesized,
    parse::Parse,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{self, Bracket, Colon, Comma, Dot, Paren},
    Error, Expr, ExprCall, ExprMethodCall, ExprPath, Path, PathArguments, PathSegment, Type,
};

use crate::{
    errors::{InfallibleError},
    expectable::{GenericArgumentExpectable, PathArgumentsExpectable, TypeExpectable},
    syntax::wrap_type,
    util::{type_arc, type_rc, type_refcell, type_rwlock},
    FACTORY_PREFIX_SCOPED, FACTORY_PREFIX_SINGLETON, FACTORY_PREFIX_STATIC,
};

use super::{error::ComponentLogicAbort, ComponentResult};

mod kw {
    syn::custom_keyword!(singleton_bind);
    syn::custom_keyword!(scoped_bind);
    syn::custom_keyword!(static_bind);
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum BindingKind {
    Singleton(Type),
    Scoped(Type),
    Static(Type),
}

impl BindingKind {
    fn factory_prefix(&self) -> &'static str {
        match self {
            BindingKind::Singleton(_) => FACTORY_PREFIX_SINGLETON,
            BindingKind::Scoped(_) => FACTORY_PREFIX_SCOPED,
            BindingKind::Static(_) => FACTORY_PREFIX_STATIC,
        }
    }

    pub(crate) fn ty(&self) -> &Type {
        match self {
            BindingKind::Singleton(ty) => ty,
            BindingKind::Scoped(ty) => ty,
            BindingKind::Static(ty) => ty,
        }
    }

    pub(crate) fn wrapped_ty(&self) -> Type {
        match self {
            BindingKind::Singleton(ty) => wrap_type(wrap_type(ty.clone(), type_rwlock), type_arc),
            BindingKind::Scoped(ty) => wrap_type(wrap_type(ty.clone(), type_refcell), type_rc),
            BindingKind::Static(ty) => ty.clone(),
        }
    }

    pub(crate) fn compare_types(&self, fun_ty: &Type) -> ComponentResult<HashMap<Type, Type>> {
        let (fun_ty, binding_ty) = match self {
            BindingKind::Singleton(ty) => {
                let fun_ty = unwrap_once(fun_ty, "Arc")?;
                let fun_ty = unwrap_once(fun_ty, "RwLock")?;
                (fun_ty, ty)
            }
            BindingKind::Scoped(ty) => {
                let fun_ty = unwrap_once(fun_ty, "Rc")?;
                let fun_ty = unwrap_once(fun_ty, "RefCell")?;
                (fun_ty, ty)
            }
            BindingKind::Static(ty) => (fun_ty, ty),
        };

        let mut map = HashMap::new();

        let maybe_args_fun = {
            let type_path = fun_ty.as_path()?;
            let segments = &type_path.path.segments;

            segments.last().map(|l| &l.arguments)
        };

        let maybe_args_binding = {
            let type_path = binding_ty.as_path()?;
            let segments = &type_path.path.segments;

            segments.last().map(|l| &l.arguments)
        };

        // Check if angle-bracketed generics match
        {
            let maybe_angle_bracketed_fun = maybe_args_fun
                .and_then(|args_fun| args_fun.as_angle_bracketed().ok())
                .map(|a| &a.args);
            let maybe_angle_bracketed_binding = maybe_args_binding
                .and_then(|args_binding| args_binding.as_angle_bracketed().ok())
                .map(|a| &a.args);

            if let Some(angle_bracketed_fun) = maybe_angle_bracketed_fun {
                let combined = maybe_angle_bracketed_binding
                    .map(|angle_bracketed_binding| (angle_bracketed_fun, angle_bracketed_binding))
                    .filter(|(angle_bracketed_fun, angle_bracketed_binding)| {
                        angle_bracketed_fun.len() == angle_bracketed_binding.len()
                    });

                if let Some((angle_bracketed_fun, angle_bracketed_binding)) = combined {
                    for (arg_fun, arg_binding) in zip(angle_bracketed_fun, angle_bracketed_binding)
                    {
                        if let Ok(arg_fun) = arg_fun.as_type() {
                            let arg_binding = arg_binding.as_type()?;
                            map.insert(arg_fun.clone(), arg_binding.clone());
                        }
                    }
                } else {
                    return Err(ComponentLogicAbort::TypeMismatch {
                        fun_type: fun_ty.clone(),
                        binding_kind: self.clone(),
                    })?;
                }
            };
        }

        Ok(map)
    }
}

fn unwrap_once<'ty>(ty: &'ty Type, expected_name: &str) -> ComponentResult<&'ty Type> {
    let type_path = ty.as_path()?;
    let last_segment = type_path
        .path
        .segments
        .last()
        .ok_or_else(|| InfallibleError::EmptyPath(ty.span()))?;

    if last_segment.ident != expected_name {
        return Err(ComponentLogicAbort::InvalidType(ty.clone()))?;
    }

    let args = &last_segment.arguments;
    let generics = match args {
        PathArguments::None => Err(ComponentLogicAbort::InvalidType(ty.clone())),
        PathArguments::AngleBracketed(genric_args) => Ok(genric_args),
        PathArguments::Parenthesized(_) => Err(ComponentLogicAbort::InvalidType(ty.clone())),
    }?;

    let generic_args = &generics.args;

    if generic_args.len() != 1 {
        return Err(ComponentLogicAbort::InvalidType(ty.clone()))?;
    }

    let arg = generic_args
        .last()
        .ok_or_else(|| InfallibleError::EmptyPath(generics.span()))?;

    Ok(arg.as_type()?)
}

impl Parse for BindingKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        let ty;

        // TODO: include kws
        let res = if lookahead.peek(kw::singleton_bind) {
            let _kw = kw::singleton_bind::parse(input)?;
            parenthesized!(ty in input);
            ty.parse().map(BindingKind::Singleton)?
        } else if lookahead.peek(kw::scoped_bind) {
            let _kw = kw::scoped_bind::parse(input)?;
            parenthesized!(ty in input);
            ty.parse().map(BindingKind::Scoped)?
        } else if lookahead.peek(kw::static_bind) {
            let _kw = kw::static_bind::parse(input)?;
            parenthesized!(ty in input);
            ty.parse().map(BindingKind::Static)?
        } else {
            return Err(lookahead.error());
        };

        if !ty.is_empty() {
            Err(Error::new(input.span(), "Did not expect further tokens"))
        } else {
            Ok(res)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Binding {
    identifier: Ident,
    colon: Colon,
    kind: BindingKind,
    bracket: Option<Bracket>,
    dependencies: Punctuated<Ident, Comma>,
}

impl Binding {
    pub(crate) fn identifier(&self) -> &Ident {
        &self.identifier
    }

    pub(crate) fn kind(&self) -> &BindingKind {
        &self.kind
    }

    pub(crate) fn dependencies(&self) -> &Punctuated<Ident, Comma> {
        &self.dependencies
    }

    pub(crate) fn get_factory_create_call(&self) -> ComponentResult<ExprCall> {
        let path = {
            let ty = self.kind.ty();

            let mut segments = ty.as_path()?.path.segments.clone();
            let last = segments
                .last_mut()
                .ok_or_else(|| InfallibleError::EmptyPath(ty.span()))?;
            last.ident = Ident::new(
                &format!("{}{}", self.kind.factory_prefix(), last.ident),
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
            args: self.provider_calls()?,
        })
    }

    pub(crate) fn provider_calls(&self) -> ComponentResult<Punctuated<Expr, Comma>> {
        let mut res = Punctuated::new();

        for dependency in &self.dependencies {
            let provider_ident = Ident::new(&format!("{}_provider", dependency), dependency.span());

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

        Ok(res)
    }
}

impl Parse for Binding {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let identifier = input.parse()?;
        let colon = input.parse()?;
        let kind = input.parse()?;

        let (bracket, dependencies) = {
            if input.peek(token::Bracket) {
                let deps;
                let bracket = bracketed!(deps in input);
                let deps = deps.parse_terminated(Ident::parse, Comma)?;
                (Some(bracket), deps)
            } else {
                (None, Punctuated::new())
            }
        };

        let res = Binding {
            identifier,
            colon,
            kind,
            bracket,
            dependencies,
        };

        Ok(res)
    }
}

impl PartialOrd for Binding {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// TODO: does not yet give meaningful error in case of cycles
impl Ord for Binding {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if other.dependencies.iter().any(|d| *d == self.identifier) {
            return std::cmp::Ordering::Less;
        }
        std::cmp::Ordering::Greater
    }
}
