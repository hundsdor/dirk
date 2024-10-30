use teloc::{dev::container, Dependency, ServiceProvider};

fn foo<T: Dependency<()>>() {
    let container = ServiceProvider::new().add_transient::<T>();
    let scope = container.fork();

    let foo: T = scope.resolve(); // This does not work and I don't know why
}

struct Foo {}

impl Dependency<()> for Foo {
    fn init(deps: ()) -> Self {
        Foo {}
    }
}

fn main() {
    foo::<Foo>();
}
