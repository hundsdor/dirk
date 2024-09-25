use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{Item, ItemUse, UseTree};

use crate::{Result, UseInjectableLogicError};

pub(crate) fn _macro(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let input_use = syn::parse::<ItemUse>(item)?;

    let mut use_factories = input_use.clone();

    use_factories.attrs = Vec::new();
    convert_use_tree(&mut use_factories.tree)?;

    let items = vec![Item::Use(input_use), Item::Use(use_factories)];

    let expaned = quote! { #(#items)* };
    Ok(TokenStream::from(expaned))
}

fn convert_use_tree(tree: &mut UseTree) -> Result<()> {
    match tree {
        UseTree::Path(path) => convert_use_tree(&mut path.tree),
        UseTree::Group(g) => g.items.iter_mut().map(|i| convert_use_tree(i)).collect(),
        UseTree::Name(name) => {
            let ident = &name.ident;
            name.ident = Ident::new(&format!("Factory{ident}"), ident.span());
            Ok(())
        }
        UseTree::Rename(use_rename) => {
            let ident = &use_rename.ident;
            use_rename.ident = Ident::new(&format!("Factory{ident}"), ident.span());

            let rename = &use_rename.rename;
            use_rename.rename = Ident::new(&format!("Factory{rename}"), rename.span());

            Ok(())
        }
        UseTree::Glob(g) => Err(UseInjectableLogicError::FoundGlob(g.clone()).into()),
    }
}
