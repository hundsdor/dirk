use binding::BindingKind;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, abort_call_site, emit_error, proc_macro_error};
use std::fmt::Debug;
use syn::{
    ItemImpl, PathArguments, ReturnType, Type,
    UseGlob,
};

mod expectable;
mod syntax;
mod util;

mod scoped_inject;
mod singleton_inject;
mod static_inject;

mod use_injectable;

mod binding;
mod component;

pub(crate) type Result<T> = std::result::Result<T, StilettoError>;

#[derive(Debug)]
enum StilettoError {
    Parsing(syn::parse::Error),
    UnexpectedToken(Box<dyn ExpectableError>),

    Component(ComponentLogicError),
    Inject(InjectLogicError),
    UseInjectable(UseInjectableLogicError),
}

impl StilettoError {
    fn abort(self) -> ! {
        match self {
            StilettoError::Parsing(e) => abort!(e.span(), e.to_compile_error()),
            StilettoError::UnexpectedToken(t) => abort_call_site!(format!("{t:?}")),
            StilettoError::Component(c) => c.abort(),
            StilettoError::Inject(i) => i.abort(),
            StilettoError::UseInjectable(u) => u.abort(),
        }
    }
}

trait ExpectableError: Debug {}

impl From<syn::parse::Error> for StilettoError {
    fn from(value: syn::Error) -> Self {
        Self::Parsing(value)
    }
}

impl<T: ExpectableError + 'static> From<T> for StilettoError {
    fn from(value: T) -> Self {
        Self::UnexpectedToken(Box::new(value))
    }
}

#[derive(Debug)]
enum ComponentLogicError {
    NotFound(Ident),
    InvalidGenericArgCount(Type),
    EmptyPath(Span),
    TypeMismatch {
        fun_type: ReturnType,
        binding_kind: BindingKind,
    },
    InvalidType(Type),
    InvalidPathArguments(PathArguments),
}

impl From<ComponentLogicError> for StilettoError {
    fn from(value: ComponentLogicError) -> Self {
        Self::Component(value)
    }
}

impl ComponentLogicError {
    fn abort(self) -> ! {
        match self {
            ComponentLogicError::NotFound(binding) => {
                abort!(binding, "Did not find binding {binding}")
            }
            ComponentLogicError::InvalidGenericArgCount(ty) => {
                abort!(ty, "Got invalid number of generic arguments on type {ty}")
            }
            ComponentLogicError::EmptyPath(span) => abort!(span, "Expected non-empty path"),
            ComponentLogicError::TypeMismatch {
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
                abort!(fun_type, "...type returned here (2/2)")
            }
            ComponentLogicError::InvalidType(ty) => abort!(ty, "Found invalid type {ty}"),
            ComponentLogicError::InvalidPathArguments(args) => {
                abort!(args, "Found invalid generic arguments {args}")
            }
        }
    }
}

#[derive(Debug)]
enum InjectLogicError {
    InjectOnTrait(ItemImpl),
    InvalidFunctionCount(ItemImpl),
    EmptyPath,
}

impl From<InjectLogicError> for StilettoError {
    fn from(value: InjectLogicError) -> Self {
        Self::Inject(value)
    }
}

impl InjectLogicError {
    fn abort(self) -> ! {
        match self {
            InjectLogicError::InjectOnTrait(_) => todo!(),
            InjectLogicError::InvalidFunctionCount(_) => todo!(),
            InjectLogicError::EmptyPath => todo!(),
        }
    }
}

#[derive(Debug)]
enum UseInjectableLogicError {
    FoundGlob(UseGlob),
}

impl From<UseInjectableLogicError> for StilettoError {
    fn from(value: UseInjectableLogicError) -> Self {
        Self::UseInjectable(value)
    }
}

impl UseInjectableLogicError {
    fn abort(self) -> ! {
        match self {
            UseInjectableLogicError::FoundGlob(use_glob) => abort!(
                use_glob,
                "#[use_injectable] on wildcard use items is not supported"
            ),
        }
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn static_inject(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = static_inject::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.abort(),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn scoped_inject(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = scoped_inject::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.abort(),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn singleton_inject(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = singleton_inject::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.abort(),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn use_injectable(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = use_injectable::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.abort(),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = component::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.abort(),
    }
}

#[cfg(test)]
mod tests {}
