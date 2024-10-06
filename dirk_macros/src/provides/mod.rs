use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    parse::Parse, token::Comma, token::PathSep, Expr, Field, FieldValue, FnArg, Generics, Ident,
    ItemImpl, ItemStatic, PathArguments, Type, TypePath,
};

use crate::{
    syntax::wrap_type,
    util::{segments, type_arc, type_rc, type_refcell, type_rwlock},
    FACTORY_PREFIX_SCOPED, FACTORY_PREFIX_SINGLETON, FACTORY_PREFIX_STATIC,
};

use quote::quote;
use syn::{
    parse_quote,
    punctuated::Punctuated,
    token::{Gt, Lt},
    AngleBracketedGenericArguments, GenericArgument, Item, ItemStruct, PathSegment,
};

use crate::util::type_provider;

use error::{ProvidesResult, ProvidesSyntaxError};
use syntax::{
    get_call_path, get_constructor_call, get_factory_ty, get_fields, get_injectable, get_providers,
};

use self::syntax::{get_instance_name, wrap_call};

mod error;
mod syntax;

pub(crate) fn _macro(attr: TokenStream, item: TokenStream) -> ProvidesResult<TokenStream> {
    let input =
        syn::parse::<ProvidesMacroInput>(attr).map_err(ProvidesSyntaxError::FailedToParseInput)?;
    let input_impl = syn::parse::<ItemImpl>(item).map_err(ProvidesSyntaxError::ExpectedImpl)?;

    let (ident, formal_fields, actual_fields) = get_fields(&input_impl)?;
    let (injectable_ty, injectable_path) = get_injectable(&input_impl)?;
    let impl_generics = input_impl.generics.clone();
    let (factory_ty, factory_path) = get_factory_ty(&input, &injectable_ty, &impl_generics.params)?;
    let providers = get_providers(&formal_fields, input.add_self())?;

    //#######
    // Wrapping type

    let injectable_ty = input.wrap_type(injectable_ty);

    //
    //#######

    let provider_ty = {
        let provider_generics = {
            let mut args = Punctuated::new();
            let arg = GenericArgument::Type(injectable_ty.clone());
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

    let injected = get_call_path(&injectable_path, ident);
    let constructor_call = get_constructor_call(injected, actual_fields);

    //#######
    // Wrapping constrcutor by Rc::new(RefCell::new(...))

    let constructor_call = input.wrap_call(constructor_call);

    //
    //#######

    let items = input.generate_items(
        input_impl,
        injectable_ty,
        factory_ty,
        factory_path,
        constructor_call,
        impl_generics,
        provider_ty,
        providers,
        formal_fields,
    );

    let expaned = quote! { #(#items)* };
    Ok(TokenStream::from(expaned))
}

mod kw {
    syn::custom_keyword!(singleton_inject);
    syn::custom_keyword!(scoped_inject);
    syn::custom_keyword!(static_inject);
}

#[derive(Debug)]
pub enum ProvidesMacroInput {
    Scoped(kw::scoped_inject),
    Singleton(kw::singleton_inject),
    Static(kw::static_inject),
}

impl Default for ProvidesMacroInput {
    fn default() -> Self {
        Self::Static(kw::static_inject {
            span: Span::call_site(),
        })
    }
}

impl Parse for ProvidesMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let lookahead = input.lookahead1();
        let res = if lookahead.peek(kw::singleton_inject) {
            let kw = kw::singleton_inject::parse(input)?;
            Self::Singleton(kw)
        } else if lookahead.peek(kw::scoped_inject) {
            let kw = kw::scoped_inject::parse(input)?;
            Self::Scoped(kw)
        } else if lookahead.peek(kw::static_inject) {
            let kw = kw::static_inject::parse(input)?;
            Self::Static(kw)
        } else {
            return Err(lookahead.error());
        };

        Ok(res)
    }
}

impl ProvidesMacroInput {
    fn add_self(&self) -> bool {
        match self {
            ProvidesMacroInput::Static(_) => true,
            ProvidesMacroInput::Scoped(_) => false,
            ProvidesMacroInput::Singleton(_) => false,
        }
    }

    fn wrap_type(&self, injectable_ty: Type) -> Type {
        match self {
            ProvidesMacroInput::Static(_) => injectable_ty,
            ProvidesMacroInput::Scoped(_) => {
                let injectable_ty = wrap_type(injectable_ty, type_refcell);
                wrap_type(injectable_ty, type_rc)
            }
            ProvidesMacroInput::Singleton(_) => {
                let injectable_ty = wrap_type(injectable_ty, type_rwlock);
                wrap_type(injectable_ty, type_arc)
            }
        }
    }

    fn wrap_call(&self, constructor_call: Expr) -> Expr {
        match self {
            ProvidesMacroInput::Static(_) => constructor_call,
            ProvidesMacroInput::Scoped(_) => {
                let constructor_call =
                    wrap_call(constructor_call, segments!("std", "cell", "RefCell", "new"));
                wrap_call(constructor_call, segments!("std", "rc", "Rc", "new"))
            }
            ProvidesMacroInput::Singleton(_) => {
                let constructor_call =
                    wrap_call(constructor_call, segments!("std", "sync", "RwLock", "new"));
                wrap_call(constructor_call, segments!("std", "sync", "Arc", "new"))
            }
        }
    }

    fn generate_items(
        &self,
        input_impl: ItemImpl,
        injectable_ty: Type,
        factory_ty: Type,
        factory_path: TypePath,
        constructor_call: Expr,
        impl_generics: Generics,
        provider_ty: Type,
        providers: (
            Punctuated<FnArg, Comma>,
            Punctuated<Field, Comma>,
            Punctuated<FieldValue, Comma>,
            Punctuated<Expr, Comma>,
        ),
        formal_fields: Punctuated<FnArg, Comma>,
    ) -> Vec<Item> {
        let (fields_providers, formal_providers, actual_providers, providers_getter) = providers;

        match self {
            ProvidesMacroInput::Static(_) => {
                let struct_factory: ItemStruct = parse_quote! {
                    pub(crate) struct #factory_path #impl_generics {
                        #fields_providers
                    }
                };

                let impl_provider_for_factory: ItemImpl = parse_quote! {

                   impl #impl_generics #provider_ty for #factory_ty {
                        fn get(&self) -> #injectable_ty {
                            Self::new_instance(#providers_getter)
                        }
                   }
                };

                let impl_factory: ItemImpl = parse_quote! {

                    impl #impl_generics #factory_ty {
                        fn new(#formal_providers) -> Self {
                            Self {
                                #actual_providers
                            }
                        }

                        pub fn create(#formal_providers) -> Self {
                            Self::new(#actual_providers)
                        }

                        fn new_instance(#formal_fields) -> #injectable_ty {
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
                        singleton: #injectable_ty
                    }
                };

                let impl_provider_for_factory: ItemImpl = parse_quote! {

                   impl #impl_generics #provider_ty for #factory_ty {
                        fn get(&self) -> #injectable_ty {
                            self.singleton.clone()
                        }
                   }
                };

                let impl_factory: ItemImpl = parse_quote! {

                    impl #impl_generics #factory_ty {
                        fn new(#formal_providers) -> Self {
                            Self {
                                singleton: Self::new_instance(#providers_getter),
                            }
                        }

                        pub fn create(#formal_providers) -> Self {
                            Self::new(#actual_providers)
                        }

                        fn new_instance(#formal_fields) -> #injectable_ty {
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
                let factory_instance_name = get_instance_name(&factory_path);

                let factory_call = get_call_path(
                    &factory_path,
                    Ident::new("new", factory_instance_name.span()),
                );
                let factory_constructor_call =
                    get_constructor_call(factory_call, Punctuated::new());

                let struct_factory: ItemStruct = parse_quote! {
                    #[derive(Clone)]
                    pub(crate) struct #factory_path #impl_generics {
                        singleton: #injectable_ty
                    }
                };

                let impl_provider_for_factory: ItemImpl = parse_quote! {

                   impl #impl_generics #provider_ty for #factory_ty {
                        fn get(&self) -> #injectable_ty {
                            self.singleton.clone()
                        }
                   }
                };

                let impl_factory: ItemImpl = parse_quote! {

                    impl #impl_generics #factory_ty {
                        fn new(#formal_providers) -> Self {
                            Self {
                                singleton: Self::new_instance(#providers_getter),
                            }
                        }

                        pub fn create(#formal_providers) -> Self {
                            #factory_instance_name.clone()
                        }

                        fn new_instance(#formal_fields) -> #injectable_ty {
                            #constructor_call
                        }
                    }
                };

                let static_factory_instance: ItemStatic = parse_quote! {
                    static #factory_instance_name: dirk::FactoryInstance<#factory_path> =
                        dirk::FactoryInstance::new(|| #factory_constructor_call);
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
    }

    pub(crate) fn factory_prefix(&self) -> &'static str {
        match self {
            ProvidesMacroInput::Static(_) => FACTORY_PREFIX_STATIC,
            ProvidesMacroInput::Scoped(_) => FACTORY_PREFIX_SCOPED,
            ProvidesMacroInput::Singleton(_) => FACTORY_PREFIX_SINGLETON,
        }
    }
}
