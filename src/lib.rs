//! Dirk is a framework for compile-time dependency injection.
//!
//! It provids several macros:
//! - [`#[provides(...)]`](macro@provides) annotates an `impl` block containing a single functions that provide an instance of a certain type
//! - [`#[component(...)]`](macro@component) declares a component that may be used to instantiate types
//! - [`#[use_injectable(...)]`](macro@use_injectable) facilitates injecting or querying types provided in a different module
//! - [`#[use_component]`](macro@use_component) facilitates using components defined in a different module
//!

#[macro_use(component, provides, use_injectable)]
#[allow(unused_imports)]
extern crate dirk_macros;

pub use dirk_macros::component;
pub use dirk_macros::provides;
pub use dirk_macros::use_component;
pub use dirk_macros::use_injectable;

pub mod provides {
    //! Contains data types used by the `#[provides]` macro

    use std::ops::Deref;

    use once_cell::sync::Lazy;

    /**
     * A trait used by the `#[provides]` macro
     *
     * The `#[provides]` macro generates types implementing this trait, by convention named `Factory`
     */
    pub trait Provider<T> {
        /**
         * Returns the thing that is being provided
         */
        fn get(&self) -> T;
    }

    /**
     * A type used by the `#[provides(singleton_inject)]` macro
     *
     * Stores an instance of a Factory
     */
    pub struct FactoryInstance<T>(Lazy<T>);

    impl<T> FactoryInstance<T> {
        #[allow(missing_docs)]
        pub const fn new(constructor: fn() -> T) -> Self {
            Self(Lazy::new(constructor))
        }
    }

    impl<T> Deref for FactoryInstance<T> {
        type Target = Lazy<T>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

pub mod component {
    //! Contains data types used by the `#[component(...)]` macro

    use builder::{Builder, StaticBuilder};

    /**
     * Entry point for querying instances
     *
     * **Do not implement this trait yourself! Use the [`#[component(...)]`](macro@component) macro to generate a type implementing this trait.**
     */
    pub trait DirkComponent<B: Builder> {
        /**
         * Returns a type-safe builder that may be used to create instances
         */
        fn builder() -> B;
    }

    /**
     * Entry point for querying instances, if no user input is required, i.e., no `*_instance_bind`s are defined
     *
     * **Do not implement this trait yourself! Use the [`#[component(...)]`](macro@component) macro to generate a type implementing this trait.**
     */
    pub trait DirkStaticComponent<T, B: StaticBuilder<T> + Builder>: DirkComponent<B> {
        /**
        Creates an instance of T, bypassing the builder pattern
        */
        #[must_use = "Instances created via dependency injection are supposed to be used somewhere"]
        fn create() -> T {
            Self::builder().build()
        }
    }

    pub mod builder {
        //! Contains data types used in the type-safe builder pattern created by the `#[component(...)]` macro

        /**
         * Used internally to create instances
         *
         * **Do not implement this trait yourself!.**
         */
        pub trait Builder {}

        /**
         * Used internally to create instances
         *
         * **Do not implement this trait yourself!.**
         */
        pub trait StaticBuilder<T> {
            /**
             * Build an instance of a component
             */
            #[must_use = "Instances created via dependency injection are supposed to be used somewhere"]
            fn build(self) -> T;
        }

        /**
         * Used in a type-safe builder pattern to indicate that some parameter has not yet been set
         */
        pub struct Unset;

        /**
         * Used in a type-safe builder pattern to contain a parameter that has been set
         */
        pub struct Set<T>(pub T);

        /**
         * Used in a type-safe builder pattern generated by the `#[component(...)]` macro
         */
        pub trait InputStatus {}

        impl InputStatus for Unset {}
        impl<T> InputStatus for Set<T> {}
    }

    pub mod instance_binds {
        //! Contains data types used by `..._instance_bind(...)` bindings that may be used in a `#[component(...)]` macro

        use crate::provides::Provider;

        /**
         * A type used by `cloned_instance_bind(...)`
         *
         * Whatever is being provided, is cloned every time it is queried
         */
        pub struct ClonedInstanceFactory<T: Clone> {
            inner: T,
        }

        impl<T: Clone> ClonedInstanceFactory<T> {
            #[allow(missing_docs)]
            pub fn new(inner: T) -> Self {
                Self { inner }
            }
        }

        impl<T: Clone> Provider<T> for ClonedInstanceFactory<T> {
            fn get(&self) -> T {
                self.inner.clone()
            }
        }

        /**
         * A type used by `scoped_instance_binds(...)`
         * Wraps whatever is being provided in a `Rc<RefCell<...>>`
         */
        pub struct ScopedInstanceFactory<T> {
            inner: std::rc::Rc<std::cell::RefCell<T>>,
        }

        impl<T> ScopedInstanceFactory<T> {
            #[allow(missing_docs)]
            pub fn new(inner: T) -> Self {
                Self {
                    inner: std::rc::Rc::new(std::cell::RefCell::new(inner)),
                }
            }
        }

        impl<T> Provider<std::rc::Rc<std::cell::RefCell<T>>> for ScopedInstanceFactory<T> {
            fn get(&self) -> std::rc::Rc<std::cell::RefCell<T>> {
                self.inner.clone()
            }
        }
    }
}
