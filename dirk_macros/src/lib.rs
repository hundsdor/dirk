//! Macros for compile-time dependency injection

use component::{error::ComponentError, processor::ComponentMacroData};
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[macro_use]
mod errors;

mod expectable;
mod syntax;
mod util;

mod component;
mod provides;
mod use_component;
mod use_provides;

pub(crate) const FACTORY_PREFIX_SINGLETON: &str = "SingletonFactory";
pub(crate) const FACTORY_PREFIX_SCOPED: &str = "ScopedFactory";
pub(crate) const FACTORY_PREFIX_STATIC: &str = "StaticFactory";

/// Annotates an `impl` block containing a function that provides an instance of a certain type
///
/// # Static inject (default)
/// `#[provides]` or `#[provides(static_inject)]` on an `impl` of type `T` provides instances of type `T`, without any specialities.
///
///```
/// #[provides]
/// impl ProvidedStatic {
///     pub fn new(inner: usize) -> Self {
///         Self {
///            inner    
///        }
///     }
/// }
/// #
/// # use dirk::provides::Provider;
/// # use dirk::provides;
/// #
/// # struct ProvidedStatic {
/// #     inner: usize   
/// # }
/// #
/// # pub struct InnerProvider { }
/// # impl dirk::provides::Provider<usize> for InnerProvider {
/// #     fn get(&self) -> usize {
/// #         return 42;
/// #     }
/// # }
/// #
/// # let inner_provider = std::rc::Rc::new(InnerProvider {});
/// # let factory = StaticFactoryProvidedStatic::create(inner_provider.clone());
/// # let provided = factory.get();
/// # assert_eq!(provided.inner, 42);
///```
///
/// # Scoped inject
/// `#[provides(scoped_inject)]` on an `impl` of type `T` provides instances of type `Rc<RefCell<T>>`.
///
/// The provided instance will be a reference-counted pointer ([`Rc`](std::rc::Rc)) that is shared in the outer scope, i.e., pointers provided by an individual scoped binding will point to the same instance.
///
///```
/// #[provides(scoped_inject)]
/// impl ProvidedScoped {
///     pub fn new(inner: usize) -> Self {
///         Self {
///            inner    
///        }
///     }
/// }
/// #
/// # use dirk::provides::Provider;
/// # use dirk::provides;
/// #
/// # struct ProvidedScoped {
/// #     inner: usize   
/// # }
/// #
/// # pub struct InnerProvider { }
/// # impl dirk::provides::Provider<usize> for InnerProvider {
/// #     fn get(&self) -> usize {
/// #         return 42;
/// #     }
/// # }
/// #
/// # let inner_provider = std::rc::Rc::new(InnerProvider {});
/// # let factory = ScopedFactoryProvidedScoped::create(inner_provider.clone());
/// # let provided = factory.get();
/// # assert_eq!(provided.borrow().inner, 42);
///```
///
/// # Singleton inject
/// `#[provides(singleton_inject)]` on an `impl` of type `T` provides instances of type `Arc<RwLock<T>>`.
///
/// The provided instance will be an atomically reference-counted pointer ([`Arc`](std::sync::Arc)) that is shared globally, i.e., pointers provided by any singleton binding will point to the same instance.
///
/// Functions providing a singleton instance cannot depend on any arguments.
///
///```
/// #[provides(singleton_inject)]
/// impl ProvidedSingleton {
///     pub fn new() -> Self {
///         Self { }
///     }
/// }
/// #
/// # use dirk::provides::Provider;
/// # use dirk::provides;
/// #
/// # struct ProvidedSingleton { }
/// # impl ProvidedSingleton {
/// #     pub(crate) fn inner(&self) -> usize {
/// #         return 42;
/// #     }
/// # }
/// #
/// # let factory = SingletonFactoryProvidedSingleton::create();
/// # let provided = factory.get();
/// # assert_eq!(provided.read().unwrap().inner(), 42);
///```
///
#[proc_macro_error]
#[proc_macro_attribute]
pub fn provides(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = provides::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.abort(),
    }
}

/// May be used to facilitate injecting or querying types provided in a different module
///
/// There are a few conditions that need to be met in order for this to work:
/// - The `impl` annotated with a `#[provides(...)]` macro needs to be present in the same module as the type it provides.
/// - The argument of the `#[use_provides(...)]` macro needs to match the one on the corresponding `#[provides(...)]` macro. If no argument is given, the default `static_inject` is assumed.
///
///```
/// #
/// #[use_provides(scoped_inject)]
/// use engine::Engine;
///
/// mod engine {
/// #    use dirk::provides;
/// #
///     pub(crate) struct Engine {
///         power: usize
///     }
///
///     #[provides(scoped_inject)]
///     impl Engine {
///         fn new() -> Self {
///             Self { power: 200 }
///         }
///     }
/// #
/// #   impl Engine {
/// #       pub(crate) fn power(&self) -> usize {
/// #           self.power
/// #       }
/// #   }
/// }
///
/// #[component(engine: scoped_bind(Engine))]
/// trait Car {
///     fn engine(&self) -> std::rc::Rc<std::cell::RefCell<Engine>>;
/// }
/// #
/// # use dirk::{component, use_provides, component::StaticComponent};
/// # let car = DirkCar::create();
/// # assert_eq!(car.engine().borrow().power(), 200);
/// #
///```
///
#[proc_macro_error]
#[proc_macro_attribute]
pub fn use_provides(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = use_provides::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.abort(),
    }
}

/// Declares a component that may be used to instantiate types.
///
/// # Usage
///
/// `#[component(...)]` may be placed on a `trait` specifying functions that can later be used to retrieve instances.
/// It may contain definitions of so-called bindings and their dependencies.
///
/// The macro results in the generation of a type providing a builder for an implementation of the `trait`, whose **name is prefixed by `Dirk`**.
///
///```no_run
/// #
/// use dirk::component; /// Required by `#[component(...)]`
///
/// #[component(
///    // ...  (bindings)
/// )]
/// trait AComponent {
///    // ... (binding `fn`s)
/// }
///
/// /// Required when using a `Component`
/// use dirk::component::Component;
/// use dirk::component::builder::Builder;
///
/// /// 1. Via a type-safe builder
/// let component = DirkAComponent::builder() // prefix `Dirk` !!!
///         // .<...>(...) (provide instance bindings here)
///            .build();
///
/// /// 2. ... or using the `create` function, in case no instance bindings are specified
/// use dirk::component::StaticComponent;
/// let component = DirkAComponent::create(); // prefix `Dirk` !!!
///
/// // Invoke binding `fn`s to retrieve instances, e.g.,
/// // let foo = component.<...>();
/// ```
///
/// # Bindings
///
/// A binding consists of four parts:
/// - a name
/// - a binding specifier (e.g., `static_bind(...)` or `cloned_instance_bind(...)`)
/// - a type
/// - (optional) dependencies, e.g. `[a_binding, b_binding, c_binding]`
///
///```no_run
/// #[component(
///     name: cloned_instance_bind(&'static str),
///     user_service: scoped_bind(UserService),
///     auth_service: scoped_bind(AuthService),
///     application: static_bind(Application) [name, user_service, auth_service]
/// )]
/// //  ^         ^  ^         ^ ^         ^  ^                                ^
/// //  |_ name  _|  |specifier| |_ type  _|  |____    dependencies        ____|
/// trait ApplicationComponent {
///     fn user_service(&self) -> std::rc::Rc<std::cell::RefCell<UserService>>;
///     fn auth_service(&self) -> std::rc::Rc<std::cell::RefCell<AuthService>>;
///     fn application(&self) -> Application;
/// }
/// #
/// # use dirk::component;
/// # use dirk::provides;
/// #
/// # struct UserService {}
/// #
/// # #[provides(scoped_inject)]
/// # impl UserService {
/// #     fn new() -> Self {
/// #         UserService { }
/// #     }    
/// # }
/// #
/// #
/// # struct AuthService {}
/// #
/// # #[provides(scoped_inject)]
/// # impl AuthService {
/// #     fn new() -> Self {
/// #         AuthService { }
/// #     }    
/// # }
/// #
/// #
/// # struct Application {
/// #     name: &'static str,
/// #     user_service: std::rc::Rc<std::cell::RefCell<UserService>>,
/// #     auth_service: std::rc::Rc<std::cell::RefCell<AuthService>>
/// # }
/// #
/// # #[provides]
/// # impl Application{
/// #     fn new(name: &'static str, user_service: std::rc::Rc<std::cell::RefCell<UserService>>, auth_service: std::rc::Rc<std::cell::RefCell<AuthService>>) -> Self {
/// #         Application { name, user_service, auth_service}
/// #     }    
/// # }
///```
///
/// If a binding is supposed to be queried, a corresponding function needs to be specified in the `trait`.
/// In case a binding acts only as a dependency for other bindings, such a function can be omitted.
///
///
/// ## Static bindings
/// `static_bind(T)` may be used to declare a static binding of type `T`.
///
/// ## Scoped bindings
/// `scoped_bind(T)` may be used to declare a scoped binding of type `Rc<RefCell<T>>`.
///
/// ## Singleton bindings
/// `singleton_bind(T)` may be used to declare a singleton binding of type `Arc<RwLock<T>>`.
///
/// ## Cloned instance bindings
/// `cloned_instance_bind(T)` may be used to declare a user-provided binding of type `T` where `T: Clone + 'static`, which is cloned every time it is queried or injected.
///
/// ## Scoped instance bindings
/// `scoped_instance_bind(T)` may be used to declare a user-provided binding of type `Rc<RefCell<T>>` where `T: + 'static`, such that all queried or injected `Rc`s point to the same instance.
///
#[proc_macro_error]
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let data = ComponentMacroData::new(attr, item);
    let res = data
        .is_helper()
        .map_err(Into::<ComponentError>::into)
        .and_then(|is_helper| {
            #[allow(clippy::if_not_else)]
            if !is_helper {
                component::_macro(data).map_err(Into::<ComponentError>::into)
            } else {
                component::_macro_helper(data)
            }
        });

    match res {
        Ok(item) => item,
        Err(e) => e.abort(),
    }
}

/// May be used to facilitate using components defined in a different module
///
///```
/// #
/// mod car {
///     use dirk::{provides, component};
///
///     pub(crate) struct Engine {
///         power: usize
///     }
///
///     #[provides(scoped_inject)]
///     impl Engine {
///         fn new() -> Self {
///             Self { power: 200 }
///         }
///     }
/// #
/// #   impl Engine {
/// #       pub(crate) fn power(&self) -> usize {
/// #           self.power
/// #       }
/// #   }
///
///     #[component(engine: scoped_bind(Engine))]
///     pub(crate) trait Car {
///         fn engine(&self) -> std::rc::Rc<std::cell::RefCell<Engine>>;
///     }
/// }
///
/// #[use_component]
/// use car::Car;
///
/// # use dirk::{use_component, component::StaticComponent};
/// let car = DirkCar::create();
/// assert_eq!(car.engine().borrow().power(), 200);
/// #
///```
///
#[proc_macro_error]
#[proc_macro_attribute]
pub fn use_component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let res = use_component::_macro(attr, item);

    match res {
        Ok(item) => item,
        Err(e) => e.abort(),
    }
}

#[cfg(test)]
mod tests {}
