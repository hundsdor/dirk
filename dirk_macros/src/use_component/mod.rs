use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse::Parse, parse_quote, Attribute, Item, ItemUse, UseTree};

use self::error::{UseComponentLogicError, UseComponentResult, UseComponentSyntaxError};

mod error;

pub(crate) fn _macro(attr: TokenStream, item: TokenStream) -> UseComponentResult<TokenStream> {
    let input = syn::parse::<UseComponentMacroInput>(attr)
        .map_err(UseComponentSyntaxError::FailedToParseInput)?;
    let input_use = syn::parse::<ItemUse>(item).map_err(UseComponentSyntaxError::ExpectedUse)?;

    let allow_attr: Attribute = parse_quote! {
        #[allow(unused_imports)]
    };
    // input_use.attrs.push(allow_attr.clone());

    let mut use_dirk = input_use.clone();
    use_dirk.attrs = Vec::new();
    input.convert_use_tree(&mut use_dirk.tree, "Dirk", "")?;

    let mut use_builder = input_use.clone();
    use_builder.attrs = Vec::new();
    input.convert_use_tree(&mut use_builder.tree, "Dirk", "Builder")?;
    use_builder.attrs.push(allow_attr.clone());

    let use_component = parse_quote! {
        #allow_attr
        use dirk::component::DirkComponent;
    };
    let use_static_component = parse_quote! {
        #allow_attr
        use dirk::component::DirkStaticComponent;
    };

    let items = vec![
        Item::Use(input_use),
        Item::Use(use_dirk),
        Item::Use(use_builder),
        Item::Use(use_component),
        Item::Use(use_static_component),
    ];

    let expaned = quote! { #(#items)* };
    Ok(TokenStream::from(expaned))
}

#[allow(dead_code)]
#[derive(Debug, Default)]
struct UseComponentMacroInput {}

impl Parse for UseComponentMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let lookahead = input.lookahead1();
        Err(lookahead.error())
    }
}

impl UseComponentMacroInput {
    #[allow(clippy::only_used_in_recursion)]
    fn convert_use_tree(
        &self,
        tree: &mut UseTree,
        prefix: &str,
        postfix: &str,
    ) -> UseComponentResult<()> {
        match tree {
            UseTree::Path(path) => self.convert_use_tree(&mut path.tree, prefix, postfix),
            UseTree::Group(g) => g
                .items
                .iter_mut()
                .try_for_each(|i| self.convert_use_tree(i, prefix, postfix)),
            UseTree::Name(name) => {
                let ident = &name.ident;
                name.ident = Ident::new(&format!("{prefix}{ident}{postfix}"), ident.span());
                Ok(())
            }
            UseTree::Rename(use_rename) => {
                let ident = &use_rename.ident;
                use_rename.ident = Ident::new(&format!("{prefix}{ident}{postfix}"), ident.span());

                let rename = &use_rename.rename;
                use_rename.rename =
                    Ident::new(&format!("{prefix}{rename}{postfix}"), rename.span());

                Ok(())
            }
            UseTree::Glob(g) => Err(UseComponentLogicError::FoundGlob(g.clone()))?,
        }
    }
}
