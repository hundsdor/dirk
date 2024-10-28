use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, token::PathSep, Ident, Path, PathArguments, PathSegment, Type, TypePath,
};

macro_rules! mk_type {
    ($ty:ident, $($segments:literal),+) => {
        mk_type!{$ty [] $($segments)+}
    };

    ($ty:ident [$($segments:literal)*] $head:literal $($tail:literal)+) => {
        mk_type!{$ty [$($segments)* $head] $($tail)+}
    };

    ($ty:ident [$($segments:literal)*] $head:literal) => {

pub(crate) fn $ty(generics: PathArguments, span: Span) -> Type {
    let path =  {
        let mut segments: Punctuated<PathSegment, PathSep> = Punctuated::new();

        $(
        let segment =  {
            let ident = Ident::new($segments, span.clone());
            let arguments = PathArguments::None;
            PathSegment { ident, arguments}
        };
        segments.push(segment);
        )*

        let segment =  {
            let ident = Ident::new($head, span);
            let arguments = generics;
            PathSegment { ident, arguments}
        };
        segments.push(segment);

        TypePath { qself: None, path:  Path { leading_colon: None, segments }}
    };
    Type::Path(path)
}
    };
}

mk_type!(type_provider, "dirk", "provides", "Provider");
mk_type!(type_factory_instance, "dirk", "provides", "FactoryInstance");
mk_type!(type_unset, "dirk", "component", "builder", "Unset");
mk_type!(type_set, "dirk", "component", "builder", "Set");
mk_type!(type_rc, "std", "rc", "Rc");
mk_type!(type_refcell, "std", "cell", "RefCell");
mk_type!(type_arc, "std", "sync", "Arc");
mk_type!(type_rwlock, "std", "sync", "RwLock");

macro_rules! mk_path {
    ($name:ident, $($segments:literal),+) => {
        mk_path!{$name [] $($segments)*}
    };

    ($name:ident [$($segments:literal)*] $head:literal $($tail:literal)+) => {
        mk_path!{$name [$($segments)* $head] $($tail)*}
    };

    ($name:ident [$($segments:literal)*] $head:literal) => {
pub(crate) fn $name(generics: PathArguments, span: Span) -> Path {
    let segments = {
        let mut segments: Punctuated<PathSegment, PathSep> = Punctuated::new();

        $(
        let segment =  {
            let ident = Ident::new($segments, span);
            let arguments = PathArguments::None;
            PathSegment { ident, arguments}
        };
        segments.push(segment);
        )*

        let segment =  {
            let ident = Ident::new($head, span);
            let arguments = generics;
            PathSegment { ident, arguments}
        };
        segments.push(segment);

        segments
    };
    Path {leading_colon: None, segments }
}
    };
}

mk_path!(path_allow, "allow");
mk_path!(path_unused_imports, "unused_imports");
mk_path!(path_crate, "crate");

mk_path!(path_provider, "dirk", "provides", "Provider");
mk_path!(
    path_factory_instance_new,
    "dirk",
    "provides",
    "FactoryInstance",
    "new"
);
mk_path!(path_component, "dirk", "component", "Component");
mk_path!(
    path_static_component,
    "dirk",
    "component",
    "StaticComponent"
);
mk_path!(path_builder, "dirk", "component", "builder", "Builder");
mk_path!(
    path_static_builder,
    "dirk",
    "component",
    "builder",
    "StaticBuilder"
);
mk_path!(path_unset, "dirk", "component", "builder", "Unset");
mk_path!(path_set, "dirk", "component", "builder", "Set");
mk_path!(
    path_input_status,
    "dirk",
    "component",
    "builder",
    "InputStatus"
);

mk_path!(path_rc_new, "std", "rc", "Rc", "new");
mk_path!(path_refcell_new, "std", "cell", "RefCell", "new");
mk_path!(path_arc_new, "std", "sync", "Arc", "new");
mk_path!(path_rwlock_new, "std", "sync", "RwLock", "new");
mk_path!(
    path_cloned_instance_factory_new,
    "dirk",
    "component",
    "instance_binds",
    "ClonedInstanceFactory",
    "new"
);
mk_path!(
    path_scoped_instance_factory_new,
    "dirk",
    "component",
    "instance_binds",
    "ScopedInstanceFactory",
    "new"
);

mk_path!(path_self, "Self");
mk_path!(path_small_self, "self");
mk_path!(path_self_new, "Self", "new");
mk_path!(path_self_new_instance, "Self", "new_instance");

mk_path!(path_derive, "derive");
mk_path!(path_clone, "Clone");
