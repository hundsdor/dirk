

use proc_macro_error::{abort, emit_error};
use syn::{Ident, Type};

use crate::{
    errors::ExpectableError,
    errors::{InfallibleError, SyntaxError},
    stringify::StringifyError,
};

use super::binding::BindingKind;

pub(crate) type ComponentResult<T> = std::result::Result<T, ComponentError>;

#[derive(Debug)]
pub(crate) enum ComponentError {
    Infallible(InfallibleError<ComponentSyntaxError>),
    Stringify(StringifyError),
    Logic(ComponentLogicAbort),
}

impl_abort!(ComponentError);
impl_from_infallible_error!(ComponentError, ComponentSyntaxError);

#[derive(Debug)]
pub(crate) enum ComponentSyntaxError {
    FailedToParseInput(syn::Error),
    ExpectedTrait(syn::Error),
}

impl SyntaxError for ComponentSyntaxError {
    fn abort(self) -> ! {
        match self {
            Self::ExpectedTrait(e) => {
                abort!(
                    e.span(),
                    e.to_string();
                    help = "`#[component(...)]` is expected to be placed on a trait"
                )
            }
            Self::FailedToParseInput(e) => abort!(e.span(), e.to_string()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum ComponentLogicAbort {
    NotFound(Ident),
    TypeMismatch {
        fun_type: Type,
        binding_kind: BindingKind,
    },
    InvalidType(Type),
}

impl From<ComponentLogicAbort> for ComponentError {
    fn from(value: ComponentLogicAbort) -> Self {
        Self::Logic(value)
    }
}

impl ComponentLogicAbort {
    fn abort(self) -> ! {
        match self {
            ComponentLogicAbort::NotFound(binding) => {
                abort!(binding, "Binding is not defined")
            }
            ComponentLogicAbort::TypeMismatch {
                fun_type,
                binding_kind,
            } => {
                let (hint, binding_type) = match binding_kind {
                    BindingKind::Singleton(ty) => {
                        let hint =
                        "singleton bindings wrap their type T into a std::sync::Arc<std::sync::RwLock<T>>";
                        (hint, ty)
                    }
                    BindingKind::Scoped(ty) => {
                        let hint =
                        "scoped bindings wrap their type T into a std::rc::Rc<std::cell::RefCell<T>>";
                        (hint, ty)
                    }
                    BindingKind::Static(ty) => {
                        let hint = "static bindings do not wrap their type T and just return a T";
                        (hint, ty)
                    }
                };
                emit_error!(binding_type, "Type of binding does not match... (1/2)"; hint=hint);
                abort!(fun_type, "...type specified here (2/2)")
            }
            ComponentLogicAbort::InvalidType(ty) => abort!(ty, "Found invalid type"),
        }
    }
}

pub(crate) enum ComponentLogicEmit {
    NotFound(Ident),
}

impl ComponentLogicEmit {
    pub(crate) fn emit(self) {
        match self {
            ComponentLogicEmit::NotFound(binding) => {
                emit_error!(binding, "Binding is not defined")
            }
        }
    }
}
