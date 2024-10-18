use std::collections::HashMap;

use itertools::Itertools;
use proc_macro2::Ident;

use syn::{
    punctuated::Punctuated,
    token::{Colon, Comma, Dot, Dyn, Eq, Impl, Let, Paren, Semi},
    Expr, ExprCall, ExprField, ExprMethodCall, ExprPath, Field, FieldValue, FnArg, GenericArgument,
    GenericParam, Lifetime, Local, LocalInit, Member, Pat, PatIdent, PatType, Path, PathArguments,
    PathSegment, Stmt, TraitBound, Type, TypeImplTrait, TypeParamBound, TypePath, TypeTraitObject,
};

use crate::{
    expectable::TypeExpectable,
    syntax::wrap_type,
    util::{path_rc_new, type_provider, type_rc},
};

use super::{error::ComponentLogicEmit, Binding, ComponentResult};

pub(crate) fn get_dirk_name(base: &Ident, suffix: Option<&str>) -> Ident {
    let suffix = suffix.unwrap_or("");
    let name = format!("Dirk{base}{suffix}");
    Ident::new(&name, base.span())
}

pub(crate) fn get_provider_call(ident: &Ident) -> Expr {
    let provider_ident = Ident::new(&format!("{ident}_provider"), ident.span());

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
    let member = Member::Named(provider_ident);

    let expr_field = ExprField {
        attrs: Vec::new(),
        base: Box::new(Expr::Path(self_expr)),
        dot_token: Dot::default(),
        member,
    };
    let receiver = Expr::Field(expr_field);

    let get_ident = Ident::new("get", ident.span());

    let method_call = ExprMethodCall {
        attrs: Vec::new(),
        receiver: Box::new(receiver),
        dot_token: Dot::default(),
        method: get_ident,
        turbofish: None,
        paren_token: Paren::default(),
        args: Punctuated::new(),
    };

    Expr::MethodCall(method_call)
}

pub(crate) fn get_providers<'bindings>(
    bindings: &HashMap<&'bindings Ident, &'bindings Binding>,
) -> ComponentResult<(
    Punctuated<Field, Comma>,
    Punctuated<FieldValue, Comma>,
    Punctuated<FnArg, Comma>,
    Vec<Stmt>,
)> {
    let mut fields = Punctuated::new();
    let mut field_values = Punctuated::new();
    let mut fn_args = Punctuated::new();
    let mut statements = Vec::new();

    let mut processed_bindings = Vec::new();

    let processing_bindings = bindings
        .iter()
        .sorted_by(|(_, r), (_, l)| r.cmp(l))
        .map(|(i, b)| (*i, *b));

    for (ident, binding) in processing_bindings
        .sorted_by(|(i1, _), (i2, _)| Ord::cmp(i1, i2))
        .sorted_by(|(_, b1), (_, b2)| Ord::cmp(b1, b2))
    {
        processed_bindings.push(ident);

        if let Some(automatic_binding) = binding.kind().as_automatic() {
            for dependency in automatic_binding.dependencies() {
                if !processed_bindings.contains(&dependency) {
                    if bindings.get(dependency).is_some() {
                        ComponentLogicEmit::CycleDetected(
                            binding.identifier().clone(),
                            dependency.clone(),
                        )
                        .emit();
                    } else {
                        ComponentLogicEmit::NotFound(dependency.clone()).emit();
                    }
                }
            }
        }

        let ty = binding.kind().wrapped_ty();

        let provider_ident = Ident::new(&format!("{ident}_provider"), ident.span());
        let provider_ty = wrap_type(ty, type_provider);

        let provider_bounds = {
            let mut provider_bounds = Punctuated::new();

            let type_path = provider_ty.as_path()?;

            let trait_bound = TraitBound {
                paren_token: None,
                modifier: syn::TraitBoundModifier::None,
                lifetimes: None,
                path: type_path.path.clone(),
            };
            provider_bounds.push(TypeParamBound::Trait(trait_bound));

            let static_bound = Lifetime {
                apostrophe: ident.span(),
                ident: Ident::new("static", ident.span()),
            };
            provider_bounds.push(TypeParamBound::Lifetime(static_bound));

            provider_bounds
        };

        let trait_object = TypeTraitObject {
            dyn_token: Some(Dyn::default()),
            bounds: provider_bounds.clone(),
        };
        let dyn_type = Type::TraitObject(trait_object);

        let rc_dyn_type = wrap_type(dyn_type, type_rc);

        let field = Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            mutability: syn::FieldMutability::None,
            ident: Some(Ident::new(&format!("{ident}_provider"), ident.span())),
            colon_token: Some(Colon::default()),
            ty: rc_dyn_type,
        };

        fields.push(field);

        let field_value = {
            let member = Member::Named(provider_ident.clone());
            let expr = syn::Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: Path::from(provider_ident.clone()),
            });
            FieldValue {
                attrs: Vec::new(),
                member,
                colon_token: None,
                expr,
            }
        };

        field_values.push(field_value);

        let pat_ident = PatIdent {
            attrs: Vec::new(),
            by_ref: None,
            mutability: None,
            ident: Ident::new(&format!("{ident}_provider"), ident.span()),
            subpat: None,
        };
        let pat = syn::Pat::Ident(pat_ident);

        let path = path_rc_new(ident.span());

        let expr_path = ExprPath {
            attrs: Vec::new(),
            qself: None,
            path,
        };

        let rc_new_func = Expr::Path(expr_path);

        let args = match binding.kind() {
            super::binding::BindingKind::Automatic(a) => {
                let expr_call = a.get_factory_create_call()?;
                let call = Expr::Call(expr_call);

                let mut args = Punctuated::new();
                args.push(call);
                args
            }
            super::binding::BindingKind::Manual(_a) => {
                let expr_path = ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: Path::from(provider_ident.clone()),
                };
                let call = Expr::Path(expr_path);

                let mut args = Punctuated::new();
                args.push(call);
                args
            }
        };

        let rc_new_call = ExprCall {
            attrs: Vec::new(),
            func: Box::new(rc_new_func),
            paren_token: Paren::default(),
            args,
        };

        let expr = Expr::Call(rc_new_call);

        let init = LocalInit {
            eq_token: Eq::default(),
            expr: Box::new(expr),
            diverge: None,
        };

        let local = Local {
            attrs: Vec::new(),
            let_token: Let::default(),
            pat,
            init: Some(init),
            semi_token: Semi::default(),
        };
        let statement = Stmt::Local(local);
        statements.push(statement);

        if let Some(_binding) = binding.kind().as_manual() {
            let pat_ident = PatIdent {
                attrs: Vec::new(),
                by_ref: None,
                mutability: None,
                ident: provider_ident,
                subpat: None,
            };
            let pat = Pat::Ident(pat_ident);

            let ty = {
                let type_impl_trait = TypeImplTrait {
                    impl_token: Impl::default(),
                    bounds: provider_bounds,
                };
                Type::ImplTrait(type_impl_trait)
            };

            let pat_type = PatType {
                attrs: Vec::new(),
                pat: Box::new(pat),
                colon_token: Colon::default(),
                ty: Box::new(ty),
            };

            let fn_arg = FnArg::Typed(pat_type);
            fn_args.push(fn_arg);
        }
    }

    Ok((fields, field_values, fn_args, statements))
}

pub(crate) fn generic_argument_from_generic_param(input: &GenericParam) -> GenericArgument {
    match input {
        GenericParam::Lifetime(lt_param) => GenericArgument::Lifetime(lt_param.lifetime.clone()),
        GenericParam::Type(ty_param) => {
            let ident = ty_param.ident.clone();
            let mut segments = Punctuated::new();
            let segment = PathSegment {
                ident,
                arguments: PathArguments::None,
            };
            segments.push(segment);
            let path = Path {
                leading_colon: None,
                segments,
            };
            let type_path = TypePath { qself: None, path };
            let ty = Type::Path(type_path);

            GenericArgument::Type(ty)
        }
        GenericParam::Const(_con_param) => {
            todo!("Handle const params")
        }
    }
}
