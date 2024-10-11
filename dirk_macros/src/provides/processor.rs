use std::{cell::OnceCell, collections::HashMap};

use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    spanned::Spanned,
    token::Dot,
    token::{Colon, Comma, Dyn, Paren},
    Expr, ExprMethodCall, ExprPath, Field, FieldValue, FnArg, Generics, Ident, ImplItem,
    ImplItemFn, ItemImpl, ItemStatic, Pat, PatIdent, PatType, Path, PathArguments, TraitBound,
    Type, TypeParamBound, TypePath, TypeTraitObject,
};

use crate::{
    errors::{InfallibleError, InfallibleResult},
    expectable::{
        FnArgExpectable, ImplItemExpectable, PatExpectable, ReturnTypeExpectable, TypeExpectable,
    },
    syntax::wrap_type,
    util::type_rc,
};

use syn::{
    parse_quote,
    punctuated::Punctuated,
    token::{Gt, Lt},
    AngleBracketedGenericArguments, GenericArgument, Item, ItemStruct,
};

use crate::util::type_provider;

use super::syntax::{get_call_path, get_constructor_call};
use super::{
    error::{ProvidesResult, ProvidesSyntaxError},
    ProvidesMacroInput,
};

use super::{
    error::ProvidesLogicError,
    syntax::{get_instance_name, map_generic_params},
};

pub(crate) struct ProvidesMacroData {
    attr: TokenStream,
    item: TokenStream,

    input_macro: OnceCell<ProvidesMacroInput>,
    input_impl: OnceCell<ItemImpl>,
}

impl ProvidesMacroData {
    pub(crate) fn new(attr: TokenStream, item: TokenStream) -> Self {
        Self {
            attr,
            item,
            input_macro: OnceCell::new(),
            input_impl: OnceCell::new(),
        }
    }
}

impl ProvidesMacroData {
    fn input_macro(&self) -> InfallibleResult<&ProvidesMacroInput, ProvidesSyntaxError> {
        if let Some(cached) = self.input_macro.get() {
            return Ok(cached);
        }

        let input_macro = {
            let attr = self.attr.clone();

            syn::parse::<ProvidesMacroInput>(attr)
                .map_err(ProvidesSyntaxError::FailedToParseInput)?
        };

        Ok(self.input_macro.get_or_init(|| input_macro))
    }

    fn input_impl(&self) -> InfallibleResult<&ItemImpl, ProvidesSyntaxError> {
        if let Some(cached) = self.input_impl.get() {
            return Ok(cached);
        }

        let input_impl = {
            let item = self.item.clone();

            syn::parse::<ItemImpl>(item).map_err(ProvidesSyntaxError::ExpectedImpl)?
        };

        Ok(self.input_impl.get_or_init(|| input_impl))
    }
}

pub(crate) struct ProvidesMacroProcessor<'data> {
    data: &'data ProvidesMacroData,

    function: OnceCell<&'data ImplItemFn>,
    wrapped_types: OnceCell<HashMap<FnArg, (Ident, Type, PatType)>>,

    field_args: OnceCell<Punctuated<FnArg, Comma>>,

    injectable_ty: OnceCell<Type>,
    injectable_path: OnceCell<TypePath>,

    injected_ty: OnceCell<Type>,
    provider_ty: OnceCell<Type>,

    generics: OnceCell<Generics>,

    factory_ty: OnceCell<Type>,
    factory_path: OnceCell<TypePath>,
}

impl<'data> ProvidesMacroProcessor<'data> {
    pub(crate) fn new(data: &'data ProvidesMacroData) -> Self {
        Self {
            data,

            function: OnceCell::new(),
            wrapped_types: OnceCell::new(),

            field_args: OnceCell::new(),

            injectable_ty: OnceCell::new(),
            injectable_path: OnceCell::new(),

            injected_ty: OnceCell::new(),
            provider_ty: OnceCell::new(),

            generics: OnceCell::new(),

            factory_ty: OnceCell::new(),
            factory_path: OnceCell::new(),
        }
    }
}

impl<'data> ProvidesMacroProcessor<'data> {
    fn function(&self) -> ProvidesResult<&ImplItemFn> {
        if let Some(cached) = self.function.get() {
            return Ok(cached);
        }

        let function = {
            let input_impl = self.data.input_impl()?;

            let items = &input_impl.items;

            let function = items
                .iter()
                .filter(|f| matches!(f, ImplItem::Fn(_)))
                .exactly_one()
                .map_err(|e| {
                    ProvidesLogicError::InvalidFunctionCount(
                        input_impl.clone(),
                        e.try_len()
                            .expect("Number of functions in an impl block needs to be finite"),
                    )
                })?;

            function.as_fn()?
        };

        Ok(self.function.get_or_init(|| function))
    }

    fn function_ident(&self) -> ProvidesResult<&Ident> {
        let function_ident = {
            let function = self.function()?;
            &function.sig.ident
        };
        Ok(function_ident)
    }

    fn field_args(&self) -> ProvidesResult<&Punctuated<FnArg, Comma>> {
        if let Some(cached) = self.field_args.get() {
            return Ok(cached);
        }

        let field_args = {
            let function = self.function()?;

            function.sig.inputs.clone()
        };

        Ok(self.field_args.get_or_init(|| field_args))
    }

    fn field_exprs(&self) -> ProvidesResult<Punctuated<Expr, Comma>> {
        let field_exprs = {
            let formal_fields = self.field_args()?;

            formal_fields
                .iter()
                .map(|f| {
                    let pat_type = f.as_typed()?;
                    let pat_ident = pat_type.pat.as_ident()?;

                    let ident = pat_ident.ident.clone();

                    let expr = Expr::Path(ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: Path::from(ident),
                    });

                    Ok(expr)
                })
                .collect::<ProvidesResult<Punctuated<Expr, Comma>>>()?
        };
        Ok(field_exprs)
    }

    fn injectable_ty(&self) -> ProvidesResult<&Type> {
        if let Some(cached) = self.injectable_ty.get() {
            return Ok(cached);
        }

        let injectable_ty = {
            let input_macro = self.data.input_macro()?;
            let input_impl = self.data.input_impl()?;
            let function = self.function()?;

            let fun_ty = function.sig.output.as_type()?.1.clone();

            let type_path = fun_ty.as_path()?;

            if !type_path.path.is_ident("Self") {
                return Err(ProvidesLogicError::InvalidReturnType(fun_ty))?;
            }

            if let ProvidesMacroInput::Singleton(_) = input_macro {
                let args = &function.sig.inputs;
                if !args.is_empty() {
                    return Err(ProvidesLogicError::SingletonWithArgs(args.clone()))?;
                }
            }

            (*input_impl.self_ty).clone()
        };

        Ok(self.injectable_ty.get_or_init(|| injectable_ty))
    }

    fn injectable_path(&self) -> ProvidesResult<&TypePath> {
        if let Some(cached) = self.injectable_path.get() {
            return Ok(cached);
        }

        let injectable_path = {
            let input_impl = self.data.input_impl()?;

            let mut type_path = input_impl.self_ty.as_path()?.clone();
            let span = type_path.span();
            let last = type_path
                .path
                .segments
                .last_mut()
                .ok_or_else(|| InfallibleError::EmptyPath(span))?;
            last.arguments = PathArguments::None;

            type_path
        };

        Ok(self.injectable_path.get_or_init(|| injectable_path))
    }

    fn injected_ty(&self) -> ProvidesResult<&Type> {
        if let Some(cached) = self.injected_ty.get() {
            return Ok(cached);
        }

        let injected_ty = {
            let input_macro = self.data.input_macro()?;
            let injectable_ty = self.injectable_ty()?;

            input_macro.wrap_type(injectable_ty.clone())
        };

        Ok(self.injected_ty.get_or_init(|| injected_ty))
    }

    fn provider_ty(&self) -> ProvidesResult<&Type> {
        if let Some(cached) = self.provider_ty.get() {
            return Ok(cached);
        }

        let provider_ty = {
            let injected_ty = self.injected_ty()?;

            let provider_generics = {
                let mut args = Punctuated::new();
                let arg = GenericArgument::Type(injected_ty.clone());
                args.push(arg);

                AngleBracketedGenericArguments {
                    colon2_token: None,
                    lt_token: Lt::default(),
                    args,
                    gt_token: Gt::default(),
                }
            };
            type_provider(PathArguments::AngleBracketed(provider_generics))
        };

        Ok(self.provider_ty.get_or_init(|| provider_ty))
    }

    fn generics(&self) -> ProvidesResult<&Generics> {
        if let Some(cached) = self.generics.get() {
            return Ok(cached);
        }

        let generics = {
            let input_impl = self.data.input_impl()?;
            input_impl.generics.clone()
        };

        Ok(self.generics.get_or_init(|| generics))
    }

    fn factory_ty(&self) -> ProvidesResult<&Type> {
        if let Some(cached) = self.factory_ty.get() {
            return Ok(cached);
        }

        let factory_ty = {
            let input_macro = self.data.input_macro()?;
            let injectable_ty = self.injectable_ty()?;
            let generics = self.generics()?;

            let args = map_generic_params(generics.params.clone());
            let angle_bracketed = AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Lt::default(),
                args,
                gt_token: Gt::default(),
            };
            let generic_args = PathArguments::AngleBracketed(angle_bracketed);

            let mut factory_ty = injectable_ty.clone();
            let span = factory_ty.span();
            let path = factory_ty.as_path_mut()?;

            let last = path
                .path
                .segments
                .last_mut()
                .ok_or_else(|| InfallibleError::EmptyPath(span))?;

            last.ident = Ident::new(
                &format!("{}{}", input_macro.factory_prefix(), last.ident),
                last.ident.span(),
            );
            last.arguments = generic_args.clone();

            factory_ty
        };

        Ok(self.factory_ty.get_or_init(|| factory_ty))
    }

    fn factory_path(&self) -> ProvidesResult<&TypePath> {
        if let Some(cached) = self.factory_path.get() {
            return Ok(cached);
        }

        let factory_path = {
            let factory_ty = self.factory_ty()?;

            let mut factory_path = factory_ty.as_path()?.clone();
            let span = factory_ty.span();
            let last = factory_path
                .path
                .segments
                .last_mut()
                .ok_or_else(|| InfallibleError::EmptyPath(span))?;
            last.arguments = PathArguments::None;

            factory_path
        };

        Ok(self.factory_path.get_or_init(|| factory_path))
    }

    fn wrapped_types(&self) -> ProvidesResult<&HashMap<FnArg, (Ident, Type, PatType)>> {
        if let Some(cached) = self.wrapped_types.get() {
            return Ok(cached);
        }

        let wrapped_types = {
            let formal_fields = self.field_args()?;

            let mut wrapped_types = HashMap::new();

            for f in formal_fields {
                let pat_type = f.as_typed()?.clone();

                let ident = {
                    let pat = &pat_type.pat;
                    let pat_ident = pat.as_ident()?;

                    Ident::new(
                        &format!("{}_provider", pat_ident.ident),
                        pat_ident.ident.span(),
                    )
                };

                let ty = {
                    let ty = pat_type.ty.as_ref().clone();

                    let provider_type = wrap_type(ty, type_provider);

                    let type_path = provider_type.as_path()?;

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

                    wrap_type(dyn_type, type_rc)
                };

                let pat_type = {
                    let pat_ident = PatIdent {
                        attrs: Vec::new(),
                        by_ref: None,
                        mutability: None,
                        ident: ident.clone(),
                        subpat: None,
                    };
                    let pat = Pat::Ident(pat_ident);
                    PatType {
                        attrs: Vec::new(),
                        pat: Box::new(pat),
                        colon_token: Colon::default(),
                        ty: Box::new(ty.clone()),
                    }
                };

                wrapped_types.insert(f.clone(), (ident, ty, pat_type));
            }

            wrapped_types
        };

        Ok(self.wrapped_types.get_or_init(|| wrapped_types))
    }

    fn providers_args(&self) -> ProvidesResult<Punctuated<FnArg, Comma>> {
        let formal_fields = self.field_args()?;
        let wrapped_types = self.wrapped_types()?;

        let mut fn_args = Punctuated::new();

        for f in formal_fields {
            let (_ident, _ty, pat_type) = wrapped_types.get(f).expect("Prepopulated");

            let fn_arg: FnArg = FnArg::Typed(pat_type.clone());
            fn_args.push(fn_arg);
        }

        Ok(fn_args)
    }

    fn providers_fields(&self) -> ProvidesResult<Punctuated<Field, Comma>> {
        let formal_fields = self.field_args()?;
        let wrapped_types = self.wrapped_types()?;

        let mut fields = Punctuated::new();

        for f in formal_fields {
            let (ident, ty, _pat_type) = wrapped_types.get(f).expect("Prepopulated");

            let field: Field = Field {
                attrs: Vec::new(),
                vis: syn::Visibility::Inherited,
                mutability: syn::FieldMutability::None,
                ident: Some(ident.clone()),
                colon_token: Some(Colon::default()),
                ty: ty.clone(),
            };

            fields.push(field);
        }

        Ok(fields)
    }

    fn providers_field_values(&self) -> ProvidesResult<Punctuated<FieldValue, Comma>> {
        let formal_fields = self.field_args()?;
        let wrapped_types = self.wrapped_types()?;

        let mut field_values = Punctuated::new();

        for f in formal_fields {
            let (ident, _ty, _pat_type) = wrapped_types.get(f).expect("Prepopulated");

            let field_value: FieldValue = {
                let member = syn::Member::Named(ident.clone());
                let expr = Expr::Path(ExprPath {
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

        Ok(field_values)
    }

    fn providers_getter(&self) -> ProvidesResult<Punctuated<Expr, Comma>> {
        let formal_fields = self.field_args()?;
        let input_macro = self.data.input_macro()?;

        let mut exprs = Punctuated::new();

        for f in formal_fields {
            let mut pat_type = f.as_typed()?.clone();

            let ident = {
                let pat = &mut pat_type.pat;
                let pat_ident = pat.as_ident_mut()?;

                let ident = Ident::new(
                    &format!("{}_provider", pat_ident.ident),
                    pat_ident.ident.span(),
                );

                pat_ident.ident = ident.clone();

                ident
            };

            let expr = {
                let receiver = input_macro.receiver(ident);

                let get_ident = Ident::new("get", Span::call_site());

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
            };

            exprs.push(expr);
        }

        Ok(exprs)
    }

    fn constructor_call(&self) -> ProvidesResult<Expr> {
        let input_macro = self.data.input_macro()?;
        let injectable_path = self.injectable_path()?;
        let function_ident = self.function_ident()?;
        let fields_exprs = self.field_exprs()?;

        let constructor_call = {
            let injected = get_call_path(injectable_path, function_ident.clone());
            let constructor_call = get_constructor_call(injected, fields_exprs);
            input_macro.wrap_call(constructor_call)
        };
        Ok(constructor_call)
    }

    pub(crate) fn process(self) -> ProvidesResult<Vec<Item>> {
        let input_macro = self.data.input_macro()?;
        let input_impl = self.data.input_impl()?.clone();

        let injected_ty = self.injected_ty()?;
        let factory_ty = self.factory_ty()?;
        let factory_path = self.factory_path()?;
        let constructor_call = self.constructor_call()?;
        let impl_generics = self.generics()?;
        let provider_ty = self.provider_ty()?;
        let formal_fields = self.field_args()?;

        let providers_args = self.providers_args()?;
        let providers_fields = self.providers_fields()?;
        let providers_field_values = self.providers_field_values()?;
        let providers_getter = self.providers_getter()?;

        let items = {
            match input_macro {
                ProvidesMacroInput::Static(_) => {
                    let struct_factory: ItemStruct = parse_quote! {
                        pub(crate) struct #factory_path #impl_generics {
                            #providers_fields
                        }
                    };

                    let impl_provider_for_factory: ItemImpl = parse_quote! {

                       impl #impl_generics #provider_ty for #factory_ty {
                            fn get(&self) -> #injected_ty {
                                Self::new_instance(#providers_getter)
                            }
                       }
                    };

                    let impl_factory: ItemImpl = parse_quote! {

                        impl #impl_generics #factory_ty {
                            fn new(#providers_args) -> Self {
                                Self {
                                    #providers_field_values
                                }
                            }

                            pub fn create(#providers_args) -> Self {
                                Self::new(#providers_field_values)
                            }

                            fn new_instance(#formal_fields) -> #injected_ty {
                                #constructor_call
                            }
                        }
                    };

                    vec![
                        Item::Struct(struct_factory),
                        Item::Impl(impl_provider_for_factory),
                        Item::Impl(impl_factory),
                        Item::Impl(input_impl),
                    ]
                }
                ProvidesMacroInput::Scoped(_) => {
                    let struct_factory: ItemStruct = parse_quote! {
                        pub(crate) struct #factory_path #impl_generics {
                            singleton: #injected_ty
                        }
                    };

                    let impl_provider_for_factory: ItemImpl = parse_quote! {

                       impl #impl_generics #provider_ty for #factory_ty {
                            fn get(&self) -> #injected_ty {
                                self.singleton.clone()
                            }
                       }
                    };

                    let impl_factory: ItemImpl = parse_quote! {

                        impl #impl_generics #factory_ty {
                            fn new(#providers_args) -> Self {
                                Self {
                                    singleton: Self::new_instance(#providers_getter),
                                }
                            }

                            pub fn create(#providers_args) -> Self {
                                Self::new(#providers_field_values)
                            }

                            fn new_instance(#formal_fields) -> #injected_ty {
                                #constructor_call
                            }
                        }
                    };

                    let items = vec![
                        Item::Struct(struct_factory),
                        Item::Impl(impl_provider_for_factory),
                        Item::Impl(impl_factory),
                        Item::Impl(input_impl),
                    ];

                    items
                }
                ProvidesMacroInput::Singleton(_) => {
                    let factory_instance_name = get_instance_name(factory_path);

                    let factory_call = get_call_path(
                        factory_path,
                        Ident::new("new", factory_instance_name.span()),
                    );
                    let factory_constructor_call =
                        get_constructor_call(factory_call, Punctuated::new());

                    let struct_factory: ItemStruct = parse_quote! {
                        #[derive(Clone)]
                        pub(crate) struct #factory_path #impl_generics {
                            singleton: #injected_ty
                        }
                    };

                    let impl_provider_for_factory: ItemImpl = parse_quote! {

                       impl #impl_generics #provider_ty for #factory_ty {
                            fn get(&self) -> #injected_ty {
                                self.singleton.clone()
                            }
                       }
                    };

                    let impl_factory: ItemImpl = parse_quote! {

                        impl #impl_generics #factory_ty {
                            fn new(#providers_args) -> Self {
                                Self {
                                    singleton: Self::new_instance(#providers_getter),
                                }
                            }

                            pub fn create(#providers_args) -> Self {
                                #factory_instance_name.clone()
                            }

                            fn new_instance(#formal_fields) -> #injected_ty {
                                #constructor_call
                            }
                        }
                    };

                    let static_factory_instance: ItemStatic = parse_quote! {
                        static #factory_instance_name: dirk::provides::FactoryInstance<#factory_path> =
                            dirk::provides::FactoryInstance::new(|| #factory_constructor_call);
                    };

                    vec![
                        Item::Struct(struct_factory),
                        Item::Impl(impl_provider_for_factory),
                        Item::Impl(impl_factory),
                        Item::Static(static_factory_instance),
                        Item::Impl(input_impl),
                    ]
                }
            }
        };

        Ok(items)
    }
}
