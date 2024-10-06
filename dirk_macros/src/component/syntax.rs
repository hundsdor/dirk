use std::collections::HashMap;

use itertools::Itertools;
use proc_macro2::{Ident, Span};

use syn::{
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::PathSep,
    token::{Brace, Colon, Comma, Dot, Dyn, Eq, Gt, Impl, Let, Lt, Paren, RArrow, Semi},
    AngleBracketedGenericArguments, Block, Expr, ExprCall, ExprField, ExprMethodCall, ExprPath,
    Field, FieldValue, FnArg, GenericArgument, GenericParam, Generics, ImplItem, ImplItemFn,
    ItemImpl, ItemTrait, Lifetime, Local, LocalInit, Member, Pat, PatIdent, PatTupleStruct,
    PatType, Path, PathArguments, PathSegment, ReturnType, Stmt, TraitBound, TraitItemFn, Type,
    TypeImplTrait, TypeParam, TypeParamBound, TypePath, TypeTraitObject,
};

use crate::{
    errors::InfallibleError,
    expectable::{
        GenericParamExpectable, ReturnTypeExpectable, TraitItemExpectable, TypeExpectable,
    },
    syntax::wrap_type,
    util::{segments, type_provider, type_rc, type_set, type_unset},
};

use super::{
    error::{ComponentLogicAbort, ComponentLogicEmit},
    Binding, ComponentResult,
};

pub(crate) fn get_dirk_name(base: &Ident, suffix: Option<&str>) -> Ident {
    let suffix = suffix.unwrap_or("");
    let name = format!("Dirk{base}{suffix}");
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
            let span_before = path_before.span();
            path_before
                .last_mut()
                .ok_or_else(|| InfallibleError::EmptyPath(span_before))?
                .arguments = PathArguments::None;
            let span_after = path_after.span();
            path_after
                .last_mut()
                .ok_or_else(|| InfallibleError::EmptyPath(span_after))?
                .arguments = PathArguments::None;

            if path_before.last() != path_after.last() {
                Err(ComponentLogicAbort::TypeMismatch {
                    fun_type: ty_before.as_type()?.1.clone(),
                    binding_kind: (*binding).kind().clone(),
                })?;
            }
        }

        // Add call to self.xxxprovider.get()
        let call = get_provider_call(ident);

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

fn get_provider_call(ident: &Ident) -> Expr {
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

    let bindings = bindings
        .iter()
        .sorted_by(|(_, r), (_, l)| r.cmp(l))
        .map(|(i, b)| (*i, *b));

    for (ident, binding) in bindings {
        processed_bindings.push(ident);

        if let Some(binding) = binding.kind().as_automatic() {
            for dependency in binding.dependencies() {
                if !processed_bindings.contains(&dependency) {
                    ComponentLogicEmit::NotFound(dependency.clone()).emit();
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
                apostrophe: Span::call_site(),
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

        let segments = segments!("std", "rc", "Rc", "new");

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

pub(crate) fn process_instance_binds<'bindings>(
    dirk_ident: &Ident,
    impl_path: &TypePath,
    trait_ident: &Ident,
    trait_type: &Type,
    generics_trait: &AngleBracketedGenericArguments,
    generics_unbound_formal: &Generics,
    unbound_generics_mapping: &HashMap<Ident, GenericParam>,
    builder_ident: &Ident,
    builder_field_values: Punctuated<FieldValue, Comma>,
    builder_statements: Vec<Stmt>,
    bindings: &HashMap<&'bindings Ident, &'bindings Binding>,
) -> (ItemImpl, Vec<ItemImpl>, ItemImpl, ItemImpl) {
    let instance_binds = bindings
        .iter()
        .filter_map(|(i, b)| b.kind().as_manual().map(|m| (*i, m)))
        .peekable(); // TODO: maybe sorted

    let (unset_generics, set_generics) = {
        let mut instance_binds = instance_binds.clone();
        if instance_binds.peek().is_none() {
            (PathArguments::None, PathArguments::None)
        } else {
            let instance_binds = instance_binds.clone();

            let mut unset_args = Punctuated::new();
            let mut set_args = Punctuated::new();

            for (_ident, binding) in instance_binds {
                let ty = binding.ty();

                let set_generics = {
                    let mut args = Punctuated::new();
                    let generic_arg = GenericArgument::Type(ty.clone());
                    args.push(generic_arg);

                    let angle_bracketed = AngleBracketedGenericArguments {
                        colon2_token: None,
                        lt_token: Lt::default(),
                        args,
                        gt_token: Gt::default(),
                    };

                    PathArguments::AngleBracketed(angle_bracketed)
                };

                let unset_arg = GenericArgument::Type(type_unset(PathArguments::None));
                let set_arg = GenericArgument::Type(type_set(set_generics));

                // handle unset_args
                unset_args.push(unset_arg.clone());

                // handle set_args
                set_args.push(set_arg.clone());
            }

            let unset = AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Lt::default(),
                args: unset_args,
                gt_token: Gt::default(),
            };

            let set = AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Lt::default(),
                args: set_args,
                gt_token: Gt::default(),
            };

            (
                PathArguments::AngleBracketed(unset),
                PathArguments::AngleBracketed(set),
            )
        }
    };

    let partials = {
        let mut instance_binds = instance_binds.clone();
        if instance_binds.peek().is_none() {
            let dirk_impl_static_component = parse_quote! {
                impl #dirk_ident {
                    fn create #generics_unbound_formal () -> impl #trait_type {
                        <Self as dirk::DirkComponent<#builder_ident>>::builder().build()
                    }
                }
            };

            vec![dirk_impl_static_component]
        } else {
            let mut partial_impls = Vec::new();

            for (index_set, (ident, binding)) in instance_binds.clone().enumerate() {
                let ty = binding.ty();

                let set_generics = {
                    let mut args = Punctuated::new();
                    let generic_arg = GenericArgument::Type(ty.clone());
                    args.push(generic_arg);

                    let angle_bracketed = AngleBracketedGenericArguments {
                        colon2_token: None,
                        lt_token: Lt::default(),
                        args,
                        gt_token: Gt::default(),
                    };

                    PathArguments::AngleBracketed(angle_bracketed)
                };

                let unset_arg = GenericArgument::Type(type_unset(PathArguments::None));
                let set_arg = GenericArgument::Type(type_set(set_generics));

                let mut args_pure: Punctuated<GenericParam, Comma> = Punctuated::new();
                let mut args_containing_unset: Punctuated<GenericArgument, Comma> =
                    Punctuated::new();
                let mut args_containing_set: Punctuated<GenericArgument, Comma> = Punctuated::new();

                let mut statements_opaque = Vec::new();

                for (index_opaque, (ident, _binding)) in instance_binds.clone().enumerate() {
                    if index_opaque == index_set {
                        args_containing_unset.push(unset_arg.clone());

                        let path = Path {
                            leading_colon: None,
                            segments: segments!("dirk", "Set"),
                        };
                        let expr_path = ExprPath {
                            attrs: Vec::new(),
                            qself: None,
                            path,
                        };
                        let set_constructor = Expr::Path(expr_path);

                        let mut args = Punctuated::new();

                        let expr_path = ExprPath {
                            attrs: Vec::new(),
                            qself: None,
                            path: Path::from(ident.clone()),
                        };
                        let arg = Expr::Path(expr_path);
                        args.push(arg);

                        let expr_call = ExprCall {
                            attrs: Vec::new(),
                            func: Box::new(set_constructor),
                            paren_token: Paren::default(),
                            args,
                        };

                        let expr_set = Expr::Call(expr_call);

                        let init = LocalInit {
                            eq_token: Eq::default(),
                            expr: Box::new(expr_set),
                            diverge: None,
                        };

                        let pat_ident = PatIdent {
                            attrs: Vec::new(),
                            by_ref: None,
                            mutability: None,
                            ident: ident.clone(),
                            subpat: None,
                        };
                        let pat = syn::Pat::Ident(pat_ident);

                        let local = Local {
                            attrs: Vec::new(),
                            let_token: Let::default(),
                            pat,
                            init: Some(init),
                            semi_token: Semi::default(),
                        };

                        let statement = Stmt::Local(local);
                        statements_opaque.push(statement);

                        args_containing_set.push(set_arg.clone());
                    } else {
                        let opaque_ident = Ident::new(&format!("S{index_opaque}"), ident.span());

                        let opaque_param = {
                            let mut bounds = Punctuated::new();

                            let path = Path {
                                leading_colon: None,
                                segments: segments!("dirk", "InputStatus"),
                            };
                            let trait_bound = TraitBound {
                                paren_token: None,
                                modifier: syn::TraitBoundModifier::None,
                                lifetimes: None,
                                path,
                            };
                            let bound = TypeParamBound::Trait(trait_bound);
                            bounds.push(bound);

                            let type_param = TypeParam {
                                attrs: Vec::new(),
                                ident: opaque_ident.clone(),
                                colon_token: None,
                                bounds,
                                eq_token: None,
                                default: None,
                            };
                            GenericParam::Type(type_param)
                        };
                        args_pure.push(opaque_param);

                        let opaque_arg = {
                            let path = Path::from(opaque_ident);
                            let type_path = TypePath { qself: None, path };
                            let opaque_ty = Type::Path(type_path);
                            GenericArgument::Type(opaque_ty)
                        };
                        args_containing_unset.push(opaque_arg.clone());
                        args_containing_set.push(opaque_arg.clone());

                        let member = Member::Named(ident.clone());
                        let expr_path = ExprPath {
                            attrs: Vec::new(),
                            qself: None,
                            path: Path::from(Ident::new("self", ident.span())),
                        };
                        let base = Expr::Path(expr_path);

                        let expr_field = ExprField {
                            attrs: Vec::new(),
                            base: Box::new(base),
                            dot_token: Dot::default(),
                            member,
                        };
                        let expr = Expr::Field(expr_field);

                        let init = LocalInit {
                            eq_token: Eq::default(),
                            expr: Box::new(expr),
                            diverge: None,
                        };

                        let pat_ident = PatIdent {
                            attrs: Vec::new(),
                            by_ref: None,
                            mutability: None,
                            ident: ident.clone(),
                            subpat: None,
                        };
                        let pat = syn::Pat::Ident(pat_ident);

                        let local = Local {
                            attrs: Vec::new(),
                            let_token: Let::default(),
                            pat,
                            init: Some(init),
                            semi_token: Semi::default(),
                        };

                        let statement = Stmt::Local(local);
                        statements_opaque.push(statement);
                    }
                }

                let partial_impl = {
                    let generics_containing_set = {
                        if args_containing_set.is_empty() {
                            PathArguments::None
                        } else {
                            let angle_bracketed = AngleBracketedGenericArguments {
                                colon2_token: None,
                                lt_token: Lt::default(),
                                args: args_containing_set,
                                gt_token: Gt::default(),
                            };
                            PathArguments::AngleBracketed(angle_bracketed)
                        }
                    };
                    let generics_containing_unset = {
                        if args_containing_unset.is_empty() {
                            PathArguments::None
                        } else {
                            let angle_bracketed = AngleBracketedGenericArguments {
                                colon2_token: None,
                                lt_token: Lt::default(),
                                args: args_containing_unset,
                                gt_token: Gt::default(),
                            };
                            PathArguments::AngleBracketed(angle_bracketed)
                        }
                    };

                    let generics_pure = {
                        let (lt, gt) = {
                            if args_pure.is_empty() {
                                (None, None)
                            } else {
                                (Some(Lt::default()), Some(Gt::default()))
                            }
                        };
                        Generics {
                            lt_token: lt,
                            params: args_pure,
                            gt_token: gt,
                            where_clause: None,
                        }
                    };

                    let generics_partial = {
                        let maybe_generic_param = ty
                            .as_path()
                            .ok()
                            .and_then(|p| p.path.get_ident())
                            .and_then(|ty_ident| unbound_generics_mapping.get(ty_ident));

                        if let Some(generic_param) = maybe_generic_param {
                            let mut params = Punctuated::new();
                            params.push(generic_param.clone());
                            Generics {
                                lt_token: Some(Lt::default()),
                                params,
                                gt_token: Some(Gt::default()),
                                where_clause: None,
                            }
                        } else {
                            Generics {
                                lt_token: None,
                                params: Punctuated::new(),
                                gt_token: None,
                                where_clause: None,
                            }
                        }
                    };

                    parse_quote! {
                        impl #generics_pure #builder_ident #generics_containing_unset {
                            fn #ident #generics_partial (self, #ident: #ty) -> #builder_ident #generics_containing_set {
                                #(#statements_opaque)*
                                #builder_ident {
                                    #builder_field_values
                                }
                            }
                        }

                    }
                };

                partial_impls.push(partial_impl);
            }

            partial_impls
        }
    };

    let impl_builder_unset = parse_quote! {
        impl #builder_ident #unset_generics {
            fn new () -> Self {
                #(#builder_statements)*
                #builder_ident { #builder_field_values }
            }
        }
    };

    let impl_builder_set = {
        let instance_binds = instance_binds.clone();

        let mut unwrap_statements = Vec::new();
        let mut providers_actual: Punctuated<Expr, Comma> = Punctuated::new();

        for (ident, binding) in instance_binds {
            let unwrap_statement = {
                let path = Path {
                    leading_colon: None,
                    segments: segments!("dirk", "Set"),
                };

                let mut elems = Punctuated::new();
                let pat_ident = PatIdent {
                    attrs: Vec::new(),
                    by_ref: None,
                    mutability: None,
                    ident: ident.clone(),
                    subpat: None,
                };
                let pat = Pat::Ident(pat_ident);
                elems.push(pat);

                let pat_tuple_struct = PatTupleStruct {
                    attrs: Vec::new(),
                    qself: None,
                    path,
                    paren_token: Paren::default(),
                    elems,
                };
                let pat = Pat::TupleStruct(pat_tuple_struct);

                let expr_path = ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: Path::from(Ident::new("self", ident.span())),
                };
                let base = Expr::Path(expr_path);

                let member = Member::Named(ident.clone());

                let expr_field = ExprField {
                    attrs: Vec::new(),
                    base: Box::new(base),
                    dot_token: Dot::default(),
                    member,
                };
                let expr = Expr::Field(expr_field);

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
                Stmt::Local(local)
            };
            unwrap_statements.push(unwrap_statement);

            let provider = binding.get_new_factory(ident);
            providers_actual.push(provider);
        }

        let impl_set = parse_quote! {
            impl #generics_unbound_formal #builder_ident #set_generics {
                fn build(self) -> impl #trait_ident #generics_trait {
                    #(#unwrap_statements)*
                    #impl_path::new(#providers_actual)
                }
            }
        };
        impl_set
    };
    let dirk_impl_component = parse_quote! {
        impl dirk::DirkComponent<#builder_ident #unset_generics> for #dirk_ident {
            fn builder() -> #builder_ident #unset_generics {
                #builder_ident::new()
            }
        }
    };

    (
        impl_builder_unset,
        partials,
        impl_builder_set,
        dirk_impl_component,
    )
}

pub(crate) fn get_builder<'bindings>(
    trait_ident: &Ident,
    bindings: &HashMap<&'bindings Ident, &'bindings Binding>,
) -> (
    Ident,
    Generics,
    Punctuated<Field, Comma>,
    Punctuated<FieldValue, Comma>,
    Vec<Stmt>,
) {
    let instance_binds = bindings
        .iter()
        .filter_map(|(i, b)| b.kind().as_manual().map(|m| (*i, m)))
        .peekable(); // TODO: maybe sorted

    let builder_path = get_dirk_name(trait_ident, Some("Builder"));

    let mut generic_params = Punctuated::new();
    let mut fields = Punctuated::new();
    let mut field_values = Punctuated::new();
    let mut statements = Vec::new();

    let input_status_bound = {
        let path = Path {
            leading_colon: None,
            segments: segments!("dirk", "InputStatus"),
        };
        let trait_bound = TraitBound {
            paren_token: None,
            modifier: syn::TraitBoundModifier::None,
            lifetimes: None,
            path,
        };
        TypeParamBound::Trait(trait_bound)
    };
    for (ident, _instanc_bind) in instance_binds {
        let param_ident = Ident::new(&format!("_{ident}"), ident.span()); // TODO: to upper camel case

        let mut bounds = Punctuated::new();
        bounds.push(input_status_bound.clone());
        let type_param = TypeParam {
            attrs: Vec::new(),
            ident: param_ident.clone(),
            colon_token: Some(Colon::default()),
            bounds,
            eq_token: None,
            default: None,
        };
        let generic_param = GenericParam::Type(type_param);
        generic_params.push(generic_param);

        let path = Path::from(param_ident);
        let type_path = TypePath { qself: None, path };
        let ty = Type::Path(type_path);
        let field = Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            mutability: syn::FieldMutability::None,
            ident: Some(ident.clone()),
            colon_token: Some(Colon::default()),
            ty: ty.clone(),
        };

        fields.push(field);

        let field_value = {
            let member = Member::Named(ident.clone());
            let expr = syn::Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: Path::from(ident.clone()),
            });
            FieldValue {
                attrs: Vec::new(),
                member,
                colon_token: None,
                expr,
            }
        };

        field_values.push(field_value);

        let path = Path {
            leading_colon: None,
            segments: segments!("dirk", "Unset"),
        };
        let expr_path = ExprPath {
            attrs: Vec::new(),
            qself: None,
            path,
        };
        let expr_unset = Expr::Path(expr_path);

        let init = LocalInit {
            eq_token: Eq::default(),
            expr: Box::new(expr_unset),
            diverge: None,
        };

        let pat_ident = PatIdent {
            attrs: Vec::new(),
            by_ref: None,
            mutability: None,
            ident: ident.clone(),
            subpat: None,
        };
        let pat = syn::Pat::Ident(pat_ident);

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

    let generics = {
        let (lt, gt) = {
            if generic_params.is_empty() {
                (None, None)
            } else {
                (Some(Lt::default()), Some(Gt::default()))
            }
        };
        Generics {
            lt_token: lt,
            params: generic_params,
            gt_token: gt,
            where_clause: None,
        }
    };

    (builder_path, generics, fields, field_values, statements)
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
                let param_ident = &param.as_type()?.ident;
                let maybe_unbound_param = value
                    .as_path()
                    .ok()
                    .and_then(|p| p.path.get_ident())
                    .filter(|i| *i == param_ident);

                if maybe_unbound_param.is_none() {
                    map.insert(param.clone(), value.clone());
                }
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
    HashMap<Ident, GenericParam>,
) {
    let mut params_trait = Punctuated::new();
    let mut params_unbound_formal = Punctuated::new();
    let mut params_unbound_actual = Punctuated::new();
    let mut unbound_generics_mapping = HashMap::new();

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

            if let Ok(type_param) = param.as_type() {
                let ident = &type_param.ident;
                unbound_generics_mapping.insert(ident.clone(), param.clone());
            }
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
        unbound_generics_mapping,
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
