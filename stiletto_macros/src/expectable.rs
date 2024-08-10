use proc_macro2::TokenStream;
use syn::{
    FnArg, Pat, PatConst, PatIdent, PatLit, PatMacro, PatOr, PatParen, PatPath, PatRange,
    PatReference, PatRest, PatSlice, PatStruct, PatTuple, PatTupleStruct, PatType, PatWild,
    Receiver, Type, TypeArray, TypeBareFn, TypeGroup, TypeImplTrait, TypeInfer, TypeMacro,
    TypeNever, TypeParen, TypePath, TypePtr, TypeReference, TypeSlice, TypeTraitObject, TypeTuple,
};

pub trait TypeExpectable {
    fn as_array(&self) -> Option<&TypeArray>;
    fn as_bare_fn(&self) -> Option<&TypeBareFn>;
    fn as_group(&self) -> Option<&TypeGroup>;
    fn as_impl_trait(&self) -> Option<&TypeImplTrait>;
    fn as_infer(&self) -> Option<&TypeInfer>;
    fn as_macro(&self) -> Option<&TypeMacro>;
    fn as_never(&self) -> Option<&TypeNever>;
    fn as_paren(&self) -> Option<&TypeParen>;
    fn as_path(&self) -> Option<&TypePath>;
    fn as_ptr(&self) -> Option<&TypePtr>;
    fn as_reference(&self) -> Option<&TypeReference>;
    fn as_slice(&self) -> Option<&TypeSlice>;
    fn as_trait_object(&self) -> Option<&TypeTraitObject>;
    fn as_tuple(&self) -> Option<&TypeTuple>;
    fn as_verbatim(&self) -> Option<&TokenStream>;

    fn unwrap_array(&self) -> &TypeArray {
        self.as_array().unwrap()
    }
    fn unwrap_bare_fn(&self) -> &TypeBareFn {
        self.as_bare_fn().unwrap()
    }
    fn unwrap_group(&self) -> &TypeGroup {
        self.as_group().unwrap()
    }
    fn unwrap_impl_trait(&self) -> &TypeImplTrait {
        self.as_impl_trait().unwrap()
    }
    fn unwrap_infer(&self) -> &TypeInfer {
        self.as_infer().unwrap()
    }
    fn unwrap_macro(&self) -> &TypeMacro {
        self.as_macro().unwrap()
    }
    fn unwrap_never(&self) -> &TypeNever {
        self.as_never().unwrap()
    }
    fn unwrap_paren(&self) -> &TypeParen {
        self.as_paren().unwrap()
    }
    fn unwrap_path(&self) -> &TypePath {
        self.as_path().unwrap()
    }
    fn unwrap_ptr(&self) -> &TypePtr {
        self.as_ptr().unwrap()
    }
    fn unwrap_reference(&self) -> &TypeReference {
        self.as_reference().unwrap()
    }
    fn unwrap_slice(&self) -> &TypeSlice {
        self.as_slice().unwrap()
    }
    fn unwrap_trait_object(&self) -> &TypeTraitObject {
        self.as_trait_object().unwrap()
    }
    fn unwrap_tuple(&self) -> &TypeTuple {
        self.as_tuple().unwrap()
    }
    fn unwrap_verbatim(&self) -> &TokenStream {
        self.as_verbatim().unwrap()
    }

    fn as_array_mut(&mut self) -> Option<&mut TypeArray>;
    fn as_bare_fn_mut(&mut self) -> Option<&mut TypeBareFn>;
    fn as_group_mut(&mut self) -> Option<&mut TypeGroup>;
    fn as_impl_trait_mut(&mut self) -> Option<&mut TypeImplTrait>;
    fn as_infer_mut(&mut self) -> Option<&mut TypeInfer>;
    fn as_macro_mut(&mut self) -> Option<&mut TypeMacro>;
    fn as_never_mut(&mut self) -> Option<&mut TypeNever>;
    fn as_paren_mut(&mut self) -> Option<&mut TypeParen>;
    fn as_path_mut(&mut self) -> Option<&mut TypePath>;
    fn as_ptr_mut(&mut self) -> Option<&mut TypePtr>;
    fn as_reference_mut(&mut self) -> Option<&mut TypeReference>;
    fn as_slice_mut(&mut self) -> Option<&mut TypeSlice>;
    fn as_trait_object_mut(&mut self) -> Option<&mut TypeTraitObject>;
    fn as_tuple_mut(&mut self) -> Option<&mut TypeTuple>;
    fn as_verbatim_mut(&mut self) -> Option<&mut TokenStream>;

    fn unwrap_array_mut(&mut self) -> &mut TypeArray {
        self.as_array_mut().unwrap()
    }
    fn unwrap_bare_fn_mut(&mut self) -> &mut TypeBareFn {
        self.as_bare_fn_mut().unwrap()
    }
    fn unwrap_group_mut(&mut self) -> &mut TypeGroup {
        self.as_group_mut().unwrap()
    }
    fn unwrap_impl_trait_mut(&mut self) -> &mut TypeImplTrait {
        self.as_impl_trait_mut().unwrap()
    }
    fn unwrap_infer_mut(&mut self) -> &mut TypeInfer {
        self.as_infer_mut().unwrap()
    }
    fn unwrap_macro_mut(&mut self) -> &mut TypeMacro {
        self.as_macro_mut().unwrap()
    }
    fn unwrap_never_mut(&mut self) -> &mut TypeNever {
        self.as_never_mut().unwrap()
    }
    fn unwrap_paren_mut(&mut self) -> &mut TypeParen {
        self.as_paren_mut().unwrap()
    }
    fn unwrap_path_mut(&mut self) -> &mut TypePath {
        self.as_path_mut().unwrap()
    }
    fn unwrap_ptr_mut(&mut self) -> &mut TypePtr {
        self.as_ptr_mut().unwrap()
    }
    fn unwrap_reference_mut(&mut self) -> &mut TypeReference {
        self.as_reference_mut().unwrap()
    }
    fn unwrap_slice_mut(&mut self) -> &mut TypeSlice {
        self.as_slice_mut().unwrap()
    }
    fn unwrap_trait_object_mut(&mut self) -> &mut TypeTraitObject {
        self.as_trait_object_mut().unwrap()
    }
    fn unwrap_tuple_mut(&mut self) -> &mut TypeTuple {
        self.as_tuple_mut().unwrap()
    }
    fn unwrap_verbatim_mut(&mut self) -> &mut TokenStream {
        self.as_verbatim_mut().unwrap()
    }
}

impl TypeExpectable for Type {
    fn as_array(&self) -> Option<&TypeArray> {
        if let Type::Array(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_bare_fn(&self) -> Option<&TypeBareFn> {
        if let Type::BareFn(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_group(&self) -> Option<&TypeGroup> {
        if let Type::Group(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_impl_trait(&self) -> Option<&TypeImplTrait> {
        if let Type::ImplTrait(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_infer(&self) -> Option<&TypeInfer> {
        if let Type::Infer(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_macro(&self) -> Option<&TypeMacro> {
        if let Type::Macro(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_never(&self) -> Option<&TypeNever> {
        if let Type::Never(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_paren(&self) -> Option<&TypeParen> {
        if let Type::Paren(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_path(&self) -> Option<&TypePath> {
        if let Type::Path(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_ptr(&self) -> Option<&TypePtr> {
        if let Type::Ptr(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_reference(&self) -> Option<&TypeReference> {
        if let Type::Reference(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_slice(&self) -> Option<&TypeSlice> {
        if let Type::Slice(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_trait_object(&self) -> Option<&TypeTraitObject> {
        if let Type::TraitObject(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_tuple(&self) -> Option<&TypeTuple> {
        if let Type::Tuple(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_verbatim(&self) -> Option<&TokenStream> {
        if let Type::Verbatim(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_array_mut(&mut self) -> Option<&mut TypeArray> {
        if let Type::Array(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_bare_fn_mut(&mut self) -> Option<&mut TypeBareFn> {
        if let Type::BareFn(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_group_mut(&mut self) -> Option<&mut TypeGroup> {
        if let Type::Group(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_impl_trait_mut(&mut self) -> Option<&mut TypeImplTrait> {
        if let Type::ImplTrait(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_infer_mut(&mut self) -> Option<&mut TypeInfer> {
        if let Type::Infer(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_macro_mut(&mut self) -> Option<&mut TypeMacro> {
        if let Type::Macro(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_never_mut(&mut self) -> Option<&mut TypeNever> {
        if let Type::Never(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_paren_mut(&mut self) -> Option<&mut TypeParen> {
        if let Type::Paren(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_path_mut(&mut self) -> Option<&mut TypePath> {
        if let Type::Path(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_ptr_mut(&mut self) -> Option<&mut TypePtr> {
        if let Type::Ptr(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_reference_mut(&mut self) -> Option<&mut TypeReference> {
        if let Type::Reference(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_slice_mut(&mut self) -> Option<&mut TypeSlice> {
        if let Type::Slice(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_trait_object_mut(&mut self) -> Option<&mut TypeTraitObject> {
        if let Type::TraitObject(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_tuple_mut(&mut self) -> Option<&mut TypeTuple> {
        if let Type::Tuple(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_verbatim_mut(&mut self) -> Option<&mut TokenStream> {
        if let Type::Verbatim(inner) = self {
            return Some(inner);
        }
        None
    }
}

pub trait PatExpectable {
    fn as_const(&self) -> Option<&PatConst>;
    fn as_ident(&self) -> Option<&PatIdent>;
    fn as_lit(&self) -> Option<&PatLit>;
    fn as_macro(&self) -> Option<&PatMacro>;
    fn as_or(&self) -> Option<&PatOr>;
    fn as_paren(&self) -> Option<&PatParen>;
    fn as_path(&self) -> Option<&PatPath>;
    fn as_range(&self) -> Option<&PatRange>;
    fn as_reference(&self) -> Option<&PatReference>;
    fn as_rest(&self) -> Option<&PatRest>;
    fn as_slice(&self) -> Option<&PatSlice>;
    fn as_struct(&self) -> Option<&PatStruct>;
    fn as_tuple(&self) -> Option<&PatTuple>;
    fn as_tuple_struct(&self) -> Option<&PatTupleStruct>;
    fn as_type(&self) -> Option<&PatType>;
    fn as_verbatim(&self) -> Option<&TokenStream>;
    fn as_wild(&self) -> Option<&PatWild>;

    fn unwrap_const(&self) -> &PatConst {
        self.as_const().unwrap()
    }
    fn unwrap_ident(&self) -> &PatIdent {
        self.as_ident().unwrap()
    }
    fn unwrap_lit(&self) -> &PatLit {
        self.as_lit().unwrap()
    }
    fn unwrap_macro(&self) -> &PatMacro {
        self.as_macro().unwrap()
    }
    fn unwrap_or(&self) -> &PatOr {
        self.as_or().unwrap()
    }
    fn unwrap_paren(&self) -> &PatParen {
        self.as_paren().unwrap()
    }
    fn unwrap_path(&self) -> &PatPath {
        self.as_path().unwrap()
    }
    fn unwrap_range(&self) -> &PatRange {
        self.as_range().unwrap()
    }
    fn unwrap_reference(&self) -> &PatReference {
        self.as_reference().unwrap()
    }
    fn unwrap_rest(&self) -> &PatRest {
        self.as_rest().unwrap()
    }
    fn unwrap_slice(&self) -> &PatSlice {
        self.as_slice().unwrap()
    }
    fn unwrap_struct(&self) -> &PatStruct {
        self.as_struct().unwrap()
    }
    fn unwrap_tuple(&self) -> &PatTuple {
        self.as_tuple().unwrap()
    }
    fn unwrap_tuple_struct(&self) -> &PatTupleStruct {
        self.as_tuple_struct().unwrap()
    }
    fn unwrap_type(&self) -> &PatType {
        self.as_type().unwrap()
    }
    fn unwrap_verbatim(&self) -> &TokenStream {
        self.as_verbatim().unwrap()
    }
    fn unwrap_wild(&self) -> &PatWild {
        self.as_wild().unwrap()
    }

    fn as_const_mut(&mut self) -> Option<&mut PatConst>;
    fn as_ident_mut(&mut self) -> Option<&mut PatIdent>;
    fn as_lit_mut(&mut self) -> Option<&mut PatLit>;
    fn as_macro_mut(&mut self) -> Option<&mut PatMacro>;
    fn as_or_mut(&mut self) -> Option<&mut PatOr>;
    fn as_paren_mut(&mut self) -> Option<&mut PatParen>;
    fn as_path_mut(&mut self) -> Option<&mut PatPath>;
    fn as_range_mut(&mut self) -> Option<&mut PatRange>;
    fn as_reference_mut(&mut self) -> Option<&mut PatReference>;
    fn as_rest_mut(&mut self) -> Option<&mut PatRest>;
    fn as_slice_mut(&mut self) -> Option<&mut PatSlice>;
    fn as_struct_mut(&mut self) -> Option<&mut PatStruct>;
    fn as_tuple_mut(&mut self) -> Option<&mut PatTuple>;
    fn as_tuple_struct_mut(&mut self) -> Option<&mut PatTupleStruct>;
    fn as_type_mut(&mut self) -> Option<&mut PatType>;
    fn as_verbatim_mut(&mut self) -> Option<&mut TokenStream>;
    fn as_wild_mut(&mut self) -> Option<&mut PatWild>;

    fn unwrap_const_mut(&mut self) -> &mut PatConst {
        self.as_const_mut().unwrap()
    }
    fn unwrap_ident_mut(&mut self) -> &mut PatIdent {
        self.as_ident_mut().unwrap()
    }
    fn unwrap_lit_mut(&mut self) -> &mut PatLit {
        self.as_lit_mut().unwrap()
    }
    fn unwrap_macro_mut(&mut self) -> &mut PatMacro {
        self.as_macro_mut().unwrap()
    }
    fn unwrap_or_mut(&mut self) -> &mut PatOr {
        self.as_or_mut().unwrap()
    }
    fn unwrap_paren_mut(&mut self) -> &mut PatParen {
        self.as_paren_mut().unwrap()
    }
    fn unwrap_path_mut(&mut self) -> &mut PatPath {
        self.as_path_mut().unwrap()
    }
    fn unwrap_range_mut(&mut self) -> &mut PatRange {
        self.as_range_mut().unwrap()
    }
    fn unwrap_reference_mut(&mut self) -> &mut PatReference {
        self.as_reference_mut().unwrap()
    }
    fn unwrap_rest_mut(&mut self) -> &mut PatRest {
        self.as_rest_mut().unwrap()
    }
    fn unwrap_slice_mut(&mut self) -> &mut PatSlice {
        self.as_slice_mut().unwrap()
    }
    fn unwrap_struct_mut(&mut self) -> &mut PatStruct {
        self.as_struct_mut().unwrap()
    }
    fn unwrap_tuple_mut(&mut self) -> &mut PatTuple {
        self.as_tuple_mut().unwrap()
    }
    fn unwrap_tuple_struct_mut(&mut self) -> &mut PatTupleStruct {
        self.as_tuple_struct_mut().unwrap()
    }
    fn unwrap_type_mut(&mut self) -> &mut PatType {
        self.as_type_mut().unwrap()
    }
    fn unwrap_verbatim_mut(&mut self) -> &mut TokenStream {
        self.as_verbatim_mut().unwrap()
    }
    fn unwrap_wild_mut(&mut self) -> &mut PatWild {
        self.as_wild_mut().unwrap()
    }
}

impl PatExpectable for Pat {
    fn as_const(&self) -> Option<&PatConst> {
        if let Pat::Const(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_ident(&self) -> Option<&PatIdent> {
        if let Pat::Ident(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_lit(&self) -> Option<&PatLit> {
        if let Pat::Lit(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_macro(&self) -> Option<&PatMacro> {
        if let Pat::Macro(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_or(&self) -> Option<&PatOr> {
        if let Pat::Or(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_paren(&self) -> Option<&PatParen> {
        if let Pat::Paren(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_path(&self) -> Option<&PatPath> {
        if let Pat::Path(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_range(&self) -> Option<&PatRange> {
        if let Pat::Range(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_reference(&self) -> Option<&PatReference> {
        if let Pat::Reference(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_rest(&self) -> Option<&PatRest> {
        if let Pat::Rest(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_slice(&self) -> Option<&PatSlice> {
        if let Pat::Slice(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_struct(&self) -> Option<&PatStruct> {
        if let Pat::Struct(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_tuple(&self) -> Option<&PatTuple> {
        if let Pat::Tuple(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_tuple_struct(&self) -> Option<&PatTupleStruct> {
        if let Pat::TupleStruct(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_type(&self) -> Option<&PatType> {
        if let Pat::Type(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_verbatim(&self) -> Option<&TokenStream> {
        if let Pat::Verbatim(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_wild(&self) -> Option<&PatWild> {
        if let Pat::Wild(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_const_mut(&mut self) -> Option<&mut PatConst> {
        if let Pat::Const(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_ident_mut(&mut self) -> Option<&mut PatIdent> {
        if let Pat::Ident(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_lit_mut(&mut self) -> Option<&mut PatLit> {
        if let Pat::Lit(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_macro_mut(&mut self) -> Option<&mut PatMacro> {
        if let Pat::Macro(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_or_mut(&mut self) -> Option<&mut PatOr> {
        if let Pat::Or(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_paren_mut(&mut self) -> Option<&mut PatParen> {
        if let Pat::Paren(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_path_mut(&mut self) -> Option<&mut PatPath> {
        if let Pat::Path(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_range_mut(&mut self) -> Option<&mut PatRange> {
        if let Pat::Range(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_reference_mut(&mut self) -> Option<&mut PatReference> {
        if let Pat::Reference(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_rest_mut(&mut self) -> Option<&mut PatRest> {
        if let Pat::Rest(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_slice_mut(&mut self) -> Option<&mut PatSlice> {
        if let Pat::Slice(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_struct_mut(&mut self) -> Option<&mut PatStruct> {
        if let Pat::Struct(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_tuple_mut(&mut self) -> Option<&mut PatTuple> {
        if let Pat::Tuple(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_tuple_struct_mut(&mut self) -> Option<&mut PatTupleStruct> {
        if let Pat::TupleStruct(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_type_mut(&mut self) -> Option<&mut PatType> {
        if let Pat::Type(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_verbatim_mut(&mut self) -> Option<&mut TokenStream> {
        if let Pat::Verbatim(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_wild_mut(&mut self) -> Option<&mut PatWild> {
        if let Pat::Wild(inner) = self {
            return Some(inner);
        }
        None
    }
}

pub trait FnArgExpectable {
    fn as_receiver(&self) -> Option<&Receiver>;
    fn as_typed(&self) -> Option<&PatType>;

    fn unwrap_receiver(&self) -> &Receiver {
        self.as_receiver().unwrap()
    }
    fn unwrap_typed(&self) -> &PatType {
        self.as_typed().unwrap()
    }

    fn as_receiver_mut(&mut self) -> Option<&mut Receiver>;
    fn as_typed_mut(&mut self) -> Option<&mut PatType>;

    fn unwrap_receiver_mut(&mut self) -> &mut Receiver {
        self.as_receiver_mut().unwrap()
    }
    fn unwrap_typed_mut(&mut self) -> &mut PatType {
        self.as_typed_mut().unwrap()
    }
}

impl FnArgExpectable for FnArg {
    fn as_receiver(&self) -> Option<&Receiver> {
        if let FnArg::Receiver(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_typed(&self) -> Option<&PatType> {
        if let FnArg::Typed(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_receiver_mut(&mut self) -> Option<&mut Receiver> {
        if let FnArg::Receiver(inner) = self {
            return Some(inner);
        }
        None
    }

    fn as_typed_mut(&mut self) -> Option<&mut PatType> {
        if let FnArg::Typed(inner) = self {
            return Some(inner);
        }
        None
    }
}
