use proc_macro2::TokenStream;
use syn::{
    token::RArrow, AngleBracketedGenericArguments, AssocConst, AssocType, ConstParam, Constraint,
    Expr, FnArg, GenericArgument, GenericParam, Lifetime, LifetimeParam,
    ParenthesizedGenericArguments, Pat, PatConst, PatIdent, PatLit, PatMacro, PatOr, PatParen,
    PatPath, PatRange, PatReference, PatRest, PatSlice, PatStruct, PatTuple, PatTupleStruct,
    PatType, PatWild, PathArguments, Receiver, ReturnType, TraitItem, TraitItemConst, TraitItemFn,
    TraitItemMacro, TraitItemType, Type, TypeArray, TypeBareFn, TypeGroup, TypeImplTrait,
    TypeInfer, TypeMacro, TypeNever, TypeParam, TypeParen, TypePath, TypePtr, TypeReference,
    TypeSlice, TypeTraitObject, TypeTuple,
};

use crate::ParsingError;

pub trait TypeExpectable {
    fn as_array(&self) -> Result<&TypeArray, ParsingError>;
    fn as_bare_fn(&self) -> Result<&TypeBareFn, ParsingError>;
    fn as_group(&self) -> Result<&TypeGroup, ParsingError>;
    fn as_impl_trait(&self) -> Result<&TypeImplTrait, ParsingError>;
    fn as_infer(&self) -> Result<&TypeInfer, ParsingError>;
    fn as_macro(&self) -> Result<&TypeMacro, ParsingError>;
    fn as_never(&self) -> Result<&TypeNever, ParsingError>;
    fn as_paren(&self) -> Result<&TypeParen, ParsingError>;
    fn as_path(&self) -> Result<&TypePath, ParsingError>;
    fn as_ptr(&self) -> Result<&TypePtr, ParsingError>;
    fn as_reference(&self) -> Result<&TypeReference, ParsingError>;
    fn as_slice(&self) -> Result<&TypeSlice, ParsingError>;
    fn as_trait_object(&self) -> Result<&TypeTraitObject, ParsingError>;
    fn as_tuple(&self) -> Result<&TypeTuple, ParsingError>;
    fn as_verbatim(&self) -> Result<&TokenStream, ParsingError>;

    fn as_array_mut(&mut self) -> Result<&mut TypeArray, ParsingError>;
    fn as_bare_fn_mut(&mut self) -> Result<&mut TypeBareFn, ParsingError>;
    fn as_group_mut(&mut self) -> Result<&mut TypeGroup, ParsingError>;
    fn as_impl_trait_mut(&mut self) -> Result<&mut TypeImplTrait, ParsingError>;
    fn as_infer_mut(&mut self) -> Result<&mut TypeInfer, ParsingError>;
    fn as_macro_mut(&mut self) -> Result<&mut TypeMacro, ParsingError>;
    fn as_never_mut(&mut self) -> Result<&mut TypeNever, ParsingError>;
    fn as_paren_mut(&mut self) -> Result<&mut TypeParen, ParsingError>;
    fn as_path_mut(&mut self) -> Result<&mut TypePath, ParsingError>;
    fn as_ptr_mut(&mut self) -> Result<&mut TypePtr, ParsingError>;
    fn as_reference_mut(&mut self) -> Result<&mut TypeReference, ParsingError>;
    fn as_slice_mut(&mut self) -> Result<&mut TypeSlice, ParsingError>;
    fn as_trait_object_mut(&mut self) -> Result<&mut TypeTraitObject, ParsingError>;
    fn as_tuple_mut(&mut self) -> Result<&mut TypeTuple, ParsingError>;
    fn as_verbatim_mut(&mut self) -> Result<&mut TokenStream, ParsingError>;
}

impl TypeExpectable for Type {
    fn as_array(&self) -> Result<&TypeArray, ParsingError> {
        if let Type::Array(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_bare_fn(&self) -> Result<&TypeBareFn, ParsingError> {
        if let Type::BareFn(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_group(&self) -> Result<&TypeGroup, ParsingError> {
        if let Type::Group(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_impl_trait(&self) -> Result<&TypeImplTrait, ParsingError> {
        if let Type::ImplTrait(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_infer(&self) -> Result<&TypeInfer, ParsingError> {
        if let Type::Infer(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_macro(&self) -> Result<&TypeMacro, ParsingError> {
        if let Type::Macro(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_never(&self) -> Result<&TypeNever, ParsingError> {
        if let Type::Never(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_paren(&self) -> Result<&TypeParen, ParsingError> {
        if let Type::Paren(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_path(&self) -> Result<&TypePath, ParsingError> {
        if let Type::Path(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_ptr(&self) -> Result<&TypePtr, ParsingError> {
        if let Type::Ptr(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_reference(&self) -> Result<&TypeReference, ParsingError> {
        if let Type::Reference(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_slice(&self) -> Result<&TypeSlice, ParsingError> {
        if let Type::Slice(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_trait_object(&self) -> Result<&TypeTraitObject, ParsingError> {
        if let Type::TraitObject(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_tuple(&self) -> Result<&TypeTuple, ParsingError> {
        if let Type::Tuple(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_verbatim(&self) -> Result<&TokenStream, ParsingError> {
        if let Type::Verbatim(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_array_mut(&mut self) -> Result<&mut TypeArray, ParsingError> {
        if let Type::Array(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_bare_fn_mut(&mut self) -> Result<&mut TypeBareFn, ParsingError> {
        if let Type::BareFn(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_group_mut(&mut self) -> Result<&mut TypeGroup, ParsingError> {
        if let Type::Group(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_impl_trait_mut(&mut self) -> Result<&mut TypeImplTrait, ParsingError> {
        if let Type::ImplTrait(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_infer_mut(&mut self) -> Result<&mut TypeInfer, ParsingError> {
        if let Type::Infer(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_macro_mut(&mut self) -> Result<&mut TypeMacro, ParsingError> {
        if let Type::Macro(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_never_mut(&mut self) -> Result<&mut TypeNever, ParsingError> {
        if let Type::Never(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_paren_mut(&mut self) -> Result<&mut TypeParen, ParsingError> {
        if let Type::Paren(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_path_mut(&mut self) -> Result<&mut TypePath, ParsingError> {
        if let Type::Path(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_ptr_mut(&mut self) -> Result<&mut TypePtr, ParsingError> {
        if let Type::Ptr(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_reference_mut(&mut self) -> Result<&mut TypeReference, ParsingError> {
        if let Type::Reference(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_slice_mut(&mut self) -> Result<&mut TypeSlice, ParsingError> {
        if let Type::Slice(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_trait_object_mut(&mut self) -> Result<&mut TypeTraitObject, ParsingError> {
        if let Type::TraitObject(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_tuple_mut(&mut self) -> Result<&mut TypeTuple, ParsingError> {
        if let Type::Tuple(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }

    fn as_verbatim_mut(&mut self) -> Result<&mut TokenStream, ParsingError> {
        if let Type::Verbatim(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedType(self.clone()))
    }
}

pub trait PatExpectable {
    fn as_const(&self) -> Result<&PatConst, ParsingError>;
    fn as_ident(&self) -> Result<&PatIdent, ParsingError>;
    fn as_lit(&self) -> Result<&PatLit, ParsingError>;
    fn as_macro(&self) -> Result<&PatMacro, ParsingError>;
    fn as_or(&self) -> Result<&PatOr, ParsingError>;
    fn as_paren(&self) -> Result<&PatParen, ParsingError>;
    fn as_path(&self) -> Result<&PatPath, ParsingError>;
    fn as_range(&self) -> Result<&PatRange, ParsingError>;
    fn as_reference(&self) -> Result<&PatReference, ParsingError>;
    fn as_rest(&self) -> Result<&PatRest, ParsingError>;
    fn as_slice(&self) -> Result<&PatSlice, ParsingError>;
    fn as_struct(&self) -> Result<&PatStruct, ParsingError>;
    fn as_tuple(&self) -> Result<&PatTuple, ParsingError>;
    fn as_tuple_struct(&self) -> Result<&PatTupleStruct, ParsingError>;
    fn as_type(&self) -> Result<&PatType, ParsingError>;
    fn as_verbatim(&self) -> Result<&TokenStream, ParsingError>;
    fn as_wild(&self) -> Result<&PatWild, ParsingError>;

    fn as_const_mut(&mut self) -> Result<&mut PatConst, ParsingError>;
    fn as_ident_mut(&mut self) -> Result<&mut PatIdent, ParsingError>;
    fn as_lit_mut(&mut self) -> Result<&mut PatLit, ParsingError>;
    fn as_macro_mut(&mut self) -> Result<&mut PatMacro, ParsingError>;
    fn as_or_mut(&mut self) -> Result<&mut PatOr, ParsingError>;
    fn as_paren_mut(&mut self) -> Result<&mut PatParen, ParsingError>;
    fn as_path_mut(&mut self) -> Result<&mut PatPath, ParsingError>;
    fn as_range_mut(&mut self) -> Result<&mut PatRange, ParsingError>;
    fn as_reference_mut(&mut self) -> Result<&mut PatReference, ParsingError>;
    fn as_rest_mut(&mut self) -> Result<&mut PatRest, ParsingError>;
    fn as_slice_mut(&mut self) -> Result<&mut PatSlice, ParsingError>;
    fn as_struct_mut(&mut self) -> Result<&mut PatStruct, ParsingError>;
    fn as_tuple_mut(&mut self) -> Result<&mut PatTuple, ParsingError>;
    fn as_tuple_struct_mut(&mut self) -> Result<&mut PatTupleStruct, ParsingError>;
    fn as_type_mut(&mut self) -> Result<&mut PatType, ParsingError>;
    fn as_verbatim_mut(&mut self) -> Result<&mut TokenStream, ParsingError>;
    fn as_wild_mut(&mut self) -> Result<&mut PatWild, ParsingError>;
}

impl PatExpectable for Pat {
    fn as_const(&self) -> Result<&PatConst, ParsingError> {
        if let Pat::Const(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_ident(&self) -> Result<&PatIdent, ParsingError> {
        if let Pat::Ident(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_lit(&self) -> Result<&PatLit, ParsingError> {
        if let Pat::Lit(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_macro(&self) -> Result<&PatMacro, ParsingError> {
        if let Pat::Macro(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_or(&self) -> Result<&PatOr, ParsingError> {
        if let Pat::Or(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_paren(&self) -> Result<&PatParen, ParsingError> {
        if let Pat::Paren(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_path(&self) -> Result<&PatPath, ParsingError> {
        if let Pat::Path(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_range(&self) -> Result<&PatRange, ParsingError> {
        if let Pat::Range(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_reference(&self) -> Result<&PatReference, ParsingError> {
        if let Pat::Reference(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_rest(&self) -> Result<&PatRest, ParsingError> {
        if let Pat::Rest(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_slice(&self) -> Result<&PatSlice, ParsingError> {
        if let Pat::Slice(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_struct(&self) -> Result<&PatStruct, ParsingError> {
        if let Pat::Struct(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_tuple(&self) -> Result<&PatTuple, ParsingError> {
        if let Pat::Tuple(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_tuple_struct(&self) -> Result<&PatTupleStruct, ParsingError> {
        if let Pat::TupleStruct(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_type(&self) -> Result<&PatType, ParsingError> {
        if let Pat::Type(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_verbatim(&self) -> Result<&TokenStream, ParsingError> {
        if let Pat::Verbatim(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_wild(&self) -> Result<&PatWild, ParsingError> {
        if let Pat::Wild(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_const_mut(&mut self) -> Result<&mut PatConst, ParsingError> {
        if let Pat::Const(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_ident_mut(&mut self) -> Result<&mut PatIdent, ParsingError> {
        if let Pat::Ident(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_lit_mut(&mut self) -> Result<&mut PatLit, ParsingError> {
        if let Pat::Lit(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_macro_mut(&mut self) -> Result<&mut PatMacro, ParsingError> {
        if let Pat::Macro(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_or_mut(&mut self) -> Result<&mut PatOr, ParsingError> {
        if let Pat::Or(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_paren_mut(&mut self) -> Result<&mut PatParen, ParsingError> {
        if let Pat::Paren(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_path_mut(&mut self) -> Result<&mut PatPath, ParsingError> {
        if let Pat::Path(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_range_mut(&mut self) -> Result<&mut PatRange, ParsingError> {
        if let Pat::Range(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_reference_mut(&mut self) -> Result<&mut PatReference, ParsingError> {
        if let Pat::Reference(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_rest_mut(&mut self) -> Result<&mut PatRest, ParsingError> {
        if let Pat::Rest(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_slice_mut(&mut self) -> Result<&mut PatSlice, ParsingError> {
        if let Pat::Slice(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_struct_mut(&mut self) -> Result<&mut PatStruct, ParsingError> {
        if let Pat::Struct(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_tuple_mut(&mut self) -> Result<&mut PatTuple, ParsingError> {
        if let Pat::Tuple(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_tuple_struct_mut(&mut self) -> Result<&mut PatTupleStruct, ParsingError> {
        if let Pat::TupleStruct(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_type_mut(&mut self) -> Result<&mut PatType, ParsingError> {
        if let Pat::Type(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_verbatim_mut(&mut self) -> Result<&mut TokenStream, ParsingError> {
        if let Pat::Verbatim(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }

    fn as_wild_mut(&mut self) -> Result<&mut PatWild, ParsingError> {
        if let Pat::Wild(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPat(self.clone()))
    }
}

pub trait FnArgExpectable {
    fn as_receiver(&self) -> Result<&Receiver, ParsingError>;
    fn as_typed(&self) -> Result<&PatType, ParsingError>;

    fn as_receiver_mut(&mut self) -> Result<&mut Receiver, ParsingError>;
    fn as_typed_mut(&mut self) -> Result<&mut PatType, ParsingError>;
}

impl FnArgExpectable for FnArg {
    fn as_receiver(&self) -> Result<&Receiver, ParsingError> {
        if let FnArg::Receiver(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedFnArg(self.clone()))
    }

    fn as_typed(&self) -> Result<&PatType, ParsingError> {
        if let FnArg::Typed(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedFnArg(self.clone()))
    }

    fn as_receiver_mut(&mut self) -> Result<&mut Receiver, ParsingError> {
        if let FnArg::Receiver(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedFnArg(self.clone()))
    }

    fn as_typed_mut(&mut self) -> Result<&mut PatType, ParsingError> {
        if let FnArg::Typed(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedFnArg(self.clone()))
    }
}

pub(crate) trait TraitItemExpectable {
    fn as_const(&self) -> Result<&TraitItemConst, ParsingError>;
    fn as_fn(&self) -> Result<&TraitItemFn, ParsingError>;
    fn as_type(&self) -> Result<&TraitItemType, ParsingError>;
    fn as_macro(&self) -> Result<&TraitItemMacro, ParsingError>;
    fn as_verbatim(&self) -> Result<&TokenStream, ParsingError>;

    fn as_const_mut(&mut self) -> Result<&mut TraitItemConst, ParsingError>;
    fn as_fn_mut(&mut self) -> Result<&mut TraitItemFn, ParsingError>;
    fn as_type_mut(&mut self) -> Result<&mut TraitItemType, ParsingError>;
    fn as_macro_mut(&mut self) -> Result<&mut TraitItemMacro, ParsingError>;
    fn as_verbatim_mut(&mut self) -> Result<&mut TokenStream, ParsingError>;
}

impl TraitItemExpectable for TraitItem {
    fn as_const(&self) -> Result<&TraitItemConst, ParsingError> {
        if let TraitItem::Const(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedTraitItem(self.clone()))
    }

    fn as_fn(&self) -> Result<&TraitItemFn, ParsingError> {
        if let TraitItem::Fn(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedTraitItem(self.clone()))
    }

    fn as_type(&self) -> Result<&TraitItemType, ParsingError> {
        if let TraitItem::Type(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedTraitItem(self.clone()))
    }

    fn as_macro(&self) -> Result<&TraitItemMacro, ParsingError> {
        if let TraitItem::Macro(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedTraitItem(self.clone()))
    }

    fn as_verbatim(&self) -> Result<&TokenStream, ParsingError> {
        if let TraitItem::Verbatim(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedTraitItem(self.clone()))
    }

    fn as_const_mut(&mut self) -> Result<&mut TraitItemConst, ParsingError> {
        if let TraitItem::Const(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedTraitItem(self.clone()))
    }

    fn as_fn_mut(&mut self) -> Result<&mut TraitItemFn, ParsingError> {
        if let TraitItem::Fn(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedTraitItem(self.clone()))
    }

    fn as_type_mut(&mut self) -> Result<&mut TraitItemType, ParsingError> {
        if let TraitItem::Type(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedTraitItem(self.clone()))
    }

    fn as_macro_mut(&mut self) -> Result<&mut TraitItemMacro, ParsingError> {
        if let TraitItem::Macro(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedTraitItem(self.clone()))
    }

    fn as_verbatim_mut(&mut self) -> Result<&mut TokenStream, ParsingError> {
        if let TraitItem::Verbatim(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedTraitItem(self.clone()))
    }
}

pub(crate) trait ReturnTypeExpectable {
    fn as_type(&self) -> Result<(&RArrow, &Box<Type>), ParsingError>;
    fn as_type_mut(&mut self) -> Result<(&mut RArrow, &mut Box<Type>), ParsingError>;
}

impl ReturnTypeExpectable for ReturnType {
    fn as_type(&self) -> Result<(&RArrow, &Box<Type>), ParsingError> {
        if let ReturnType::Type(arrow, ty) = self {
            return Ok((arrow, ty));
        }
        Err(ParsingError::UnexpectedReturnType(self.clone()))
    }

    fn as_type_mut(&mut self) -> Result<(&mut RArrow, &mut Box<Type>), ParsingError> {
        if let ReturnType::Type(arrow, ty) = self {
            return Ok((arrow, ty));
        }
        Err(ParsingError::UnexpectedReturnType(self.clone()))
    }
}

pub(crate) trait GenericArgumentExpectable {
    fn as_lifetime(&self) -> Result<&Lifetime, ParsingError>;
    fn as_type(&self) -> Result<&Type, ParsingError>;
    fn as_const(&self) -> Result<&Expr, ParsingError>;
    fn as_assoc_type(&self) -> Result<&AssocType, ParsingError>;
    fn as_assoc_const(&self) -> Result<&AssocConst, ParsingError>;
    fn as_constraint(&self) -> Result<&Constraint, ParsingError>;

    fn as_lifetime_mut(&mut self) -> Result<&mut Lifetime, ParsingError>;
    fn as_type_mut(&mut self) -> Result<&mut Type, ParsingError>;
    fn as_const_mut(&mut self) -> Result<&mut Expr, ParsingError>;
    fn as_assoc_type_mut(&mut self) -> Result<&mut AssocType, ParsingError>;
    fn as_assoc_const_mut(&mut self) -> Result<&mut AssocConst, ParsingError>;
    fn as_constraint_mut(&mut self) -> Result<&mut Constraint, ParsingError>;
}

impl GenericArgumentExpectable for GenericArgument {
    fn as_lifetime(&self) -> Result<&Lifetime, ParsingError> {
        if let GenericArgument::Lifetime(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericArgument(self.clone()))
    }

    fn as_type(&self) -> Result<&Type, ParsingError> {
        if let GenericArgument::Type(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericArgument(self.clone()))
    }

    fn as_const(&self) -> Result<&Expr, ParsingError> {
        if let GenericArgument::Const(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericArgument(self.clone()))
    }

    fn as_assoc_type(&self) -> Result<&AssocType, ParsingError> {
        if let GenericArgument::AssocType(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericArgument(self.clone()))
    }

    fn as_assoc_const(&self) -> Result<&AssocConst, ParsingError> {
        if let GenericArgument::AssocConst(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericArgument(self.clone()))
    }

    fn as_constraint(&self) -> Result<&Constraint, ParsingError> {
        if let GenericArgument::Constraint(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericArgument(self.clone()))
    }

    fn as_lifetime_mut(&mut self) -> Result<&mut Lifetime, ParsingError> {
        if let GenericArgument::Lifetime(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericArgument(self.clone()))
    }

    fn as_type_mut(&mut self) -> Result<&mut Type, ParsingError> {
        if let GenericArgument::Type(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericArgument(self.clone()))
    }

    fn as_const_mut(&mut self) -> Result<&mut Expr, ParsingError> {
        if let GenericArgument::Const(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericArgument(self.clone()))
    }

    fn as_assoc_type_mut(&mut self) -> Result<&mut AssocType, ParsingError> {
        if let GenericArgument::AssocType(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericArgument(self.clone()))
    }

    fn as_assoc_const_mut(&mut self) -> Result<&mut AssocConst, ParsingError> {
        if let GenericArgument::AssocConst(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericArgument(self.clone()))
    }

    fn as_constraint_mut(&mut self) -> Result<&mut Constraint, ParsingError> {
        if let GenericArgument::Constraint(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericArgument(self.clone()))
    }
}

pub(crate) trait PathArgumentsExpectable {
    fn expect_none(&self) -> Result<(), ParsingError>;

    fn as_angle_bracketed(&self) -> Result<&AngleBracketedGenericArguments, ParsingError>;
    fn as_parenthesized(&self) -> Result<&ParenthesizedGenericArguments, ParsingError>;

    fn as_angle_bracketed_mut(
        &mut self,
    ) -> Result<&mut AngleBracketedGenericArguments, ParsingError>;
    fn as_parenthesized_mut(&mut self) -> Result<&mut ParenthesizedGenericArguments, ParsingError>;
}

impl PathArgumentsExpectable for PathArguments {
    fn expect_none(&self) -> Result<(), ParsingError> {
        if let PathArguments::None = self {
            return Ok(());
        }
        Err(ParsingError::UnexpectedPathArguments(self.clone()))
    }

    fn as_angle_bracketed(&self) -> Result<&AngleBracketedGenericArguments, ParsingError> {
        if let PathArguments::AngleBracketed(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPathArguments(self.clone()))
    }

    fn as_parenthesized(&self) -> Result<&ParenthesizedGenericArguments, ParsingError> {
        if let PathArguments::Parenthesized(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPathArguments(self.clone()))
    }

    fn as_angle_bracketed_mut(
        &mut self,
    ) -> Result<&mut AngleBracketedGenericArguments, ParsingError> {
        if let PathArguments::AngleBracketed(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPathArguments(self.clone()))
    }

    fn as_parenthesized_mut(&mut self) -> Result<&mut ParenthesizedGenericArguments, ParsingError> {
        if let PathArguments::Parenthesized(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedPathArguments(self.clone()))
    }
}

pub(crate) trait GenericParamExpectable {
    fn as_lifetime(&self) -> Result<&LifetimeParam, ParsingError>;
    fn as_type(&self) -> Result<&TypeParam, ParsingError>;
    fn as_const(&self) -> Result<&ConstParam, ParsingError>;

    fn as_lifetime_mut(&mut self) -> Result<&mut LifetimeParam, ParsingError>;
    fn as_type_mut(&mut self) -> Result<&mut TypeParam, ParsingError>;
    fn as_const_mut(&mut self) -> Result<&mut ConstParam, ParsingError>;
}

impl GenericParamExpectable for GenericParam {
    fn as_lifetime(&self) -> Result<&LifetimeParam, ParsingError> {
        if let GenericParam::Lifetime(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericParam(self.clone()))
    }

    fn as_type(&self) -> Result<&TypeParam, ParsingError> {
        if let GenericParam::Type(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericParam(self.clone()))
    }

    fn as_const(&self) -> Result<&ConstParam, ParsingError> {
        if let GenericParam::Const(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericParam(self.clone()))
    }

    fn as_lifetime_mut(&mut self) -> Result<&mut LifetimeParam, ParsingError> {
        if let GenericParam::Lifetime(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericParam(self.clone()))
    }

    fn as_type_mut(&mut self) -> Result<&mut TypeParam, ParsingError> {
        if let GenericParam::Type(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericParam(self.clone()))
    }

    fn as_const_mut(&mut self) -> Result<&mut ConstParam, ParsingError> {
        if let GenericParam::Const(inner) = self {
            return Ok(inner);
        }
        Err(ParsingError::UnexpectedGenericParam(self.clone()))
    }
}
