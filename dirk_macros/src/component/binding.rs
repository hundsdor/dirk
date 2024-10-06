use std::{collections::HashMap, iter::zip};

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
};

use self::{automatic::AutomaticBindingKind, manual::ManualBindingKind};

use super::{error::ComponentLogicAbort, ComponentResult};

mod automatic;
mod manual;

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
            input.parse::<ManualBindingKind>().map(BindingKind::Manual)
        } else if lookahead.peek(automatic::kw::singleton_bind)
            || lookahead.peek(automatic::kw::scoped_bind)
            || lookahead.peek(automatic::kw::static_bind)
        {
            input
                .parse::<AutomaticBindingKind>()
                .map(BindingKind::Automatic)
        } else {
            return Err(lookahead.error());
        }
    }
}

impl BindingKind {
    pub(crate) fn as_automatic(&self) -> Option<&AutomaticBindingKind> {
        match self {
            BindingKind::Automatic(a) => Some(a),
            BindingKind::Manual(_) => None,
        }
    }

    pub(crate) fn as_manual(&self) -> Option<&ManualBindingKind> {
        match self {
            BindingKind::Automatic(_) => None,
            BindingKind::Manual(m) => Some(m),
        }
    }

    pub(crate) fn ty(&self) -> &Type {
        match self {
            BindingKind::Automatic(a) => a.ty(),
            BindingKind::Manual(m) => m.ty(),
        }
    }

    pub(crate) fn wrapped_ty(&self) -> Type {
        match self {
            BindingKind::Automatic(a) => a.wrapped_ty(),
            BindingKind::Manual(m) => m.wrapped_ty(),
        }
    }

    pub(crate) fn unwrap_ty<'o>(&self, other: &'o Type) -> ComponentResult<&'o Type> {
        match self {
            BindingKind::Automatic(a) => a.unwrap_ty(other),
            BindingKind::Manual(m) => m.unwrap_ty(other),
        }
    }

    pub(crate) fn dependencies(&self) -> Option<&Punctuated<Ident, Comma>> {
        match self {
            BindingKind::Automatic(a) => Some(a.dependencies()),
            BindingKind::Manual(_) => None,
        }
    }

    pub(crate) fn compare_types(&self, fun_ty: &Type) -> ComponentResult<HashMap<Type, Type>> {
        let binding_ty = self.ty();
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
    let type_path = ty.as_path()?;
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
        PathArguments::AngleBracketed(genric_args) => Ok(genric_args),
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
}

impl Binding {
    pub(crate) fn identifier(&self) -> &Ident {
        &self.identifier
    }

    pub(crate) fn kind(&self) -> &BindingKind {
        &self.kind
    }
}

impl Parse for Binding {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let identifier = input.parse()?;
        let colon = input.parse()?;
        let kind = input.parse()?;

        let res = Binding {
            identifier,
            colon,
            kind,
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
        if let Some(dependencies) = other.kind.dependencies() {
            if dependencies.iter().any(|d| *d == self.identifier) {
                return std::cmp::Ordering::Less;
            }
        }

        std::cmp::Ordering::Greater
    }
}
