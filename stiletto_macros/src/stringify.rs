use proc_macro_error::abort;
use syn::{spanned::Spanned, GenericArgument, Type, TypeParamBound};

#[derive(Debug)]
pub(crate) enum StringifyError {
    TypeInferenceNotSupported(Type),
    UnsupportedGenericArg(GenericArgument),
    UnsupportedType(Type),
    UnsupportedTypeParamBound(TypeParamBound),
}

impl StringifyError {
    pub(crate) fn abort(self) -> ! {
        match self {
            Self::TypeInferenceNotSupported(ty) => {
                abort!(
                    ty.span(),
                    "Type inference is not supported in injectable types"
                )
            }
            Self::UnsupportedGenericArg(generic_argument) => {
                abort!(
                    generic_argument.span(),
                    "Found unsupported kind of generic argument"
                )
            }
            Self::UnsupportedType(ty) => {
                abort!(ty.span(), "Found unsupported kind of type")
            }
            Self::UnsupportedTypeParamBound(ty_param_bound) => {
                abort!(
                    ty_param_bound.span(),
                    "Found unsupported kind of type parameter bound"
                )
            }
        }
    }
}

pub(crate) trait Stringify {
    fn stringify(&self) -> Result<String, StringifyError>;
}

impl Stringify for GenericArgument {
    fn stringify(&self) -> Result<String, StringifyError> {
        match self {
            GenericArgument::Type(ty) => Ok(ty.stringify()?),
            GenericArgument::Lifetime(_lt) => Ok(String::new()),
            GenericArgument::Const(_expr) => todo!(),
            GenericArgument::AssocType(_assoc_type) => {
                Err(StringifyError::UnsupportedGenericArg(self.clone()))
            }
            GenericArgument::AssocConst(_assoc_const) => {
                Err(StringifyError::UnsupportedGenericArg(self.clone()))
            }
            GenericArgument::Constraint(_constraint) => {
                Err(StringifyError::UnsupportedGenericArg(self.clone()))
            }
            _ => todo!(),
        }
    }
}

impl Stringify for Type {
    fn stringify(&self) -> Result<String, StringifyError> {
        match self {
            Type::Array(type_array) => Ok(format!("_a_{}_a_", type_array.elem.stringify()?)),
            Type::BareFn(_type_bare_fn) => Err(StringifyError::UnsupportedType(self.clone())),
            Type::Group(type_group) => Ok(format!("_g_{}_", type_group.elem.stringify()?)),
            Type::ImplTrait(_type_impl_trait) => Err(StringifyError::UnsupportedType(self.clone())),
            Type::Infer(_type_infer) => {
                Err(StringifyError::TypeInferenceNotSupported(self.clone()))
            }
            Type::Paren(type_paren) => Ok(type_paren.elem.stringify()?),
            Type::Reference(type_reference) => {
                Ok(format!("_r_{}", type_reference.elem.stringify()?))
            }
            Type::Ptr(type_ptr) => Ok(format!("_p_{}", type_ptr.elem.stringify()?)),
            Type::Slice(type_slice) => Ok(format!("_s_{}", type_slice.elem.stringify()?)),
            Type::TraitObject(type_trait_object) => {
                let mut acc = String::new();
                for bound in &type_trait_object.bounds {
                    acc += "_b_";
                    acc += &bound.stringify()?;
                }
                Ok(acc)
            }
            Type::Tuple(type_tuple) => {
                let mut acc = String::new();
                for ty in &type_tuple.elems {
                    acc += "_t_";
                    acc += &ty.stringify()?;
                }
                acc += "_t_";
                Ok(acc)
            }
            Type::Macro(_) => Err(StringifyError::UnsupportedType(self.clone())),
            Type::Never(_) => Err(StringifyError::UnsupportedType(self.clone())),
            Type::Verbatim(_) => Err(StringifyError::UnsupportedType(self.clone())),
            Type::Path(type_path) => {
                if let Some(_qself) = &type_path.qself {
                    Err(StringifyError::UnsupportedType(self.clone()))
                } else {
                    let mut acc = String::new();
                    for segment in &(type_path.path).segments {
                        acc += "_";
                        acc += &format!("{}", segment.ident);
                        match &segment.arguments {
                            syn::PathArguments::None => {}
                            syn::PathArguments::AngleBracketed(angle_bracketed) => {
                                for arg in &angle_bracketed.args {
                                    acc += &arg.stringify()?;
                                }
                            }
                            syn::PathArguments::Parenthesized(_) => {
                                return Err(StringifyError::UnsupportedType(self.clone()));
                            }
                        }
                    }
                    Ok(acc)
                }
            }
            _ => todo!(),
        }
    }
}

impl Stringify for TypeParamBound {
    fn stringify(&self) -> Result<String, StringifyError> {
        match self {
            TypeParamBound::Trait(trait_bound) => {
                let mut acc = String::new();
                for segment in &trait_bound.path.segments {
                    acc += "_";
                    acc += &format!("{}", segment.ident).to_uppercase();
                }
                Ok(acc)
            }
            TypeParamBound::Lifetime(_lifetime) => Ok(String::new()),
            TypeParamBound::Verbatim(_) => {
                Err(StringifyError::UnsupportedTypeParamBound(self.clone()))
            }
            _ => todo!(),
        }
    }
}
