//! An example involving a car composed of an engine

use dirk_framework::use_component;

mod car {
    use dirk_framework::{component, provides};

    pub(crate) struct Engine {
        power: usize,
    }

    #[provides(scoped_inject)]
    impl Engine {
        fn new() -> Self {
            Self { power: 200 }
        }
    }

    impl Engine {
        pub(crate) fn power(&self) -> usize {
            self.power
        }
    }

    #[component(engine: scoped_bind(Engine))]
    pub(crate) trait Car {
        fn engine(&self) -> std::rc::Rc<std::cell::RefCell<Engine>>;
    }
}

#[use_component]
fn main() {
    let car = DirkCar::create();
    assert_eq!(car.engine().borrow().power(), 200);
}
