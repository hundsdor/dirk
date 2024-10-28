use std::{cell::OnceCell, collections::HashMap};

use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro::TokenStream;

use proc_macro2::Span;

use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{
        Brace, Bracket, Colon, Comma, Dot, Eq, For, Gt, Impl, Let, Lt, Paren, Pound, RArrow,
        SelfValue, Semi, Struct,
    },
    AngleBracketedGenericArguments, Attribute, Block, Expr, ExprCall, ExprField, ExprPath,
    ExprStruct, Field, FieldValue, Fields, FieldsNamed, FnArg, GenericArgument, GenericParam,
    Generics, Ident, ImplItem, ImplItemFn, Item, ItemImpl, ItemStruct, ItemTrait, Local, LocalInit,
    Member, Meta, MetaList, Pat, PatIdent, PatTupleStruct, PatType, Path, PathArguments,
    PathSegment, Receiver, ReturnType, Stmt, TraitBound, Type, TypeParam, TypeParamBound, TypePath,
};

use crate::{
    component::error::ComponentLogicAbort,
    errors::{InfallibleError, InfallibleResult},
    expectable::{
        GenericParamExpectable, ReturnTypeExpectable, TraitItemExpectable, TypeExpectable,
    },
    syntax::{mk_fn, wrap_path},
    util::{
        path_builder, path_component, path_input_status, path_self, path_set, path_static_builder,
        path_static_component, path_unset, type_set, type_unset,
    },
};

use super::{
    binding::{manual::ManualBindingKind, Binding},
    error::{ComponentResult, ComponentSyntaxError},
    syntax::{
        generic_argument_from_generic_param, get_dirk_name, get_provider_call, get_providers,
    },
    ComponentMacroInput,
};

#[derive(Debug)]
pub(crate) struct ComponentMacroData {
    attr: TokenStream,
    item: TokenStream,

    helper_attribute: OnceCell<Attribute>,

    input_macro: OnceCell<ComponentMacroInput>,
    input_trait: OnceCell<ItemTrait>,
}

impl ComponentMacroData {
    pub(crate) fn new(attr: TokenStream, item: TokenStream) -> Self {
        Self {
            attr,
            item,

            helper_attribute: OnceCell::new(),

            input_macro: OnceCell::new(),
            input_trait: OnceCell::new(),
        }
    }
}

impl ComponentMacroData {
    pub(crate) fn is_helper(&self) -> InfallibleResult<bool, ComponentSyntaxError> {
        Ok(self.input_macro()?.inner.is_some())
    }

    fn helper_attribute(&self) -> &Attribute {
        if let Some(cached) = self.helper_attribute.get() {
            return cached;
        }

        let attr = {
            let attr = self.attr.clone();

            let mut segments = Punctuated::new();
            segments.push(Ident::new("dirk", Span::call_site()).into());
            segments.push(Ident::new("component", Span::call_site()).into());

            let path = Path {
                leading_colon: None,
                segments,
            };

            let mut tokens: proc_macro2::TokenStream = ComponentMacroInput::inner_marker();
            tokens.extend(std::iter::once(std::convert::Into::<
                proc_macro2::TokenStream,
            >::into(attr)));

            let meta_list = MetaList {
                path,
                delimiter: syn::MacroDelimiter::Paren(Paren::default()),
                tokens,
            };
            let meta = Meta::List(meta_list);

            Attribute {
                pound_token: Pound::default(),
                style: syn::AttrStyle::Outer,
                bracket_token: Bracket::default(),
                meta,
            }
        };

        self.helper_attribute.get_or_init(|| attr)
    }

    fn input_macro(&self) -> InfallibleResult<&ComponentMacroInput, ComponentSyntaxError> {
        if let Some(cached) = self.input_macro.get() {
            return Ok(cached);
        }

        let input_macro = {
            let attr = self.attr.clone();

            syn::parse::<ComponentMacroInput>(attr)
                .map_err(ComponentSyntaxError::FailedToParseInput)?
        };

        Ok(self.input_macro.get_or_init(|| input_macro))
    }

    fn input_trait(&self) -> InfallibleResult<&ItemTrait, ComponentSyntaxError> {
        if let Some(cached) = self.input_trait.get() {
            return Ok(cached);
        }

        let input_trait = {
            let item = self.item.clone();

            syn::parse::<ItemTrait>(item).map_err(ComponentSyntaxError::ExpectedTrait)?
        };

        Ok(self.input_trait.get_or_init(|| input_trait))
    }
}

pub(crate) struct InfallibleComponentMacroProcessor<'data> {
    data: &'data ComponentMacroData,

    trait_ident: OnceCell<&'data Ident>,
    dirk_ident: OnceCell<Ident>, // TODO: maybe convert to TypePath
}

impl<'data> InfallibleComponentMacroProcessor<'data> {
    pub(crate) fn new(data: &'data ComponentMacroData) -> Self {
        Self {
            data,

            trait_ident: OnceCell::new(),
            dirk_ident: OnceCell::new(),
        }
    }
}

impl<'data> InfallibleComponentMacroProcessor<'data> {
    fn trait_ident(&self) -> InfallibleResult<&Ident, ComponentSyntaxError> {
        if let Some(cached) = self.trait_ident.get() {
            return Ok(cached);
        }

        let trait_ident = {
            let input_trait = self.data.input_trait()?;
            &input_trait.ident
        };

        Ok(self.trait_ident.get_or_init(|| trait_ident))
    }

    fn dirk_ident(&self) -> InfallibleResult<&Ident, ComponentSyntaxError> {
        if let Some(cached) = self.dirk_ident.get() {
            return Ok(cached);
        }

        let dirk_ident = {
            let trait_ident = self.trait_ident()?;

            get_dirk_name(trait_ident, None)
        };

        Ok(self.dirk_ident.get_or_init(|| dirk_ident))
    }

    pub(crate) fn process(self) -> InfallibleResult<Vec<Item>, ComponentSyntaxError> {
        let mut input_trait = self.data.input_trait()?.clone();
        let attr = self.data.helper_attribute().clone();

        let dirk_ident = self.dirk_ident()?;

        input_trait.attrs.push(attr);

        let dirk_struct = ItemStruct {
            attrs: Vec::new(),
            vis: input_trait.vis.clone(),
            struct_token: Struct::default(),
            ident: dirk_ident.clone(),
            generics: Generics::default(),
            fields: syn::Fields::Unit,
            semi_token: Some(Semi::default()),
        };

        Ok(vec![Item::Struct(dirk_struct), Item::Trait(input_trait)])
    }
}

pub(crate) struct ComponentMacroProcessor<'data> {
    data: &'data ComponentMacroData,
    delegate: InfallibleComponentMacroProcessor<'data>,

    impl_ident: OnceCell<Ident>,
    impl_ty: OnceCell<Type>,

    bindings: OnceCell<HashMap<&'data Ident, &'data Binding>>,
    generics_mapping: OnceCell<HashMap<&'data GenericParam, &'data Type>>,

    unbound_generics: OnceCell<HashMap<&'data Ident, &'data GenericParam>>,

    generics_unbound: OnceCell<Generics>,
}

impl<'data> ComponentMacroProcessor<'data> {
    pub(crate) fn new(data: &'data ComponentMacroData) -> Self {
        Self {
            data,
            delegate: InfallibleComponentMacroProcessor::new(data),

            impl_ident: OnceCell::new(),
            impl_ty: OnceCell::new(),

            bindings: OnceCell::new(),
            generics_mapping: OnceCell::new(),

            unbound_generics: OnceCell::new(),

            generics_unbound: OnceCell::new(),
        }
    }
}

impl<'data> ComponentMacroProcessor<'data> {
    fn trait_ident(&self) -> ComponentResult<&Ident> {
        self.delegate.trait_ident().map_err(Into::into)
    }

    fn trait_path(&self) -> ComponentResult<Path> {
        let trait_ident = self.trait_ident()?;
        let generic_args_trait = self.generic_args_trait()?;

        let mut segments = Punctuated::new();
        let segment = PathSegment {
            ident: trait_ident.clone(),
            arguments: syn::PathArguments::AngleBracketed(generic_args_trait.clone()),
        };
        segments.push(segment);

        let path = Path {
            leading_colon: None,
            segments,
        };

        Ok(path)
    }

    fn dirk_ty(&self) -> ComponentResult<Type> {
        let dirk_ty = {
            let dirk_ident = self.delegate.dirk_ident()?;

            let path = Path::from(dirk_ident.clone());
            let type_path = TypePath { qself: None, path };

            Type::Path(type_path)
        };

        Ok(dirk_ty)
    }

    fn impl_ident(&self) -> ComponentResult<&Ident> {
        if let Some(cached) = self.impl_ident.get() {
            return Ok(cached);
        }

        let impl_ident = {
            let trait_ident = self.trait_ident()?;
            get_dirk_name(trait_ident, Some("Impl"))
        };

        Ok(self.impl_ident.get_or_init(|| impl_ident))
    }

    fn impl_path(&self) -> ComponentResult<Path> {
        let impl_path = {
            let impl_ident = self.impl_ident()?;

            let mut segments = Punctuated::new();
            let segment = PathSegment {
                ident: impl_ident.clone(),
                arguments: PathArguments::None,
            };
            segments.push(segment);

            Path {
                leading_colon: None,
                segments,
            }
        };

        Ok(impl_path)
    }

    fn impl_ty(&self) -> ComponentResult<&Type> {
        if let Some(cached) = self.impl_ty.get() {
            return Ok(cached);
        }

        let impl_ty = {
            let impl_ident = self.impl_ident()?;
            let generics_unbound_actual = self.generic_args_unbound()?;

            let mut segments = Punctuated::new();
            let segment = PathSegment {
                ident: impl_ident.clone(),
                arguments: generics_unbound_actual.clone(),
            };
            segments.push(segment);

            let path = Path {
                leading_colon: None,
                segments,
            };

            let type_path = TypePath { qself: None, path };

            Type::Path(type_path)
        };

        Ok(self.impl_ty.get_or_init(|| impl_ty))
    }

    fn bindings(&self) -> ComponentResult<&HashMap<&'data Ident, &'data Binding>> {
        if let Some(cached) = self.bindings.get() {
            return Ok(cached);
        }

        let bindings = {
            let input_macro = self.data.input_macro()?;

            input_macro
                .bindings
                .iter()
                .map(|b| (b.identifier(), b))
                .collect::<HashMap<_, _>>()
        };

        Ok(self.bindings.get_or_init(|| bindings))
    }

    fn generics_mapping(&self) -> ComponentResult<&HashMap<&'data GenericParam, &'data Type>> {
        if let Some(cached) = self.generics_mapping.get() {
            return Ok(cached);
        }

        let generics_mapping = {
            let input_trait = self.data.input_trait()?;
            let bindings = self.bindings()?;

            let map_arguments = {
                let funs = input_trait.items.iter().filter_map(|i| i.as_fn().ok());

                funs.map(|fun| {
                    let name = &fun.sig.ident;
                    let ty = &fun.sig.output.as_type()?;

                    bindings
                        .get(name)
                        .ok_or_else(|| ComponentLogicAbort::NotFound(name.clone()))
                        .map_err(Into::into)
                        .and_then(|binding| binding.kind().compare_types(ty.1))
                        .map(std::iter::IntoIterator::into_iter)
                })
                .collect::<ComponentResult<Vec<_>>>()?
                .into_iter()
                .flatten()
                .collect::<HashMap<_, _>>()
            };

            input_trait
                .generics
                .params
                .iter()
                .filter_map(|param| {
                    param
                        .as_type()
                        .map(|ty| {
                            let mut segments = Punctuated::new();
                            let segment = PathSegment {
                                ident: ty.ident.clone(),
                                arguments: PathArguments::None,
                            };
                            segments.push(segment);
                            let path = Path {
                                leading_colon: None,
                                segments,
                            };
                            let type_path = TypePath { qself: None, path };
                            let key = Type::Path(type_path);

                            map_arguments.get(&key).map(|value| (param, value))
                        })
                        .transpose()
                })
                .filter_map(|r| {
                    r.and_then(|(param, value)| param.as_type().map(|ty| (param, value, ty)))
                        .ok()
                        .and_then(|(param, value, ty)| {
                            let maybe_unbound_param = value
                                .as_path()
                                .ok()
                                .and_then(|p| p.path.get_ident())
                                .filter(|i| *i == (&ty.ident));

                            if maybe_unbound_param.is_none() {
                                Some((param, *value))
                            } else {
                                None
                            }
                        })
                })
                .collect()
        };

        Ok(self.generics_mapping.get_or_init(|| generics_mapping))
    }

    fn generic_args_trait(&self) -> ComponentResult<AngleBracketedGenericArguments> {
        let input_trait = self.data.input_trait()?;
        let generics_mapping = self.generics_mapping()?;

        let mut params_trait = Punctuated::new();

        for param in &input_trait.generics.params {
            if let Some(ty) = generics_mapping.get(param) {
                // bound to ty
                let arg = GenericArgument::Type((*ty).clone());
                params_trait.push(arg);
            } else {
                // unbound
                let actual = generic_argument_from_generic_param(param);
                params_trait.push(actual);
            }
        }

        Ok(AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: Lt::default(),
            args: params_trait,
            gt_token: Gt::default(),
        })
    }

    fn unbound_generics(&self) -> ComponentResult<&HashMap<&Ident, &GenericParam>> {
        if let Some(cached) = self.unbound_generics.get() {
            return Ok(cached);
        }

        let unbound_generics = {
            let input_trait = self.data.input_trait()?;
            let generics_mapping = self.generics_mapping()?;

            {
                input_trait
                    .generics
                    .params
                    .iter()
                    .filter(|param| generics_mapping.get(param).is_none())
                    .filter_map(|param| {
                        param
                            .as_type()
                            .ok()
                            .map(|type_param| (&type_param.ident, param))
                    })
                    .collect()
            }
        };

        Ok(self.unbound_generics.get_or_init(|| unbound_generics))
    }

    fn generics_unbound(&self) -> ComponentResult<&Generics> {
        if let Some(cached) = self.generics_unbound.get() {
            return Ok(cached);
        }

        let generics_unbound = {
            let input_trait = self.data.input_trait()?;
            let generics_mapping = self.generics_mapping()?;

            let params_unbound = input_trait
                .generics
                .params
                .iter()
                .filter(|p| generics_mapping.get(p).is_none())
                .cloned()
                .collect::<Punctuated<_, _>>();

            Generics {
                lt_token: Some(Lt::default()),
                params: params_unbound,
                gt_token: Some(Gt::default()),
                where_clause: None, // TODO: include where clause
            }
        };

        Ok(self.generics_unbound.get_or_init(|| generics_unbound))
    }

    fn generic_args_unbound(&self) -> ComponentResult<PathArguments> {
        let generic_args_unbound = {
            let input_trait = self.data.input_trait()?;
            let generics_mapping = self.generics_mapping()?;

            let params_unbound_actual = input_trait
                .generics
                .params
                .iter()
                .filter(|p| generics_mapping.get(p).is_none())
                .map(generic_argument_from_generic_param)
                .collect::<Punctuated<_, _>>();

            if params_unbound_actual.is_empty() {
                PathArguments::None
            } else {
                let angle_bracketed = AngleBracketedGenericArguments {
                    colon2_token: None,
                    lt_token: Lt::default(),
                    args: params_unbound_actual,
                    gt_token: Gt::default(),
                };
                PathArguments::AngleBracketed(angle_bracketed)
            }
        };

        Ok(generic_args_unbound)
    }

    fn functions(&self) -> ComponentResult<Vec<ImplItem>> {
        let functions = {
            let input_trait = self.data.input_trait()?;
            let bindings = self.bindings()?;

            let fns = input_trait
                .items
                .iter()
                .map(|i| i.as_fn().map_err(std::convert::Into::into))
                .collect::<ComponentResult<Vec<_>>>()?;

            let mut res = Vec::new();

            for function in fns {
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

            res
        };

        Ok(functions)
    }

    fn builder_kind(&self) -> ComponentResult<ComponentBuilderKind> {
        let builder_data = ComponentBuilderData::new(self.bindings()?, self.trait_ident()?);
        ComponentBuilderKind::evaluate(&builder_data, self)
    }

    pub(crate) fn process(self) -> ComponentResult<Vec<Item>> {
        let bindings = self.bindings()?;

        let impl_ident = self.impl_ident()?;
        let impl_ty = self.impl_ty()?;

        let trait_path = self.trait_path()?;

        let functions = self.functions()?;

        let generics_unbound_formal = self.generics_unbound()?;

        let (providers_signature, providers_actual, providers_formal, providers_instantiation) =
            get_providers(bindings)?;

        let items = {
            let input_trait = self.data.input_trait()?.clone();
            let trait_visibility = input_trait.vis.clone();

            let fields_named = FieldsNamed {
                brace_token: Brace::default(),
                named: providers_signature,
            };

            let struct_impl = ItemStruct {
                attrs: Vec::new(),
                vis: trait_visibility,
                struct_token: Struct::default(),
                ident: impl_ident.clone(),
                generics: generics_unbound_formal.clone(),
                fields: syn::Fields::Named(fields_named),
                semi_token: None,
            };

            let impl_impl = {
                let span = trait_path.span();

                let self_path = path_self(PathArguments::None, span);
                let type_path = TypePath {
                    qself: None,
                    path: self_path.clone(),
                };
                let self_ty = Type::Path(type_path);

                let mut stmts = providers_instantiation;
                let expr_struct = ExprStruct {
                    attrs: Vec::new(),
                    qself: None,
                    path: self_path,
                    brace_token: Brace::default(),
                    fields: providers_actual,
                    dot2_token: None,
                    rest: None,
                };
                let self_struct = Expr::Struct(expr_struct);
                stmts.push(Stmt::Expr(self_struct, None));

                let block = Block {
                    brace_token: Brace::default(),
                    stmts,
                };

                let new_fn = mk_fn(
                    Ident::new("new", span),
                    syn::Visibility::Inherited,
                    Generics::default(),
                    providers_formal,
                    self_ty,
                    block,
                );

                ItemImpl {
                    attrs: Vec::new(),
                    defaultness: None,
                    unsafety: None,
                    impl_token: Impl::default(),
                    generics: generics_unbound_formal.clone(),
                    trait_: None,
                    self_ty: Box::new(impl_ty.clone()),
                    brace_token: Brace::default(),
                    items: vec![new_fn],
                }
            };

            let trait_impl = ItemImpl {
                attrs: Vec::new(),
                defaultness: None,
                unsafety: None,
                impl_token: Impl::default(),
                generics: generics_unbound_formal.clone(),
                trait_: Some((None, trait_path, For::default())),
                self_ty: Box::new(impl_ty.clone()),
                brace_token: Brace::default(),
                items: functions,
            };

            let builder = self.builder_kind()?;

            let mut items = Vec::new();

            match builder {
                ComponentBuilderKind::StaticBuilder {
                    struct_builder,
                    impl_unset,
                    impl_builder_set,
                    impl_static_builder,
                    impl_component,
                    impl_static_component,
                } => {
                    items.push(Item::Struct(struct_builder));
                    items.push(Item::Impl(impl_unset));
                    items.push(Item::Impl(impl_builder_set));
                    items.push(Item::Impl(impl_static_builder));
                    items.push(Item::Impl(impl_component));
                    items.push(Item::Impl(impl_static_component));
                }
                ComponentBuilderKind::NonStaticBuilder {
                    struct_builder,
                    impl_unset,
                    impl_builder_unset,
                    impl_builder_set,
                    partial_impls,
                    impl_static_builder,
                    impl_component,
                } => {
                    items.push(Item::Struct(struct_builder));
                    items.push(Item::Impl(impl_unset));
                    items.push(Item::Impl(impl_builder_unset));
                    items.push(Item::Impl(impl_builder_set));
                    items.extend(partial_impls.into_iter().map(Item::Impl));
                    items.push(Item::Impl(impl_static_builder));
                    items.push(Item::Impl(impl_component));
                }
            }

            items.push(Item::Struct(struct_impl));
            items.push(Item::Impl(impl_impl));
            items.push(Item::Impl(trait_impl));
            items.push(Item::Trait(input_trait));

            items
        };

        Ok(items)
    }
}

enum ComponentBuilderKind {
    StaticBuilder {
        struct_builder: ItemStruct,
        impl_unset: ItemImpl,
        impl_builder_set: ItemImpl,
        impl_static_builder: ItemImpl,
        impl_component: ItemImpl,
        impl_static_component: ItemImpl,
    },
    NonStaticBuilder {
        struct_builder: ItemStruct,
        impl_unset: ItemImpl,
        impl_builder_unset: ItemImpl,
        impl_builder_set: ItemImpl,
        partial_impls: Vec<ItemImpl>,
        impl_static_builder: ItemImpl,
        impl_component: ItemImpl,
    },
}

impl ComponentBuilderKind {
    fn evaluate(
        builder_data: &ComponentBuilderData,
        data: &ComponentMacroProcessor,
    ) -> ComponentResult<Self> {
        let input_trait = data.data.input_trait()?;
        let trait_visibility = &input_trait.vis;

        let dirk_ty = data.dirk_ty()?;

        let impl_path = data.impl_path()?;
        let impl_ty = data.impl_ty()?;

        let generics_unbound_formal = data.generics_unbound()?;
        let unbound_generics_mapping = data.unbound_generics()?;

        let builder_ident = builder_data.builder_ident();
        let builder_path = builder_data.builder_path();
        let builder_generics = builder_data.builder_generics();
        let builder_fields = builder_data.builder_fields();
        let builder_field_values = builder_data.builder_field_values();
        let builder_statements = builder_data.builder_statements();

        let instance_binds = builder_data.instance_binds();

        let struct_builder = {
            let fields_named = FieldsNamed {
                brace_token: Brace::default(),
                named: builder_fields,
            };
            ItemStruct {
                attrs: Vec::new(),
                vis: trait_visibility.clone(),
                struct_token: Struct::default(),
                ident: builder_ident.clone(),
                generics: builder_generics,
                fields: Fields::Named(fields_named),
                semi_token: None,
            }
        };

        let builder_kind = {
            let instance_binds = instance_binds.clone().into_iter().peekable();

            let (unset_generics, set_generics) = {
                let mut instance_binds = instance_binds.clone();
                if instance_binds.peek().is_none() {
                    (PathArguments::None, PathArguments::None)
                } else {
                    let instance_binds = instance_binds.clone();

                    let mut unset_args = Punctuated::new();
                    let mut set_args = Punctuated::new();

                    for (_ident, binding) in instance_binds {
                        let ty = binding.ty()?;

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

                        let unset_arg =
                            GenericArgument::Type(type_unset(PathArguments::None, ty.span()));
                        let set_arg = GenericArgument::Type(type_set(set_generics, ty.span()));

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

            let impl_unset = {
                let span = builder_ident.span();

                let builder_ty = builder_data.builder_ty(unset_generics.clone());

                let self_path = path_self(PathArguments::None, span);
                let type_path = TypePath {
                    qself: None,
                    path: self_path.clone(),
                };
                let self_ty = Type::Path(type_path);

                let mut stmts = builder_statements;
                let expr_struct = ExprStruct {
                    attrs: Vec::new(),
                    qself: None,
                    path: Path::from(builder_ident.clone()),
                    brace_token: Brace::default(),
                    fields: builder_field_values.clone(),
                    dot2_token: None,
                    rest: None,
                };
                let builder_struct = Expr::Struct(expr_struct);
                stmts.push(Stmt::Expr(builder_struct, None));

                let block = Block {
                    brace_token: Brace::default(),
                    stmts,
                };
                let new_fn = mk_fn(
                    Ident::new("new", span),
                    syn::Visibility::Inherited,
                    Generics::default(),
                    Punctuated::new(),
                    self_ty,
                    block,
                );

                ItemImpl {
                    attrs: Vec::new(),
                    defaultness: None,
                    unsafety: None,
                    impl_token: Impl::default(),
                    generics: Generics::default(),
                    trait_: None,
                    self_ty: Box::new(builder_ty),
                    brace_token: Brace::default(),
                    items: vec![new_fn],
                }
            };

            let impl_static_builder = {
                let instance_binds = instance_binds.clone();

                let mut unwrap_statements = Vec::new();
                let mut providers_actual: Punctuated<Expr, Comma> = Punctuated::new();

                for (ident, binding) in instance_binds {
                    let unwrap_statement = {
                        let path = path_set(PathArguments::None, ident.span());

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

                let span = builder_ident.span();

                let mut args = Punctuated::new();
                let arg = GenericArgument::Type(impl_ty.clone());
                args.push(arg);
                let angle_bracketed = AngleBracketedGenericArguments {
                    colon2_token: None,
                    lt_token: Lt::default(),
                    args,
                    gt_token: Gt::default(),
                };
                let arguments = PathArguments::AngleBracketed(angle_bracketed);
                let static_builder_path = path_static_builder(arguments, span);

                let builder_ty = builder_data.builder_ty(set_generics.clone());

                let build_fn = {
                    let type_path = TypePath {
                        qself: None,
                        path: path_self(PathArguments::None, span),
                    };
                    let self_ty = Type::Path(type_path);

                    let mut inputs = Punctuated::new();
                    let self_arg = FnArg::Receiver(Receiver {
                        attrs: Vec::new(),
                        reference: None,
                        mutability: None,
                        self_token: SelfValue::default(),
                        colon_token: None,
                        ty: Box::new(self_ty),
                    });
                    inputs.push(self_arg);

                    let mut stmts = unwrap_statements;
                    let mut path = impl_path.clone();
                    let new_segment = PathSegment::from(Ident::new("new", span));
                    path.segments.push(new_segment);
                    let expr_path = ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path,
                    };
                    let func = Expr::Path(expr_path);
                    let expr_call = ExprCall {
                        attrs: Vec::new(),
                        func: Box::new(func),
                        paren_token: Paren::default(),
                        args: providers_actual,
                    };
                    let expr = Expr::Call(expr_call);
                    let stmt = Stmt::Expr(expr, None);
                    stmts.push(stmt);
                    let block = Block {
                        brace_token: Brace::default(),
                        stmts,
                    };

                    mk_fn(
                        Ident::new("build", span),
                        syn::Visibility::Inherited,
                        Generics::default(),
                        inputs,
                        impl_ty.clone(),
                        block,
                    )
                };

                ItemImpl {
                    attrs: Vec::new(),
                    defaultness: None,
                    unsafety: None,
                    impl_token: Impl::default(),
                    generics: generics_unbound_formal.clone(),
                    trait_: Some((None, static_builder_path, For::default())),
                    self_ty: Box::new(builder_ty),
                    brace_token: Brace::default(),
                    items: vec![build_fn],
                }
            };

            let impl_builder_set = {
                let span = builder_ident.span();

                let builder_ty = builder_data.builder_ty(set_generics.clone());
                let builder_path = path_builder(PathArguments::None, span);
                ItemImpl {
                    attrs: Vec::new(),
                    defaultness: None,
                    unsafety: None,
                    impl_token: Impl::default(),
                    generics: generics_unbound_formal.clone(),
                    trait_: Some((None, builder_path, For::default())),
                    self_ty: Box::new(builder_ty),
                    brace_token: Brace::default(),
                    items: Vec::new(),
                }
            };

            let impl_component = {
                let span = builder_ident.span();

                let builder_ty = builder_data.builder_ty(unset_generics.clone());
                let component_path = wrap_path(builder_ty.clone(), path_component);

                let builder_fn = {
                    let mut path = Path::from(builder_ident.clone());
                    let new_segment = PathSegment::from(Ident::new("new", span));
                    path.segments.push(new_segment);
                    let expr_path = ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path,
                    };
                    let func = Expr::Path(expr_path);
                    let expr_call = ExprCall {
                        attrs: Vec::new(),
                        func: Box::new(func),
                        paren_token: Paren::default(),
                        args: Punctuated::new(),
                    };
                    let expr = Expr::Call(expr_call);
                    let stmt = Stmt::Expr(expr, None);
                    let block = Block {
                        brace_token: Brace::default(),
                        stmts: vec![stmt],
                    };

                    mk_fn(
                        Ident::new("builder", span),
                        syn::Visibility::Inherited,
                        Generics::default(),
                        Punctuated::new(),
                        builder_ty.clone(),
                        block,
                    )
                };

                ItemImpl {
                    attrs: Vec::new(),
                    defaultness: None,
                    unsafety: None,
                    impl_token: Impl::default(),
                    generics: Generics::default(),
                    trait_: Some((None, component_path, For::default())),
                    self_ty: Box::new(dirk_ty.clone()),
                    brace_token: Brace::default(),
                    items: vec![builder_fn],
                }
            };

            let mut instance_binds = instance_binds.clone();
            if instance_binds.peek().is_none() {
                let impl_static_component = {
                    let span = builder_ident.span();

                    let builder_ty = builder_data.builder_ty(unset_generics.clone());
                    let static_component_path = {
                        let impl_arg = GenericArgument::Type(impl_ty.clone());
                        let builder_arg = GenericArgument::Type(builder_ty.clone());

                        let mut args = Punctuated::new();
                        args.push(impl_arg);
                        args.push(builder_arg);

                        let generic_arguments = AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: Lt::default(),
                            args,
                            gt_token: Gt::default(),
                        };
                        path_static_component(
                            PathArguments::AngleBracketed(generic_arguments),
                            span,
                        )
                    };

                    ItemImpl {
                        attrs: Vec::new(),
                        defaultness: None,
                        unsafety: None,
                        impl_token: Impl::default(),
                        generics: Generics::default(),
                        trait_: Some((None, static_component_path, For::default())),
                        self_ty: Box::new(dirk_ty.clone()),
                        brace_token: Brace::default(),
                        items: Vec::new(),
                    }
                };

                Self::StaticBuilder {
                    struct_builder,
                    impl_unset,
                    impl_builder_set,
                    impl_static_builder,
                    impl_component,
                    impl_static_component,
                }
            } else {
                let impl_builder_unset = {
                    let span = builder_ident.span();

                    let builder_ty = builder_data.builder_ty(unset_generics.clone());
                    let builder_path = path_builder(PathArguments::None, span);
                    ItemImpl {
                        attrs: Vec::new(),
                        defaultness: None,
                        unsafety: None,
                        impl_token: Impl::default(),
                        generics: Generics::default(),
                        trait_: Some((None, builder_path, For::default())),
                        self_ty: Box::new(builder_ty),
                        brace_token: Brace::default(),
                        items: Vec::new(),
                    }
                };

                let mut partial_impls = Vec::new();

                for (index_set, (ident, binding)) in instance_binds.clone().enumerate() {
                    let ty = binding.ty()?;

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

                    let unset_arg =
                        GenericArgument::Type(type_unset(PathArguments::None, ty.span()));
                    let set_arg = GenericArgument::Type(type_set(set_generics, ty.span()));

                    let mut args_pure: Punctuated<GenericParam, Comma> = Punctuated::new();
                    let mut args_containing_unset: Punctuated<GenericArgument, Comma> =
                        Punctuated::new();
                    let mut args_containing_set: Punctuated<GenericArgument, Comma> =
                        Punctuated::new();

                    let mut statements_opaque = Vec::new();

                    for (index_opaque, (ident, _binding)) in instance_binds.clone().enumerate() {
                        if index_opaque == index_set {
                            args_containing_unset.push(unset_arg.clone());

                            let path = path_set(PathArguments::None, ident.span());
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
                            let opaque_ident =
                                Ident::new(&format!("S{index_opaque}"), ident.span());

                            let opaque_param = {
                                let mut bounds = Punctuated::new();

                                let path = path_input_status(PathArguments::None, ident.span());
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
                                params.push((*generic_param).clone());
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

                        let span = builder_ident.span();

                        let builder_ty_unset = builder_data.builder_ty(generics_containing_unset);
                        let builder_ty_set = builder_data.builder_ty(generics_containing_set);

                        let mut inputs = Punctuated::new();
                        let type_path = TypePath {
                            qself: None,
                            path: path_self(PathArguments::None, span),
                        };
                        let self_ty = Type::Path(type_path);
                        let self_arg = FnArg::Receiver(Receiver {
                            attrs: Vec::new(),
                            reference: None,
                            mutability: None,
                            self_token: SelfValue::default(),
                            colon_token: None,
                            ty: Box::new(self_ty),
                        });
                        inputs.push(self_arg);
                        let pat_ident = PatIdent {
                            attrs: Vec::new(),
                            by_ref: None,
                            mutability: None,
                            ident: ident.clone(),
                            subpat: None,
                        };
                        let pat = Pat::Ident(pat_ident);
                        let pat_type = PatType {
                            attrs: Vec::new(),
                            pat: Box::new(pat),
                            colon_token: Colon::default(),
                            ty: Box::new(ty.clone()),
                        };
                        let partial_arg = FnArg::Typed(pat_type);
                        inputs.push(partial_arg);

                        let mut stmts = statements_opaque;
                        let expr_struct = ExprStruct {
                            attrs: Vec::new(),
                            qself: None,
                            path: builder_path.clone(),
                            brace_token: Brace::default(),
                            fields: builder_field_values.clone(),
                            dot2_token: None,
                            rest: None,
                        };
                        let self_struct = Expr::Struct(expr_struct);
                        stmts.push(Stmt::Expr(self_struct, None));

                        let block = Block {
                            brace_token: Brace::default(),
                            stmts,
                        };

                        let partial_fn = mk_fn(
                            ident.clone(),
                            syn::Visibility::Inherited,
                            generics_partial,
                            inputs,
                            builder_ty_set,
                            block,
                        );

                        ItemImpl {
                            attrs: Vec::new(),
                            defaultness: None,
                            unsafety: None,
                            impl_token: Impl::default(),
                            generics: generics_pure,
                            trait_: None,
                            self_ty: Box::new(builder_ty_unset),
                            brace_token: Brace::default(),
                            items: vec![partial_fn],
                        }
                    };

                    partial_impls.push(partial_impl);
                }
                Self::NonStaticBuilder {
                    struct_builder,
                    impl_unset,
                    impl_builder_unset,
                    impl_builder_set,
                    partial_impls,
                    impl_static_builder,
                    impl_component,
                }
            }
        };

        Ok(builder_kind)
    }
}

struct ComponentBuilderData<'data, 'bindings: 'data> {
    bindings: &'data HashMap<&'bindings Ident, &'bindings Binding>,
    trait_ident: &'data Ident,

    instance_binds: OnceCell<Vec<(&'data Ident, &'data ManualBindingKind)>>,
    builder_ident: OnceCell<Ident>,
}

impl<'data, 'bindings: 'data> ComponentBuilderData<'data, 'bindings> {
    fn new(
        bindings: &'data HashMap<&'bindings Ident, &'bindings Binding>,
        trait_ident: &'data Ident,
    ) -> Self {
        Self {
            bindings,
            trait_ident,

            instance_binds: OnceCell::new(),
            builder_ident: OnceCell::new(),
        }
    }
}

impl<'data, 'bindings: 'data> ComponentBuilderData<'data, 'bindings> {
    fn param_ident(ident: &Ident) -> Ident {
        let content = &format!("_{ident}");
        let content = content.to_case(Case::Pascal);
        Ident::new(&content, ident.span())
    }

    fn instance_binds(&self) -> &Vec<(&'data Ident, &'data ManualBindingKind)> {
        if let Some(cached) = self.instance_binds.get() {
            return cached;
        }

        let instance_binds = {
            let bindings = self.bindings;

            bindings
                .iter()
                .sorted()
                .filter_map(|(i, b)| b.kind().as_manual().map(|m| (*i, m)))
                .collect()
        };

        self.instance_binds.get_or_init(|| instance_binds)
    }

    fn builder_ident(&self) -> &Ident {
        if let Some(cached) = self.builder_ident.get() {
            return cached;
        }

        let builder_path = {
            //TODO: maybe convert to TypePath
            let trait_ident = self.trait_ident;
            get_dirk_name(trait_ident, Some("Builder"))
        };

        self.builder_ident.get_or_init(|| builder_path)
    }

    fn builder_path(&self) -> Path {
        Path::from(self.builder_ident().clone())
    }

    fn builder_ty(&self, arguments: PathArguments) -> Type {
        let trait_ident = self.trait_ident;

        let builder_ident = get_dirk_name(trait_ident, Some("Builder"));

        let mut segments = Punctuated::new();
        let segment = PathSegment {
            ident: builder_ident,
            arguments,
        };
        segments.push(segment);

        let path = Path {
            leading_colon: None,
            segments,
        };

        let type_path = TypePath { qself: None, path };

        Type::Path(type_path)
    }

    fn builder_generics(&self) -> Generics {
        let instance_binds = self.instance_binds().iter().peekable();

        let mut generic_params = Punctuated::new();

        for (ident, _instanc_bind) in instance_binds {
            let mut bounds = Punctuated::new();
            let input_status_bound = {
                let path = path_input_status(PathArguments::None, ident.span());
                let trait_bound = TraitBound {
                    paren_token: None,
                    modifier: syn::TraitBoundModifier::None,
                    lifetimes: None,
                    path,
                };
                TypeParamBound::Trait(trait_bound)
            };
            bounds.push(input_status_bound.clone());
            let type_param = TypeParam {
                attrs: Vec::new(),
                ident: Self::param_ident(ident),
                colon_token: Some(Colon::default()),
                bounds,
                eq_token: None,
                default: None,
            };
            let generic_param = GenericParam::Type(type_param);
            generic_params.push(generic_param);
        }

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
    }

    fn builder_fields(&self) -> Punctuated<Field, Comma> {
        let instance_binds = self.instance_binds().clone().into_iter().peekable();

        let mut fields = Punctuated::new();

        for (ident, _instanc_bind) in instance_binds {
            let param_ident = Self::param_ident(ident);

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
        }

        fields
    }

    fn builder_field_values(&self) -> Punctuated<FieldValue, Comma> {
        let instance_binds = self.instance_binds().clone().into_iter().peekable();

        let mut field_values = Punctuated::new();

        for (ident, _instanc_bind) in instance_binds {
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
        }

        field_values
    }

    fn builder_statements(&self) -> Vec<Stmt> {
        let instance_binds = self.instance_binds().clone().into_iter().peekable();

        let mut statements = Vec::new();

        for (ident, _instanc_bind) in instance_binds {
            let path = path_unset(PathArguments::None, ident.span());
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

        statements
    }
}
