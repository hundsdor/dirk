use std::collections::HashMap;

use itertools::Itertools;
use proc_macro2::Ident;

use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Brace, Colon, Comma, Dot, Dyn, Eq, Gt, Let, Lt, Paren, RArrow, Semi},
    AngleBracketedGenericArguments, Block, Expr, ExprCall, ExprField, ExprMethodCall, ExprPath,
    Field, FieldValue, GenericArgument, GenericParam, Generics, ImplItem, ImplItemFn, ItemTrait,
    Local, LocalInit, Member, PatIdent, Path, PathArguments, PathSegment, ReturnType, Stmt,
    TraitBound, TraitItemFn, Type, TypeParamBound, TypePath, TypeTraitObject,
};

use crate::{
    errors::{InfallibleError},
    expectable::{
        GenericParamExpectable, ReturnTypeExpectable, TraitItemExpectable, TypeExpectable,
    },
    syntax::wrap_type,
    util::{type_provider, type_rc},
};

use super::{
    error::{ComponentLogicAbort, ComponentLogicEmit},
    Binding, ComponentResult,
};

pub(crate) fn get_stiletto_name(base: &Ident, suffix: Option<&str>) -> Ident {
    let suffix = suffix.unwrap_or("");
    let name = format!("Stiletto{base}{suffix}");
    Ident::new(&name, base.span())
}

pub(crate) fn get_bindings(raw: &Punctuated<Binding, Comma>) -> HashMap<&Ident, &Binding> {
    let mut res = HashMap::new();

    for binding in raw {
        res.insert(binding.identifier(), binding);
    }

    res
}

pub(crate) fn get_functions<'bindings>(
    base: Vec<&TraitItemFn>,
    bindings: &HashMap<&'bindings Ident, &'bindings Binding>,
) -> ComponentResult<Vec<ImplItem>> {
    let mut res = Vec::new();

    for function in base {
        let ident = &function.sig.ident;
        let binding = bindings
            .get(&function.sig.ident)
            .ok_or_else(|| ComponentLogicAbort::NotFound(ident.clone()))?;

        // Replace return type
        let ty_before = &function.sig.output;
        let ty_after = ReturnType::Type(
            RArrow::default(),
            Box::new(binding.kind().wrapped_ty().clone()),
        );

        // Check if types match
        {
            let mut path_before = ty_before.as_type()?.1.as_path()?.path.segments.clone();
            let mut path_after = ty_after.as_type()?.1.as_path()?.path.segments.clone();
            let span_before = path_before.span().clone();
            path_before
                .last_mut()
                .ok_or_else(|| InfallibleError::EmptyPath(span_before))?
                .arguments = PathArguments::None;
            let span_after = path_after.span().clone();
            path_after
                .last_mut()
                .ok_or_else(|| InfallibleError::EmptyPath(span_after))?
                .arguments = PathArguments::None;

            if path_before.last() != path_after.last() {
                return Err(ComponentLogicAbort::TypeMismatch {
                    fun_type: ty_before.as_type()?.1.as_ref().clone(),
                    binding_kind: (*binding).kind().clone(),
                })?;
            }
        }

        // Add call to self.xxxprovider.get()
        let call = get_provider_call(ident)?;

        let mut sig = function.sig.clone();
        sig.output = ty_after;

        let stmt = syn::Stmt::Expr(call, None);

        let block = Block {
            brace_token: Brace::default(),
            stmts: vec![stmt],
        };

        let new_function = ImplItemFn {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            defaultness: None,
            sig,
            block,
        };

        let impl_item = ImplItem::Fn(new_function);

        res.push(impl_item);
    }

    Ok(res)
}

fn get_provider_call(ident: &Ident) -> ComponentResult<Expr> {
    let provider_ident = Ident::new(&format!("{}_provider", ident.to_string()), ident.span());

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

    Ok(Expr::MethodCall(method_call))
}

pub(crate) fn get_providers<'bindings>(
    bindings: &HashMap<&'bindings Ident, &'bindings Binding>,
) -> ComponentResult<(
    Punctuated<Field, Comma>,
    Punctuated<FieldValue, Comma>,
    Vec<Stmt>,
)> {
    let mut fields = Punctuated::new();
    let mut field_values = Punctuated::new();
    let mut statements = Vec::new();

    let mut processed_bindings = Vec::new();

    for (ident, binding) in bindings
        .into_iter()
        .map(|(i, b)| (*i, *b))
        .sorted_by(|(_, r), (_, l)| r.cmp(l))
    {
        processed_bindings.push(ident);

        for dependency in binding.dependencies() {
            if !processed_bindings.contains(&dependency) {
                ComponentLogicEmit::NotFound(dependency.clone()).emit();
            }
        }

        let ty = binding.kind().wrapped_ty();
        let ty = wrap_type(ty, type_provider);

        let type_path = ty.as_path()?;

        let trait_bound = TraitBound {
            paren_token: None,
            modifier: syn::TraitBoundModifier::None,
            lifetimes: None,
            path: type_path.path.clone(),
        };
        let bound = TypeParamBound::Trait(trait_bound);

        let mut bounds = Punctuated::new();
        bounds.push(bound);

        let trait_object = TypeTraitObject {
            dyn_token: Some(Dyn::default()),
            bounds,
        };
        let dyn_type = Type::TraitObject(trait_object);

        let provider_ty = wrap_type(dyn_type, type_rc);

        let field = Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            mutability: syn::FieldMutability::None,
            ident: Some(Ident::new(&format!("{}_provider", ident), ident.span())),
            colon_token: Some(Colon::default()),
            ty: provider_ty,
        };

        fields.push(field);

        let field_value = {
            let ident = Ident::new(&format!("{}_provider", ident), ident.span());
            let member = Member::Named(ident.clone());
            let expr = syn::Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: Path::from(ident),
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
            ident: Ident::new(&format!("{}_provider", ident), ident.span()),
            subpat: None,
        };
        let pat = syn::Pat::Ident(pat_ident);

        let mut segments = Punctuated::new();
        segments.push(Ident::new("Rc", ident.span()).into());
        segments.push(Ident::new("new", ident.span()).into());

        let path = Path {
            leading_colon: None,
            segments,
        };

        let expr_path = ExprPath {
            attrs: Vec::new(),
            qself: None,
            path,
        };

        let rc_new_func = Expr::Path(expr_path);

        let rc_new_call = ExprCall {
            attrs: Vec::new(),
            func: Box::new(rc_new_func),
            paren_token: Paren::default(),
            args: std::iter::once(Expr::Call(binding.get_factory_create_call()?)).collect(),
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
    }

    Ok((fields, field_values, statements))
}

pub(crate) fn get_generics_mapping<'bindings>(
    input_trait: &ItemTrait,
    bindings: &HashMap<&'bindings Ident, &'bindings Binding>,
) -> ComponentResult<HashMap<GenericParam, Type>> {
    let funs = input_trait.items.iter().filter_map(|i| i.as_fn().ok());

    let map_arguments = {
        let mut map_arguments = HashMap::new();

        for fun in funs {
            let name = &fun.sig.ident;
            let ty = &fun.sig.output.as_type()?;

            let binding = bindings
                .get(name)
                .ok_or_else(|| ComponentLogicAbort::NotFound(name.clone()))?;

            let mapping = binding.kind().compare_types(ty.1)?;

            for (k, v) in mapping {
                let _ = map_arguments.insert(k, v);
            }
        }
        map_arguments
    };

    let params = {
        let generics = &input_trait.generics;
        &generics.params
    };

    let mut map = HashMap::new();

    for param in params {
        if let Ok(type_param) = param.as_type() {
            let ident = type_param.ident.clone();

            let key = {
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
                Type::Path(type_path)
            };

            if let Some(value) = map_arguments.get(&key) {
                map.insert(param.clone(), value.clone());
            }
        }
    }

    Ok(map)
}

pub(crate) fn process_generics(
    mapping: &HashMap<GenericParam, Type>,
    trait_generics: &Generics,
) -> (
    AngleBracketedGenericArguments,
    Generics,
    AngleBracketedGenericArguments,
) {
    let mut params_trait = Punctuated::new();
    let mut params_unbound_formal = Punctuated::new();
    let mut params_unbound_actual = Punctuated::new();

    for param in &trait_generics.params {
        if let Some(ty) = mapping.get(param) {
            // bound to ty
            let arg = GenericArgument::Type(ty.clone());
            params_trait.push(arg);
        } else {
            // unbound
            let actual = generic_argument_from_generic_param(param);

            params_unbound_formal.push(param.clone());
            params_unbound_actual.push(actual.clone());

            params_trait.push(actual);
        }
    }

    let generics_trait = AngleBracketedGenericArguments {
        colon2_token: None,
        lt_token: Lt::default(),
        args: params_trait,
        gt_token: Gt::default(),
    };
    let generics_unbound_formal = Generics {
        lt_token: Some(Lt::default()),
        params: params_unbound_formal,
        gt_token: Some(Gt::default()),
        where_clause: None, // TODO: include where clause
    };
    let generics_unbound_actual = AngleBracketedGenericArguments {
        colon2_token: None,
        lt_token: Lt::default(),
        args: params_unbound_actual,
        gt_token: Gt::default(),
    };

    (
        generics_trait,
        generics_unbound_formal,
        generics_unbound_actual,
    )
}

fn generic_argument_from_generic_param(input: &GenericParam) -> GenericArgument {
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