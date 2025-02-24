use std::{collections::HashMap, iter::zip};

use bindable::Bindable;
use proc_macro2::Ident;

use syn::{
    parse::Parse,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Colon, Comma},
    PathArguments, Type,
};

use crate::{
    errors::InfallibleError,
    expectable::{GenericArgumentExpectable, PathArgumentsExpectable, TypeExpectable},
    parse::ParseWithContext,
};

use self::{automatic::AutomaticBindingKind, manual::ManualBindingKind};

use super::{error::ComponentLogicAbort, ComponentResult};

pub(crate) mod bindable;

pub(crate) mod automatic;
pub(crate) mod manual;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum BindingKind {
    Automatic(AutomaticBindingKind),
    Manual(ManualBindingKind),
}

impl Parse for BindingKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = (input).lookahead1();

        if lookahead.peek(manual::kw::cloned_instance_bind)
            || lookahead.peek(manual::kw::scoped_instance_bind)
        {
            return input.parse::<ManualBindingKind>().map(BindingKind::Manual);
        }

        if lookahead.peek(automatic::kw::singleton_bind)
            || lookahead.peek(automatic::kw::scoped_bind)
            || lookahead.peek(automatic::kw::static_bind)
        {
            return input
                .parse::<AutomaticBindingKind>()
                .map(BindingKind::Automatic);
        }

        Err(lookahead.error())
    }
}

#[allow(clippy::match_wildcard_for_single_variants)]
impl BindingKind {
    #[allow(unused)]
    pub(crate) fn as_automatic(&self) -> Option<&AutomaticBindingKind> {
        match self {
            BindingKind::Automatic(a) => Some(a),
            _ => None,
        }
    }

    #[allow(unused)]
    pub(crate) fn as_manual(&self) -> Option<&ManualBindingKind> {
        match self {
            BindingKind::Manual(m) => Some(m),
            _ => None,
        }
    }

    pub(crate) fn ty(&self) -> ComponentResult<Type> {
        match self {
            BindingKind::Automatic(a) => a.ty(),
            BindingKind::Manual(m) => m.ty(),
        }
    }

    pub(crate) fn wrapped_ty(&self) -> ComponentResult<Type> {
        match self {
            BindingKind::Automatic(a) => a.wrapped_ty(),
            BindingKind::Manual(m) => m.wrapped_ty(),
        }
    }

    pub(crate) fn unwrap_ty<'o>(&self, other: &'o Type) -> ComponentResult<&'o Type> {
        let ty = match self {
            BindingKind::Automatic(a) => a.unwrap_ty(other),
            BindingKind::Manual(m) => m.unwrap_ty(other),
        }?;

        if let Ok(type_impl_trait) = ty.as_impl_trait() {
            Err(ComponentLogicAbort::ImplTraitBinding(
                type_impl_trait.clone(),
            ))?;
        }
        Ok(ty)
    }

    pub(crate) fn dependencies(&self) -> Option<&Punctuated<Ident, Comma>> {
        match self {
            BindingKind::Automatic(a) => a.dependencies(),
            BindingKind::Manual(_) => None,
        }
    }

    pub(crate) fn compare_types<'t>(
        &'t self,
        fun_ty: &'t Type,
    ) -> ComponentResult<HashMap<Type, Type>> {
        let binding_ty = self.ty()?;
        let fun_ty = self.unwrap_ty(fun_ty)?;

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

    pub(crate) fn hint(&self) -> &'static str {
        match self {
            BindingKind::Automatic(a) => a.hint(),
            BindingKind::Manual(m) => m.hint(),
        }
    }
}

fn unwrap_once<'ty>(ty: &'ty Type, expected_name: &str) -> ComponentResult<&'ty Type> {
    let type_path = ty
        .as_path()
        .map_err(|_e| ComponentLogicAbort::InvalidType(ty.clone()))?;

    let last_segment = type_path
        .path
        .segments
        .last()
        .ok_or_else(|| InfallibleError::EmptyPath(ty.span()))?;

    if last_segment.ident != expected_name {
        Err(ComponentLogicAbort::InvalidType(ty.clone()))?;
    }

    let args = &last_segment.arguments;
    let generics = match args {
        PathArguments::None => Err(ComponentLogicAbort::InvalidType(ty.clone())),
        PathArguments::AngleBracketed(generic_args) => Ok(generic_args),
        PathArguments::Parenthesized(_) => Err(ComponentLogicAbort::InvalidType(ty.clone())),
    }?;

    let generic_args = &generics.args;

    if generic_args.len() != 1 {
        Err(ComponentLogicAbort::InvalidType(ty.clone()))?;
    }

    let arg = generic_args
        .last()
        .ok_or_else(|| InfallibleError::EmptyPath(generics.span()))?;

    Ok(arg.as_type()?)
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Binding {
    identifier: Ident,
    colon: Colon,
    kind: BindingKind,
    index: usize,
}

impl Binding {
    pub(crate) fn identifier(&self) -> &Ident {
        &self.identifier
    }

    pub(crate) fn kind(&self) -> &BindingKind {
        &self.kind
    }
}

impl ParseWithContext<usize> for Binding {
    fn parse_with_context(input: syn::parse::ParseStream, index: usize) -> syn::Result<Self> {
        let identifier = input.parse()?;
        let colon = input.parse()?;
        let kind = input.parse()?;

        let res = Binding {
            identifier,
            colon,
            kind,
            index,
        };

        Ok(res)
    }
}

impl PartialOrd for Binding {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Binding {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if let Some(dependencies) = other.kind.dependencies() {
            if dependencies.iter().any(|d| *d == self.identifier) {
                return std::cmp::Ordering::Less;
            }
        }

        if let Some(dependencies) = self.kind.dependencies() {
            if dependencies.iter().any(|d| *d == other.identifier) {
                return std::cmp::Ordering::Greater;
            }
        }

        Ord::cmp(&self.index, &other.index)
    }
}
