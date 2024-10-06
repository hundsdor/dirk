use proc_macro::TokenStream;

use proc_macro2::Span;
use quote::quote;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    token::{Bracket, Comma, Paren, Pound},
    Attribute, Ident, Item, ItemImpl, ItemStruct, ItemTrait, Meta, MetaList, Path, PathArguments,
    PathSegment, Type, TypePath,
};

use crate::{
    component::syntax::process_generics, errors::InfallibleResult, expectable::TraitItemExpectable,
};

use self::{
    binding::Binding,
    error::{ComponentResult, ComponentSyntaxError},
    syntax::{
        get_bindings, get_builder, get_dirk_name, get_functions, get_generics_mapping,
        get_providers, process_instance_binds,
    },
};

mod error;
mod syntax;

mod binding;

pub(crate) fn _macro(
    attr: TokenStream,
    item: TokenStream,
) -> InfallibleResult<TokenStream, ComponentSyntaxError> {
    let mut input_trait =
        syn::parse::<ItemTrait>(item).map_err(ComponentSyntaxError::ExpectedTrait)?;

    //#######

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

    let input_attr = Attribute {
        pound_token: Pound::default(),
        style: syn::AttrStyle::Outer,
        bracket_token: Bracket::default(),
        meta,
    };

    input_trait.attrs.push(input_attr);

    //#######

    let trait_ident = &input_trait.ident;
    let dirk_path = get_dirk_name(trait_ident, None);

    //#######
    let dirk_struct = parse_quote! {
        struct #dirk_path {}
    };

    let items = vec![Item::Struct(dirk_struct), Item::Trait(input_trait)];

    let expaned = quote! { #(#items)* };
    Ok(TokenStream::from(expaned))
}

pub(crate) fn _macro_helper(attr: TokenStream, item: TokenStream) -> ComponentResult<TokenStream> {
    let input_trait = syn::parse::<ItemTrait>(item).map_err(ComponentSyntaxError::ExpectedTrait)?;
    let input_attr = syn::parse::<ComponentMacroInput>(attr)
        .map_err(ComponentSyntaxError::FailedToParseInput)?;

    let bindings = get_bindings(&input_attr.bindings);
    let mapping = get_generics_mapping(&input_trait, &bindings)?;

    let (
        generics_trait,
        generics_unbound_formal,
        generics_unbound_actual,
        unbound_generics_mapping,
    ) = process_generics(&mapping, &input_trait.generics);

    //#######

    let trait_ident = &input_trait.ident;
    let trait_type = {
        let ident = trait_ident.clone();

        let mut segments = Punctuated::new();
        let segment = PathSegment {
            ident,
            arguments: syn::PathArguments::AngleBracketed(generics_trait.clone()),
        };
        segments.push(segment);

        let path = Path {
            leading_colon: None,
            segments,
        };

        let type_path = TypePath { qself: None, path };

        Type::Path(type_path)
    };

    let dirk_path = get_dirk_name(trait_ident, None);

    let impl_path = {
        let ident = get_dirk_name(trait_ident, Some("Impl"));

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

        TypePath { qself: None, path }
    };

    let fns = input_trait
        .items
        .iter()
        .map(|i| i.as_fn().map_err(std::convert::Into::into))
        .collect::<ComponentResult<_>>()?;
    let functions = get_functions(fns, &bindings)?;

    let (builder_path, builder_generics, builder_fields, builder_field_values, builder_statements) =
        get_builder(trait_ident, &bindings);

    let (impl_builder_unset, partial_generics, impl_builder_set, dirk_impl_component) =
        process_instance_binds(
            &dirk_path,
            &impl_path,
            trait_ident,
            &trait_type,
            &generics_trait,
            &generics_unbound_formal,
            &unbound_generics_mapping,
            &builder_path,
            builder_field_values,
            builder_statements,
            &bindings,
        );

    let (providers_signature, providers_actual, providers_formal, providers_instantiation) =
        get_providers(&bindings)?;

    //#######

    let struct_builder: ItemStruct = parse_quote! {
        pub(crate) struct #builder_path #builder_generics {
            #builder_fields
        }
    };

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

    let mut items = Vec::new();

    items.push(Item::Struct(struct_builder));
    items.push(Item::Impl(impl_builder_unset));
    items.extend(partial_generics.into_iter().map(Item::Impl));
    items.push(Item::Impl(impl_builder_set));
    items.push(Item::Struct(struct_impl));
    items.push(Item::Impl(impl_impl));
    items.push(Item::Impl(trait_impl));
    items.push(Item::Impl(dirk_impl_component));
    items.push(Item::Trait(input_trait));

    let expaned = quote! { #(#items)* };
    Ok(TokenStream::from(expaned))
}

#[derive(Debug)]
struct ComponentMacroInput {
    _bracket: Bracket,
    bindings: Punctuated<Binding, Comma>,
}

impl Parse for ComponentMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let binds;

        let bracket = bracketed!(binds in input);
        let bindings = binds.parse_terminated(Binding::parse, Comma)?;

        let res = ComponentMacroInput {
            _bracket: bracket,
            bindings,
        };

        Ok(res)
    }
}
