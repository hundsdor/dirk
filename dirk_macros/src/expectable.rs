#![allow(dead_code)]

use proc_macro2::TokenStream;
use proc_macro_error::abort;
use syn::{
    token::RArrow, AngleBracketedGenericArguments, AssocConst, AssocType, ConstParam, Constraint,
    Expr, FnArg, GenericArgument, GenericParam, ImplItem, ImplItemConst, ImplItemFn, ImplItemMacro,
    ImplItemType, Lifetime, LifetimeParam, ParenthesizedGenericArguments, Pat, PatConst, PatIdent,
    PatLit, PatMacro, PatOr, PatParen, PatPath, PatRange, PatReference, PatRest, PatSlice,
    PatStruct, PatTuple, PatTupleStruct, PatType, PatWild, PathArguments, Receiver, ReturnType,
    TraitItem, TraitItemConst, TraitItemFn, TraitItemMacro, TraitItemType, Type, TypeArray,
    TypeBareFn, TypeGroup, TypeImplTrait, TypeInfer, TypeMacro, TypeNever, TypeParam, TypeParen,
    TypePath, TypePtr, TypeReference, TypeSlice, TypeTraitObject, TypeTuple,
};

use crate::errors::ExpectableError;

#[derive(Debug)]
pub(crate) struct UnexpectedImplItemKind(ImplItem);
impl ExpectableError for UnexpectedImplItemKind {
    fn abort(&self) -> ! {
        abort!(
            self.0,
            format!("Found unexpected kind of ImplItem: {:?}", self.0)
        )
    }
}

pub(crate) trait ImplItemExpectable {
    fn as_const(&self) -> Result<&ImplItemConst, UnexpectedImplItemKind>;
    fn as_fn(&self) -> Result<&ImplItemFn, UnexpectedImplItemKind>;
    fn as_type(&self) -> Result<&ImplItemType, UnexpectedImplItemKind>;
    fn as_macro(&self) -> Result<&ImplItemMacro, UnexpectedImplItemKind>;
    fn as_verbatim(&self) -> Result<&TokenStream, UnexpectedImplItemKind>;

    fn as_const_mut(&mut self) -> Result<&mut ImplItemConst, UnexpectedImplItemKind>;
    fn as_fn_mut(&mut self) -> Result<&mut ImplItemFn, UnexpectedImplItemKind>;
    fn as_type_mut(&mut self) -> Result<&mut ImplItemType, UnexpectedImplItemKind>;
    fn as_macro_mut(&mut self) -> Result<&mut ImplItemMacro, UnexpectedImplItemKind>;
    fn as_verbatim_mut(&mut self) -> Result<&mut TokenStream, UnexpectedImplItemKind>;
}

impl ImplItemExpectable for ImplItem {
    fn as_const(&self) -> Result<&ImplItemConst, UnexpectedImplItemKind> {
        if let ImplItem::Const(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedImplItemKind(self.clone()))
    }

    fn as_fn(&self) -> Result<&ImplItemFn, UnexpectedImplItemKind> {
        if let ImplItem::Fn(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedImplItemKind(self.clone()))
    }

    fn as_type(&self) -> Result<&ImplItemType, UnexpectedImplItemKind> {
        if let ImplItem::Type(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedImplItemKind(self.clone()))
    }

    fn as_macro(&self) -> Result<&ImplItemMacro, UnexpectedImplItemKind> {
        if let ImplItem::Macro(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedImplItemKind(self.clone()))
    }

    fn as_verbatim(&self) -> Result<&TokenStream, UnexpectedImplItemKind> {
        if let ImplItem::Verbatim(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedImplItemKind(self.clone()))
    }

    fn as_const_mut(&mut self) -> Result<&mut ImplItemConst, UnexpectedImplItemKind> {
        if let ImplItem::Const(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedImplItemKind(self.clone()))
    }

    fn as_fn_mut(&mut self) -> Result<&mut ImplItemFn, UnexpectedImplItemKind> {
        if let ImplItem::Fn(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedImplItemKind(self.clone()))
    }

    fn as_type_mut(&mut self) -> Result<&mut ImplItemType, UnexpectedImplItemKind> {
        if let ImplItem::Type(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedImplItemKind(self.clone()))
    }

    fn as_macro_mut(&mut self) -> Result<&mut ImplItemMacro, UnexpectedImplItemKind> {
        if let ImplItem::Macro(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedImplItemKind(self.clone()))
    }

    fn as_verbatim_mut(&mut self) -> Result<&mut TokenStream, UnexpectedImplItemKind> {
        if let ImplItem::Verbatim(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedImplItemKind(self.clone()))
    }
}

#[derive(Debug)]
pub(crate) struct UnexpectedTypeKind(Type);
impl ExpectableError for UnexpectedTypeKind {
    fn abort(&self) -> ! {
        abort!(
            self.0,
            format!("Found unexpected kind of Type: {:?}", self.0)
        )
    }
}

pub(crate) trait TypeExpectable {
    fn as_array(&self) -> Result<&TypeArray, UnexpectedTypeKind>;
    fn as_bare_fn(&self) -> Result<&TypeBareFn, UnexpectedTypeKind>;
    fn as_group(&self) -> Result<&TypeGroup, UnexpectedTypeKind>;
    fn as_impl_trait(&self) -> Result<&TypeImplTrait, UnexpectedTypeKind>;
    fn as_infer(&self) -> Result<&TypeInfer, UnexpectedTypeKind>;
    fn as_macro(&self) -> Result<&TypeMacro, UnexpectedTypeKind>;
    fn as_never(&self) -> Result<&TypeNever, UnexpectedTypeKind>;
    fn as_paren(&self) -> Result<&TypeParen, UnexpectedTypeKind>;
    fn as_path(&self) -> Result<&TypePath, UnexpectedTypeKind>;
    fn as_ptr(&self) -> Result<&TypePtr, UnexpectedTypeKind>;
    fn as_reference(&self) -> Result<&TypeReference, UnexpectedTypeKind>;
    fn as_slice(&self) -> Result<&TypeSlice, UnexpectedTypeKind>;
    fn as_trait_object(&self) -> Result<&TypeTraitObject, UnexpectedTypeKind>;
    fn as_tuple(&self) -> Result<&TypeTuple, UnexpectedTypeKind>;
    fn as_verbatim(&self) -> Result<&TokenStream, UnexpectedTypeKind>;

    fn as_array_mut(&mut self) -> Result<&mut TypeArray, UnexpectedTypeKind>;
    fn as_bare_fn_mut(&mut self) -> Result<&mut TypeBareFn, UnexpectedTypeKind>;
    fn as_group_mut(&mut self) -> Result<&mut TypeGroup, UnexpectedTypeKind>;
    fn as_impl_trait_mut(&mut self) -> Result<&mut TypeImplTrait, UnexpectedTypeKind>;
    fn as_infer_mut(&mut self) -> Result<&mut TypeInfer, UnexpectedTypeKind>;
    fn as_macro_mut(&mut self) -> Result<&mut TypeMacro, UnexpectedTypeKind>;
    fn as_never_mut(&mut self) -> Result<&mut TypeNever, UnexpectedTypeKind>;
    fn as_paren_mut(&mut self) -> Result<&mut TypeParen, UnexpectedTypeKind>;
    fn as_path_mut(&mut self) -> Result<&mut TypePath, UnexpectedTypeKind>;
    fn as_ptr_mut(&mut self) -> Result<&mut TypePtr, UnexpectedTypeKind>;
    fn as_reference_mut(&mut self) -> Result<&mut TypeReference, UnexpectedTypeKind>;
    fn as_slice_mut(&mut self) -> Result<&mut TypeSlice, UnexpectedTypeKind>;
    fn as_trait_object_mut(&mut self) -> Result<&mut TypeTraitObject, UnexpectedTypeKind>;
    fn as_tuple_mut(&mut self) -> Result<&mut TypeTuple, UnexpectedTypeKind>;
    fn as_verbatim_mut(&mut self) -> Result<&mut TokenStream, UnexpectedTypeKind>;
}

impl TypeExpectable for Type {
    fn as_array(&self) -> Result<&TypeArray, UnexpectedTypeKind> {
        if let Type::Array(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_bare_fn(&self) -> Result<&TypeBareFn, UnexpectedTypeKind> {
        if let Type::BareFn(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_group(&self) -> Result<&TypeGroup, UnexpectedTypeKind> {
        if let Type::Group(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_impl_trait(&self) -> Result<&TypeImplTrait, UnexpectedTypeKind> {
        if let Type::ImplTrait(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_infer(&self) -> Result<&TypeInfer, UnexpectedTypeKind> {
        if let Type::Infer(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_macro(&self) -> Result<&TypeMacro, UnexpectedTypeKind> {
        if let Type::Macro(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_never(&self) -> Result<&TypeNever, UnexpectedTypeKind> {
        if let Type::Never(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_paren(&self) -> Result<&TypeParen, UnexpectedTypeKind> {
        if let Type::Paren(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_path(&self) -> Result<&TypePath, UnexpectedTypeKind> {
        if let Type::Path(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_ptr(&self) -> Result<&TypePtr, UnexpectedTypeKind> {
        if let Type::Ptr(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_reference(&self) -> Result<&TypeReference, UnexpectedTypeKind> {
        if let Type::Reference(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_slice(&self) -> Result<&TypeSlice, UnexpectedTypeKind> {
        if let Type::Slice(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_trait_object(&self) -> Result<&TypeTraitObject, UnexpectedTypeKind> {
        if let Type::TraitObject(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_tuple(&self) -> Result<&TypeTuple, UnexpectedTypeKind> {
        if let Type::Tuple(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_verbatim(&self) -> Result<&TokenStream, UnexpectedTypeKind> {
        if let Type::Verbatim(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_array_mut(&mut self) -> Result<&mut TypeArray, UnexpectedTypeKind> {
        if let Type::Array(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_bare_fn_mut(&mut self) -> Result<&mut TypeBareFn, UnexpectedTypeKind> {
        if let Type::BareFn(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_group_mut(&mut self) -> Result<&mut TypeGroup, UnexpectedTypeKind> {
        if let Type::Group(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_impl_trait_mut(&mut self) -> Result<&mut TypeImplTrait, UnexpectedTypeKind> {
        if let Type::ImplTrait(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_infer_mut(&mut self) -> Result<&mut TypeInfer, UnexpectedTypeKind> {
        if let Type::Infer(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_macro_mut(&mut self) -> Result<&mut TypeMacro, UnexpectedTypeKind> {
        if let Type::Macro(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_never_mut(&mut self) -> Result<&mut TypeNever, UnexpectedTypeKind> {
        if let Type::Never(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_paren_mut(&mut self) -> Result<&mut TypeParen, UnexpectedTypeKind> {
        if let Type::Paren(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_path_mut(&mut self) -> Result<&mut TypePath, UnexpectedTypeKind> {
        if let Type::Path(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_ptr_mut(&mut self) -> Result<&mut TypePtr, UnexpectedTypeKind> {
        if let Type::Ptr(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_reference_mut(&mut self) -> Result<&mut TypeReference, UnexpectedTypeKind> {
        if let Type::Reference(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_slice_mut(&mut self) -> Result<&mut TypeSlice, UnexpectedTypeKind> {
        if let Type::Slice(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_trait_object_mut(&mut self) -> Result<&mut TypeTraitObject, UnexpectedTypeKind> {
        if let Type::TraitObject(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_tuple_mut(&mut self) -> Result<&mut TypeTuple, UnexpectedTypeKind> {
        if let Type::Tuple(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }

    fn as_verbatim_mut(&mut self) -> Result<&mut TokenStream, UnexpectedTypeKind> {
        if let Type::Verbatim(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTypeKind(self.clone()))
    }
}

#[derive(Debug)]
pub(crate) struct UnexpectedPatKind(Pat);
impl ExpectableError for UnexpectedPatKind {
    fn abort(&self) -> ! {
        abort!(
            self.0,
            format!("Found unexpected kind of Pat: {:?}", self.0)
        )
    }
}

pub(crate) trait PatExpectable {
    fn as_const(&self) -> Result<&PatConst, UnexpectedPatKind>;
    fn as_ident(&self) -> Result<&PatIdent, UnexpectedPatKind>;
    fn as_lit(&self) -> Result<&PatLit, UnexpectedPatKind>;
    fn as_macro(&self) -> Result<&PatMacro, UnexpectedPatKind>;
    fn as_or(&self) -> Result<&PatOr, UnexpectedPatKind>;
    fn as_paren(&self) -> Result<&PatParen, UnexpectedPatKind>;
    fn as_path(&self) -> Result<&PatPath, UnexpectedPatKind>;
    fn as_range(&self) -> Result<&PatRange, UnexpectedPatKind>;
    fn as_reference(&self) -> Result<&PatReference, UnexpectedPatKind>;
    fn as_rest(&self) -> Result<&PatRest, UnexpectedPatKind>;
    fn as_slice(&self) -> Result<&PatSlice, UnexpectedPatKind>;
    fn as_struct(&self) -> Result<&PatStruct, UnexpectedPatKind>;
    fn as_tuple(&self) -> Result<&PatTuple, UnexpectedPatKind>;
    fn as_tuple_struct(&self) -> Result<&PatTupleStruct, UnexpectedPatKind>;
    fn as_type(&self) -> Result<&PatType, UnexpectedPatKind>;
    fn as_verbatim(&self) -> Result<&TokenStream, UnexpectedPatKind>;
    fn as_wild(&self) -> Result<&PatWild, UnexpectedPatKind>;

    fn as_const_mut(&mut self) -> Result<&mut PatConst, UnexpectedPatKind>;
    fn as_ident_mut(&mut self) -> Result<&mut PatIdent, UnexpectedPatKind>;
    fn as_lit_mut(&mut self) -> Result<&mut PatLit, UnexpectedPatKind>;
    fn as_macro_mut(&mut self) -> Result<&mut PatMacro, UnexpectedPatKind>;
    fn as_or_mut(&mut self) -> Result<&mut PatOr, UnexpectedPatKind>;
    fn as_paren_mut(&mut self) -> Result<&mut PatParen, UnexpectedPatKind>;
    fn as_path_mut(&mut self) -> Result<&mut PatPath, UnexpectedPatKind>;
    fn as_range_mut(&mut self) -> Result<&mut PatRange, UnexpectedPatKind>;
    fn as_reference_mut(&mut self) -> Result<&mut PatReference, UnexpectedPatKind>;
    fn as_rest_mut(&mut self) -> Result<&mut PatRest, UnexpectedPatKind>;
    fn as_slice_mut(&mut self) -> Result<&mut PatSlice, UnexpectedPatKind>;
    fn as_struct_mut(&mut self) -> Result<&mut PatStruct, UnexpectedPatKind>;
    fn as_tuple_mut(&mut self) -> Result<&mut PatTuple, UnexpectedPatKind>;
    fn as_tuple_struct_mut(&mut self) -> Result<&mut PatTupleStruct, UnexpectedPatKind>;
    fn as_type_mut(&mut self) -> Result<&mut PatType, UnexpectedPatKind>;
    fn as_verbatim_mut(&mut self) -> Result<&mut TokenStream, UnexpectedPatKind>;
    fn as_wild_mut(&mut self) -> Result<&mut PatWild, UnexpectedPatKind>;
}

impl PatExpectable for Pat {
    fn as_const(&self) -> Result<&PatConst, UnexpectedPatKind> {
        if let Pat::Const(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_ident(&self) -> Result<&PatIdent, UnexpectedPatKind> {
        if let Pat::Ident(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_lit(&self) -> Result<&PatLit, UnexpectedPatKind> {
        if let Pat::Lit(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_macro(&self) -> Result<&PatMacro, UnexpectedPatKind> {
        if let Pat::Macro(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_or(&self) -> Result<&PatOr, UnexpectedPatKind> {
        if let Pat::Or(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_paren(&self) -> Result<&PatParen, UnexpectedPatKind> {
        if let Pat::Paren(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_path(&self) -> Result<&PatPath, UnexpectedPatKind> {
        if let Pat::Path(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_range(&self) -> Result<&PatRange, UnexpectedPatKind> {
        if let Pat::Range(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_reference(&self) -> Result<&PatReference, UnexpectedPatKind> {
        if let Pat::Reference(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_rest(&self) -> Result<&PatRest, UnexpectedPatKind> {
        if let Pat::Rest(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_slice(&self) -> Result<&PatSlice, UnexpectedPatKind> {
        if let Pat::Slice(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_struct(&self) -> Result<&PatStruct, UnexpectedPatKind> {
        if let Pat::Struct(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_tuple(&self) -> Result<&PatTuple, UnexpectedPatKind> {
        if let Pat::Tuple(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_tuple_struct(&self) -> Result<&PatTupleStruct, UnexpectedPatKind> {
        if let Pat::TupleStruct(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_type(&self) -> Result<&PatType, UnexpectedPatKind> {
        if let Pat::Type(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_verbatim(&self) -> Result<&TokenStream, UnexpectedPatKind> {
        if let Pat::Verbatim(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_wild(&self) -> Result<&PatWild, UnexpectedPatKind> {
        if let Pat::Wild(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_const_mut(&mut self) -> Result<&mut PatConst, UnexpectedPatKind> {
        if let Pat::Const(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_ident_mut(&mut self) -> Result<&mut PatIdent, UnexpectedPatKind> {
        if let Pat::Ident(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_lit_mut(&mut self) -> Result<&mut PatLit, UnexpectedPatKind> {
        if let Pat::Lit(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_macro_mut(&mut self) -> Result<&mut PatMacro, UnexpectedPatKind> {
        if let Pat::Macro(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_or_mut(&mut self) -> Result<&mut PatOr, UnexpectedPatKind> {
        if let Pat::Or(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_paren_mut(&mut self) -> Result<&mut PatParen, UnexpectedPatKind> {
        if let Pat::Paren(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_path_mut(&mut self) -> Result<&mut PatPath, UnexpectedPatKind> {
        if let Pat::Path(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_range_mut(&mut self) -> Result<&mut PatRange, UnexpectedPatKind> {
        if let Pat::Range(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_reference_mut(&mut self) -> Result<&mut PatReference, UnexpectedPatKind> {
        if let Pat::Reference(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_rest_mut(&mut self) -> Result<&mut PatRest, UnexpectedPatKind> {
        if let Pat::Rest(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_slice_mut(&mut self) -> Result<&mut PatSlice, UnexpectedPatKind> {
        if let Pat::Slice(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_struct_mut(&mut self) -> Result<&mut PatStruct, UnexpectedPatKind> {
        if let Pat::Struct(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_tuple_mut(&mut self) -> Result<&mut PatTuple, UnexpectedPatKind> {
        if let Pat::Tuple(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_tuple_struct_mut(&mut self) -> Result<&mut PatTupleStruct, UnexpectedPatKind> {
        if let Pat::TupleStruct(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_type_mut(&mut self) -> Result<&mut PatType, UnexpectedPatKind> {
        if let Pat::Type(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_verbatim_mut(&mut self) -> Result<&mut TokenStream, UnexpectedPatKind> {
        if let Pat::Verbatim(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }

    fn as_wild_mut(&mut self) -> Result<&mut PatWild, UnexpectedPatKind> {
        if let Pat::Wild(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPatKind(self.clone()))
    }
}

#[derive(Debug)]
pub(crate) struct UnexpectedFnArgKind(FnArg);
impl ExpectableError for UnexpectedFnArgKind {
    fn abort(&self) -> ! {
        abort!(
            self.0,
            format!("Found unexpected kind of FnArg: {:?}", self.0)
        )
    }
}

pub(crate) trait FnArgExpectable {
    fn as_receiver(&self) -> Result<&Receiver, UnexpectedFnArgKind>;
    fn as_typed(&self) -> Result<&PatType, UnexpectedFnArgKind>;

    fn as_receiver_mut(&mut self) -> Result<&mut Receiver, UnexpectedFnArgKind>;
    fn as_typed_mut(&mut self) -> Result<&mut PatType, UnexpectedFnArgKind>;
}

impl FnArgExpectable for FnArg {
    fn as_receiver(&self) -> Result<&Receiver, UnexpectedFnArgKind> {
        if let FnArg::Receiver(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedFnArgKind(self.clone()))
    }

    fn as_typed(&self) -> Result<&PatType, UnexpectedFnArgKind> {
        if let FnArg::Typed(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedFnArgKind(self.clone()))
    }

    fn as_receiver_mut(&mut self) -> Result<&mut Receiver, UnexpectedFnArgKind> {
        if let FnArg::Receiver(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedFnArgKind(self.clone()))
    }

    fn as_typed_mut(&mut self) -> Result<&mut PatType, UnexpectedFnArgKind> {
        if let FnArg::Typed(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedFnArgKind(self.clone()))
    }
}

#[derive(Debug)]
pub(crate) struct UnexpectedTraitItemKind(TraitItem);
impl ExpectableError for UnexpectedTraitItemKind {
    fn abort(&self) -> ! {
        abort!(
            self.0,
            format!("Found unexpected kind of TraitItem: {:?}", self.0)
        )
    }
}

pub(crate) trait TraitItemExpectable {
    fn as_const(&self) -> Result<&TraitItemConst, UnexpectedTraitItemKind>;
    fn as_fn(&self) -> Result<&TraitItemFn, UnexpectedTraitItemKind>;
    fn as_type(&self) -> Result<&TraitItemType, UnexpectedTraitItemKind>;
    fn as_macro(&self) -> Result<&TraitItemMacro, UnexpectedTraitItemKind>;
    fn as_verbatim(&self) -> Result<&TokenStream, UnexpectedTraitItemKind>;

    fn as_const_mut(&mut self) -> Result<&mut TraitItemConst, UnexpectedTraitItemKind>;
    fn as_fn_mut(&mut self) -> Result<&mut TraitItemFn, UnexpectedTraitItemKind>;
    fn as_type_mut(&mut self) -> Result<&mut TraitItemType, UnexpectedTraitItemKind>;
    fn as_macro_mut(&mut self) -> Result<&mut TraitItemMacro, UnexpectedTraitItemKind>;
    fn as_verbatim_mut(&mut self) -> Result<&mut TokenStream, UnexpectedTraitItemKind>;
}

impl TraitItemExpectable for TraitItem {
    fn as_const(&self) -> Result<&TraitItemConst, UnexpectedTraitItemKind> {
        if let TraitItem::Const(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTraitItemKind(self.clone()))
    }

    fn as_fn(&self) -> Result<&TraitItemFn, UnexpectedTraitItemKind> {
        if let TraitItem::Fn(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTraitItemKind(self.clone()))
    }

    fn as_type(&self) -> Result<&TraitItemType, UnexpectedTraitItemKind> {
        if let TraitItem::Type(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTraitItemKind(self.clone()))
    }

    fn as_macro(&self) -> Result<&TraitItemMacro, UnexpectedTraitItemKind> {
        if let TraitItem::Macro(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTraitItemKind(self.clone()))
    }

    fn as_verbatim(&self) -> Result<&TokenStream, UnexpectedTraitItemKind> {
        if let TraitItem::Verbatim(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTraitItemKind(self.clone()))
    }

    fn as_const_mut(&mut self) -> Result<&mut TraitItemConst, UnexpectedTraitItemKind> {
        if let TraitItem::Const(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTraitItemKind(self.clone()))
    }

    fn as_fn_mut(&mut self) -> Result<&mut TraitItemFn, UnexpectedTraitItemKind> {
        if let TraitItem::Fn(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTraitItemKind(self.clone()))
    }

    fn as_type_mut(&mut self) -> Result<&mut TraitItemType, UnexpectedTraitItemKind> {
        if let TraitItem::Type(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTraitItemKind(self.clone()))
    }

    fn as_macro_mut(&mut self) -> Result<&mut TraitItemMacro, UnexpectedTraitItemKind> {
        if let TraitItem::Macro(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTraitItemKind(self.clone()))
    }

    fn as_verbatim_mut(&mut self) -> Result<&mut TokenStream, UnexpectedTraitItemKind> {
        if let TraitItem::Verbatim(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedTraitItemKind(self.clone()))
    }
}

#[derive(Debug)]
pub(crate) struct UnexpectedReturnTypeKind(ReturnType);
impl ExpectableError for UnexpectedReturnTypeKind {
    fn abort(&self) -> ! {
        abort!(
            self.0,
            format!("Found unexpected kind of ReturnType: {:?}", self.0)
        )
    }
}

pub(crate) trait ReturnTypeExpectable {
    fn as_type(&self) -> Result<(&RArrow, &Type), UnexpectedReturnTypeKind>;
    fn as_type_mut(&mut self) -> Result<(&mut RArrow, &mut Box<Type>), UnexpectedReturnTypeKind>;
}

impl ReturnTypeExpectable for ReturnType {
    fn as_type(&self) -> Result<(&RArrow, &Type), UnexpectedReturnTypeKind> {
        if let ReturnType::Type(arrow, ty) = self {
            return Ok((arrow, ty));
        }
        Err(UnexpectedReturnTypeKind(self.clone()))
    }

    fn as_type_mut(&mut self) -> Result<(&mut RArrow, &mut Box<Type>), UnexpectedReturnTypeKind> {
        if let ReturnType::Type(arrow, ty) = self {
            return Ok((arrow, ty));
        }
        Err(UnexpectedReturnTypeKind(self.clone()))
    }
}

#[derive(Debug)]
pub(crate) struct UnexpectedGenericArgumentKind(GenericArgument);
impl ExpectableError for UnexpectedGenericArgumentKind {
    fn abort(&self) -> ! {
        abort!(
            self.0,
            format!("Found unexpected kind of GenericArgument: {:?}", self.0)
        )
    }
}

pub(crate) trait GenericArgumentExpectable {
    fn as_lifetime(&self) -> Result<&Lifetime, UnexpectedGenericArgumentKind>;
    fn as_type(&self) -> Result<&Type, UnexpectedGenericArgumentKind>;
    fn as_const(&self) -> Result<&Expr, UnexpectedGenericArgumentKind>;
    fn as_assoc_type(&self) -> Result<&AssocType, UnexpectedGenericArgumentKind>;
    fn as_assoc_const(&self) -> Result<&AssocConst, UnexpectedGenericArgumentKind>;
    fn as_constraint(&self) -> Result<&Constraint, UnexpectedGenericArgumentKind>;

    fn as_lifetime_mut(&mut self) -> Result<&mut Lifetime, UnexpectedGenericArgumentKind>;
    fn as_type_mut(&mut self) -> Result<&mut Type, UnexpectedGenericArgumentKind>;
    fn as_const_mut(&mut self) -> Result<&mut Expr, UnexpectedGenericArgumentKind>;
    fn as_assoc_type_mut(&mut self) -> Result<&mut AssocType, UnexpectedGenericArgumentKind>;
    fn as_assoc_const_mut(&mut self) -> Result<&mut AssocConst, UnexpectedGenericArgumentKind>;
    fn as_constraint_mut(&mut self) -> Result<&mut Constraint, UnexpectedGenericArgumentKind>;
}

impl GenericArgumentExpectable for GenericArgument {
    fn as_lifetime(&self) -> Result<&Lifetime, UnexpectedGenericArgumentKind> {
        if let GenericArgument::Lifetime(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericArgumentKind(self.clone()))
    }

    fn as_type(&self) -> Result<&Type, UnexpectedGenericArgumentKind> {
        if let GenericArgument::Type(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericArgumentKind(self.clone()))
    }

    fn as_const(&self) -> Result<&Expr, UnexpectedGenericArgumentKind> {
        if let GenericArgument::Const(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericArgumentKind(self.clone()))
    }

    fn as_assoc_type(&self) -> Result<&AssocType, UnexpectedGenericArgumentKind> {
        if let GenericArgument::AssocType(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericArgumentKind(self.clone()))
    }

    fn as_assoc_const(&self) -> Result<&AssocConst, UnexpectedGenericArgumentKind> {
        if let GenericArgument::AssocConst(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericArgumentKind(self.clone()))
    }

    fn as_constraint(&self) -> Result<&Constraint, UnexpectedGenericArgumentKind> {
        if let GenericArgument::Constraint(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericArgumentKind(self.clone()))
    }

    fn as_lifetime_mut(&mut self) -> Result<&mut Lifetime, UnexpectedGenericArgumentKind> {
        if let GenericArgument::Lifetime(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericArgumentKind(self.clone()))
    }

    fn as_type_mut(&mut self) -> Result<&mut Type, UnexpectedGenericArgumentKind> {
        if let GenericArgument::Type(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericArgumentKind(self.clone()))
    }

    fn as_const_mut(&mut self) -> Result<&mut Expr, UnexpectedGenericArgumentKind> {
        if let GenericArgument::Const(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericArgumentKind(self.clone()))
    }

    fn as_assoc_type_mut(&mut self) -> Result<&mut AssocType, UnexpectedGenericArgumentKind> {
        if let GenericArgument::AssocType(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericArgumentKind(self.clone()))
    }

    fn as_assoc_const_mut(&mut self) -> Result<&mut AssocConst, UnexpectedGenericArgumentKind> {
        if let GenericArgument::AssocConst(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericArgumentKind(self.clone()))
    }

    fn as_constraint_mut(&mut self) -> Result<&mut Constraint, UnexpectedGenericArgumentKind> {
        if let GenericArgument::Constraint(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericArgumentKind(self.clone()))
    }
}

#[derive(Debug)]
pub(crate) struct UnexpectedPathArgumentsKind(pub(crate) PathArguments);
impl ExpectableError for UnexpectedPathArgumentsKind {
    fn abort(&self) -> ! {
        abort!(
            self.0,
            format!("Found unexpected kind of PathArguments: {:?}", self.0)
        )
    }
}

pub(crate) trait PathArgumentsExpectable {
    fn expect_none(&self) -> Result<(), UnexpectedPathArgumentsKind>;

    fn as_angle_bracketed(
        &self,
    ) -> Result<&AngleBracketedGenericArguments, UnexpectedPathArgumentsKind>;
    fn as_parenthesized(
        &self,
    ) -> Result<&ParenthesizedGenericArguments, UnexpectedPathArgumentsKind>;

    fn as_angle_bracketed_mut(
        &mut self,
    ) -> Result<&mut AngleBracketedGenericArguments, UnexpectedPathArgumentsKind>;
    fn as_parenthesized_mut(
        &mut self,
    ) -> Result<&mut ParenthesizedGenericArguments, UnexpectedPathArgumentsKind>;
}

impl PathArgumentsExpectable for PathArguments {
    fn expect_none(&self) -> Result<(), UnexpectedPathArgumentsKind> {
        if let PathArguments::None = self {
            return Ok(());
        }
        Err(UnexpectedPathArgumentsKind(self.clone()))
    }

    fn as_angle_bracketed(
        &self,
    ) -> Result<&AngleBracketedGenericArguments, UnexpectedPathArgumentsKind> {
        if let PathArguments::AngleBracketed(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPathArgumentsKind(self.clone()))
    }

    fn as_parenthesized(
        &self,
    ) -> Result<&ParenthesizedGenericArguments, UnexpectedPathArgumentsKind> {
        if let PathArguments::Parenthesized(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPathArgumentsKind(self.clone()))
    }

    fn as_angle_bracketed_mut(
        &mut self,
    ) -> Result<&mut AngleBracketedGenericArguments, UnexpectedPathArgumentsKind> {
        if let PathArguments::AngleBracketed(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPathArgumentsKind(self.clone()))
    }

    fn as_parenthesized_mut(
        &mut self,
    ) -> Result<&mut ParenthesizedGenericArguments, UnexpectedPathArgumentsKind> {
        if let PathArguments::Parenthesized(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedPathArgumentsKind(self.clone()))
    }
}

#[derive(Debug)]
pub(crate) struct UnexpectedGenericParamKind(GenericParam);
impl ExpectableError for UnexpectedGenericParamKind {
    fn abort(&self) -> ! {
        abort!(
            self.0,
            format!("Found unexpected kind of GenericParam: {:?}", self.0)
        )
    }
}

pub(crate) trait GenericParamExpectable {
    fn as_lifetime(&self) -> Result<&LifetimeParam, UnexpectedGenericParamKind>;
    fn as_type(&self) -> Result<&TypeParam, UnexpectedGenericParamKind>;
    fn as_const(&self) -> Result<&ConstParam, UnexpectedGenericParamKind>;

    fn as_lifetime_mut(&mut self) -> Result<&mut LifetimeParam, UnexpectedGenericParamKind>;
    fn as_type_mut(&mut self) -> Result<&mut TypeParam, UnexpectedGenericParamKind>;
    fn as_const_mut(&mut self) -> Result<&mut ConstParam, UnexpectedGenericParamKind>;
}

impl GenericParamExpectable for GenericParam {
    fn as_lifetime(&self) -> Result<&LifetimeParam, UnexpectedGenericParamKind> {
        if let GenericParam::Lifetime(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericParamKind(self.clone()))
    }

    fn as_type(&self) -> Result<&TypeParam, UnexpectedGenericParamKind> {
        if let GenericParam::Type(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericParamKind(self.clone()))
    }

    fn as_const(&self) -> Result<&ConstParam, UnexpectedGenericParamKind> {
        if let GenericParam::Const(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericParamKind(self.clone()))
    }

    fn as_lifetime_mut(&mut self) -> Result<&mut LifetimeParam, UnexpectedGenericParamKind> {
        if let GenericParam::Lifetime(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericParamKind(self.clone()))
    }

    fn as_type_mut(&mut self) -> Result<&mut TypeParam, UnexpectedGenericParamKind> {
        if let GenericParam::Type(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericParamKind(self.clone()))
    }

    fn as_const_mut(&mut self) -> Result<&mut ConstParam, UnexpectedGenericParamKind> {
        if let GenericParam::Const(inner) = self {
            return Ok(inner);
        }
        Err(UnexpectedGenericParamKind(self.clone()))
    }
}
