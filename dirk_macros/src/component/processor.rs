use std::{cell::OnceCell, collections::HashMap};

use proc_macro::TokenStream;

use proc_macro2::Span;

use syn::{
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{
        Brace, Bracket, Colon, Comma, Dot, Eq, Gt, Let, Lt, Paren, PathSep, Pound, RArrow, Semi,
    },
    AngleBracketedGenericArguments, Attribute, Block, Expr, ExprCall, ExprField, ExprPath, Field,
    FieldValue, GenericArgument, GenericParam, Generics, Ident, ImplItem, ImplItemFn, Item,
    ItemImpl, ItemStruct, ItemTrait, Local, LocalInit, Member, Meta, MetaList, Pat, PatIdent,
    PatTupleStruct, Path, PathArguments, PathSegment, ReturnType, Stmt, TraitBound, Type,
    TypeParam, TypeParamBound, TypePath,
};

use crate::{
    component::error::ComponentLogicAbort,
    errors::{InfallibleError, InfallibleResult},
    expectable::{
        GenericParamExpectable, ReturnTypeExpectable, TraitItemExpectable, TypeExpectable,
    },
    util::{segments, type_set, type_unset},
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
    fn helper_attribute(&self) -> &Attribute {
        if let Some(cached) = self.helper_attribute.get() {
            return cached;
        }

        let attr = {
            let attr = self.attr.clone();

            let mut segments = Punctuated::new();
            segments.push(Ident::new("dirk_macros", Span::call_site()).into());
            segments.push(Ident::new("__component", Span::call_site()).into());

            let path = Path {
                leading_colon: None,
                segments,
            };

            let meta_list = MetaList {
                path,
                delimiter: syn::MacroDelimiter::Paren(Paren::default()),
                tokens: attr.into(),
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

        let dirk_struct = parse_quote! {
            struct #dirk_ident {}
        };

        Ok(vec![Item::Struct(dirk_struct), Item::Trait(input_trait)])
    }
}

pub(crate) struct ComponentMacroProcessor<'data> {
    data: &'data ComponentMacroData,
    delegate: InfallibleComponentMacroProcessor<'data>,

    trait_type: OnceCell<Type>,

    impl_path: OnceCell<TypePath>,

    bindings: OnceCell<HashMap<&'data Ident, &'data Binding>>,
    generics_mapping: OnceCell<HashMap<&'data GenericParam, &'data Type>>,

    generic_args_trait: OnceCell<AngleBracketedGenericArguments>,
    unbound_generics: OnceCell<HashMap<&'data Ident, &'data GenericParam>>,

    generics_unbound: OnceCell<Generics>,
    generic_args_unbound: OnceCell<AngleBracketedGenericArguments>,

    functions: OnceCell<Vec<ImplItem>>,
}

impl<'data> ComponentMacroProcessor<'data> {
    pub(crate) fn new(data: &'data ComponentMacroData) -> Self {
        Self {
            data,
            delegate: InfallibleComponentMacroProcessor::new(data),

            trait_type: OnceCell::new(),

            impl_path: OnceCell::new(),

            bindings: OnceCell::new(),
            generics_mapping: OnceCell::new(),

            generic_args_trait: OnceCell::new(),
            unbound_generics: OnceCell::new(),

            generics_unbound: OnceCell::new(),
            generic_args_unbound: OnceCell::new(),

            functions: OnceCell::new(),
        }
    }
}

impl<'data> ComponentMacroProcessor<'data> {
    fn trait_ident(&self) -> ComponentResult<&Ident> {
        self.delegate.trait_ident().map_err(Into::into)
    }

    fn trait_type(&self) -> ComponentResult<&Type> {
        if let Some(cached) = self.trait_type.get() {
            return Ok(cached);
        }

        let trait_type = {
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

            let type_path = TypePath { qself: None, path };

            Type::Path(type_path)
        };

        Ok(self.trait_type.get_or_init(|| trait_type))
    }

    fn dirk_ident(&self) -> ComponentResult<&Ident> {
        self.delegate.dirk_ident().map_err(Into::into)
    }

    fn impl_path(&self) -> ComponentResult<&TypePath> {
        if let Some(cached) = self.impl_path.get() {
            return Ok(cached);
        }

        let impl_path = {
            let trait_ident = self.trait_ident()?;

            let ident = get_dirk_name(trait_ident, Some("Impl"));

            TypePath {
                qself: None,
                path: Path::from(ident),
            }
        };

        Ok(self.impl_path.get_or_init(|| impl_path))
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
                        .map(|r| r.into_iter())
                })
                .collect::<ComponentResult<Vec<_>>>()?
                .into_iter()
                .flatten()
                .collect::<HashMap<_, _>>()
            };

            // TODO: functionalize
            let mut map = HashMap::new();

            for param in &(input_trait.generics).params {
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
                            map.insert(param, *value);
                        }
                    }
                }
            }

            map
        };

        Ok(self.generics_mapping.get_or_init(|| generics_mapping))
    }

    fn generic_args_trait(&self) -> ComponentResult<&AngleBracketedGenericArguments> {
        if let Some(cached) = self.generic_args_trait.get() {
            return Ok(cached);
        }

        let generic_args_trait = {
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

            let generics_trait = AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Lt::default(),
                args: params_trait,
                gt_token: Gt::default(),
            };

            generics_trait
        };

        Ok(self.generic_args_trait.get_or_init(|| generic_args_trait))
    }

    fn unbound_generics(&self) -> ComponentResult<&HashMap<&Ident, &GenericParam>> {
        if let Some(cached) = self.unbound_generics.get() {
            return Ok(cached);
        }

        let unbound_generics = {
            let input_trait = self.data.input_trait()?;
            let generics_mapping = self.generics_mapping()?;

            let unbound_generics = {
                // TODO: functionalize
                let mut mapping = HashMap::new();

                for param in &input_trait.generics.params {
                    if generics_mapping.get(param).is_none() {
                        if let Ok(type_param) = param.as_type() {
                            let ident = &type_param.ident;
                            mapping.insert(ident, param);
                        }
                    }
                }
                mapping
            };

            unbound_generics
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

    fn generic_args_unbound(&self) -> ComponentResult<&AngleBracketedGenericArguments> {
        if let Some(cached) = self.generic_args_unbound.get() {
            return Ok(cached);
        }

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

            let generics_unbound_actual = AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Lt::default(),
                args: params_unbound_actual,
                gt_token: Gt::default(),
            };

            generics_unbound_actual
        };

        Ok(self
            .generic_args_unbound
            .get_or_init(|| generic_args_unbound))
    }

    fn functions(&self) -> ComponentResult<&Vec<ImplItem>> {
        if let Some(cached) = self.functions.get() {
            return Ok(cached);
        }

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

        Ok(self.functions.get_or_init(|| functions))
    }

    fn builder_kind(&self) -> ComponentResult<ComponentBuilderKind> {
        let builder_data = ComponentBuilderData::new(self.bindings()?, self.trait_ident()?);
        ComponentBuilderKind::evaluate(builder_data, self)
    }

    pub(crate) fn process(self) -> ComponentResult<Vec<Item>> {
        let bindings = self.bindings()?;

        let impl_path = self.impl_path()?;
        let trait_type = self.trait_type()?;

        let functions = self.functions()?;

        let generics_unbound_formal = self.generics_unbound()?;
        let generics_unbound_actual = self.generic_args_unbound()?;

        let (providers_signature, providers_actual, providers_formal, providers_instantiation) =
            get_providers(&bindings)?;

        let items = {
            let input_trait = self.data.input_trait()?.clone();

            let struct_impl: ItemStruct = parse_quote! {
                struct #impl_path #generics_unbound_formal {
                    #providers_signature
                }
            };

            let impl_impl: ItemImpl = parse_quote! {
                impl #generics_unbound_formal #impl_path #generics_unbound_actual {
                    fn new(#providers_formal) -> Self {
                        #(#providers_instantiation)*
                        Self {
                            #providers_actual
                        }
                    }
                }
            };

            let trait_impl = parse_quote! {
                impl #generics_unbound_formal #trait_type for #impl_path #generics_unbound_actual {
                    #(#functions)*
                }
            };

            let builder = self.builder_kind()?;

            let mut items = Vec::new();

            match builder {
                ComponentBuilderKind::StaticBuilder {
                    struct_builder,
                    impl_unset,
                    impl_set,
                    impl_static,
                    impl_component,
                } => {
                    items.push(Item::Struct(struct_builder));
                    items.push(Item::Impl(impl_unset));
                    items.push(Item::Impl(impl_set));
                    items.push(Item::Impl(impl_static));
                    items.push(Item::Impl(impl_component));
                }
                ComponentBuilderKind::NonStaticBuilder {
                    struct_builder,
                    impl_unset,
                    impl_set,
                    partial_impls,
                    impl_component,
                } => {
                    items.push(Item::Struct(struct_builder));
                    items.push(Item::Impl(impl_unset));
                    items.push(Item::Impl(impl_set));
                    items.extend(partial_impls.into_iter().map(|i| Item::Impl(i)));
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
        impl_set: ItemImpl,
        impl_static: ItemImpl,
        impl_component: ItemImpl,
    },
    NonStaticBuilder {
        struct_builder: ItemStruct,
        impl_unset: ItemImpl,
        impl_set: ItemImpl,
        partial_impls: Vec<ItemImpl>,
        impl_component: ItemImpl,
    },
}

impl ComponentBuilderKind {
    fn evaluate(
        builder_data: ComponentBuilderData,
        data: &ComponentMacroProcessor,
    ) -> ComponentResult<Self> {
        let dirk_path = data.dirk_ident()?;
        let impl_path = data.impl_path()?;
        let trait_ident = data.trait_ident()?;
        let trait_type = data.trait_type()?;
        let generics_trait = data.generic_args_trait()?;
        let generics_unbound_formal = data.generics_unbound()?;
        let unbound_generics_mapping = data.unbound_generics()?;

        let builder_path = builder_data.builder_path();
        let builder_generics = builder_data.builder_generics();
        let builder_fields = builder_data.builder_fields();
        let builder_field_values = builder_data.builder_field_values();
        let builder_statements = builder_data.builder_statements();
        let instance_binds = builder_data.instance_binds();

        let struct_builder: ItemStruct = parse_quote! {
            pub(crate) struct #builder_path #builder_generics {
                #builder_fields
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

            let impl_builder_unset = parse_quote! {
                impl #builder_path #unset_generics {
                    fn new () -> Self {
                        #(#builder_statements)*
                        #builder_path { #builder_field_values }
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
                    impl #generics_unbound_formal #builder_path #set_generics {
                        fn build(self) -> impl #trait_ident #generics_trait {
                            #(#unwrap_statements)*
                            #impl_path::new(#providers_actual)
                        }
                    }
                };
                impl_set
            };
            let dirk_impl_component = parse_quote! {
                impl dirk::DirkComponent<#builder_path #unset_generics> for #dirk_path {
                    fn builder() -> #builder_path #unset_generics {
                        #builder_path::new()
                    }
                }
            };

            let mut instance_binds = instance_binds.clone();
            if instance_binds.peek().is_none() {
                let dirk_impl_static_component = parse_quote! {
                    impl #dirk_path {
                        fn create #generics_unbound_formal () -> impl #trait_type {
                            <Self as dirk::DirkComponent<#builder_path>>::builder().build()
                        }
                    }
                };

                Self::StaticBuilder {
                    struct_builder,
                    impl_unset: impl_builder_unset,
                    impl_set: impl_builder_set,
                    impl_static: dirk_impl_static_component,
                    impl_component: dirk_impl_component,
                }
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
                    let mut args_containing_set: Punctuated<GenericArgument, Comma> =
                        Punctuated::new();

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
                            let opaque_ident =
                                Ident::new(&format!("S{index_opaque}"), ident.span());

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

                        parse_quote! {
                            impl #generics_pure #builder_path #generics_containing_unset {
                                fn #ident #generics_partial (self, #ident: #ty) -> #builder_path #generics_containing_set {
                                    #(#statements_opaque)*
                                    #builder_path {
                                        #builder_field_values
                                    }
                                }
                            }

                        }
                    };

                    partial_impls.push(partial_impl);
                }
                Self::NonStaticBuilder {
                    struct_builder,
                    impl_unset: impl_builder_unset,
                    impl_set: impl_builder_set,
                    partial_impls,
                    impl_component: dirk_impl_component,
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
    builder_path: OnceCell<Ident>,
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
            builder_path: OnceCell::new(),
        }
    }
}

impl<'data, 'bindings: 'data> ComponentBuilderData<'data, 'bindings> {
    fn param_ident(ident: &Ident) -> Ident {
        Ident::new(&format!("_{ident}"), ident.span()) // TODO: to upper camel case
    }

    fn instance_binds(&self) -> &Vec<(&'data Ident, &'data ManualBindingKind)> {
        if let Some(cached) = self.instance_binds.get() {
            return cached;
        }

        let instance_binds = {
            let bindings = self.bindings;

            bindings
                .iter()
                .filter_map(|(i, b)| b.kind().as_manual().map(|m| (*i, m)))
                .collect() // TODO: maybe sorted
        };

        self.instance_binds.get_or_init(|| instance_binds)
    }

    fn builder_path(&self) -> &Ident {
        if let Some(cached) = self.builder_path.get() {
            return cached;
        }

        let builder_path = {
            //TODO: maybe convert to TypePath
            let trait_ident = self.trait_ident;
            get_dirk_name(trait_ident, Some("Builder"))
        };

        self.builder_path.get_or_init(|| builder_path)
    }

    fn builder_generics(&self) -> Generics {
        let instance_binds = self.instance_binds().into_iter().peekable();

        let mut generic_params = Punctuated::new();

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
            let mut bounds = Punctuated::new();
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
            let param_ident = Ident::new(&format!("_{ident}"), ident.span()); // TODO: to upper camel case

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

        statements
    }
}
