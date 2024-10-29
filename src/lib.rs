//! Dirk is a framework for compile-time dependency injection, focusing on usability and developer experience.
//!
//! # Usage
//!
//! Dependency Injection using dirk relies on two equally important concepts:
//! - Providers (see [`#[provides(...)]`](macro@provides)) specify how instances are created
//!    - static, not wrapped
//!    - singleton, wrapped in `Arc<RwLock<...>>` (shared globally)
//!    - scoped, wrapped in `Rc<RefCell<...>>` (shared inside an individual component)
//! - Components (see [`#[component(...)]`](macro@component)) provide a way to retrieve instances (possibly containing multiple dependencies, specified by so-called bindings)
//!    - bindings provided via a provider
//!    - instance bindings, supplied by the user
//!        - scoped instance, wrapped in `Rc<RefCell<...>>` (shared inside an individual component)
//!        - cloned instance, not wrapped (cloned wehenever it is required)
//!
//! [`#[use_provides(...)]`](macro@use_provides) and [`#[use_component(...)]`](macro@use_component) may be used to import providers and components in other modules.
//!
//! # Examples
//!
//!```no_run
//! # use std::cell::RefCell;
//! # use std::rc::Rc;
//! use dirk::provides;
//!
//! struct UserService {}
//!
//! #[provides(scoped_inject)]
//! impl UserService {
//!     fn new() -> Self { UserService { /* ... */ } }    
//! }
//!
//! struct AuthService {}
//!
//! #[provides(scoped_inject)]
//! impl AuthService {
//!     fn new() -> Self { AuthService { /* ... */ } }    
//! }
//!
//! struct Application {
//!     user_service: Rc<RefCell<UserService>>,
//!     auth_service: Rc<RefCell<AuthService>>
//! }
//!
//! #[provides(static_inject)]
//! impl Application{
//!     fn new(user_service: Rc<RefCell<UserService>>,
//!             auth_service: Rc<RefCell<AuthService>>) -> Self {
//!         Application {user_service, auth_service}
//!     }    
//!  }
//!
//! #[component(
//!     user_service: scoped_bind(UserService),
//!     auth_service: scoped_bind(AuthService),
//!     application: static_bind(Application) [user_service, auth_service]
//! )]
//! trait ApplicationComponent {
//!     fn user_service(&self) -> Rc<RefCell<UserService>>;
//!     fn auth_service(&self) -> Rc<RefCell<AuthService>>;
//!     fn application(&self) -> Application;
//! }
//!
//! use dirk::component;
//! use dirk::component::{Component, StaticComponent, builder::Builder};
//!
//! let component = DirkApplicationComponent::create(); // <- Auto-generated
//! let application = component.application();
//!```
//!
//! ## Generic Components
//! Components are even allowed to be generic, as long as no `where` clause is used.
//!
//!```
//! use dirk::component;
//! use dirk::component::{Component, StaticComponent, builder::Builder};
//!
//! #[component(answer: cloned_instance_bind(T))]
//! trait GenericComponent<T: Clone + 'static> {
//!     fn answer(&self) -> T;
//! }
//!
//! let component = DirkGenericComponent::builder().answer(42).build();
//! assert_eq!(component.answer(), 42);
//!```

#[macro_use(component, provides, use_provides)]
#[allow(unused_imports)]
extern crate dirk_macros;

pub use dirk_macros::component;
pub use dirk_macros::provides;
pub use dirk_macros::use_component;
pub use dirk_macros::use_provides;

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

    use builder::{Builder, UnsetBuilder};

    /**
     * Entry point for querying instances
     *
     * **Do not implement this trait yourself! Use the [`#[component(...)]`](macro@component) macro to generate a type implementing this trait.**
     */
    pub trait Component<B: UnsetBuilder> {
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
    pub trait StaticComponent<T, B: Builder<T> + UnsetBuilder>: Component<B> {
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
        pub trait UnsetBuilder {}

        /**
         * Used internally to create instances
         *
         * **Do not implement this trait yourself!.**
         */
        pub trait Builder<T> {
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
