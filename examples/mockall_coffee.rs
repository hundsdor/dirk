//! An example involving a coffee machine - using mockall

use std::{cell::RefCell, rc::Rc};

use dirk_framework::provides;
use heater::Heater;
use pump::Pump;

/* Code commented out may be used in production */

// use dirk_framework::component;

fn main() {
    // let coffee_shop = DirkCoffeeShop::create();
    // coffee_shop.maker().brew();
}

// #[component(
//     heater: scoped_bind(/* */),
//     pump: scoped_instance_bind(/* */),
//     maker: static_bind(CoffeeMaker</* */, /* */>) [heater, pump]
// )]
// trait CoffeeShop<H: Heater + 'static, P: Pump + 'static> {
//     fn maker(&self) -> CoffeeMaker<H, P>;
// }

#[cfg(test)]
mod test {
    use dirk_framework::{
        component,
        component::{builder::Builder, Component},
        use_provides,
    };

    use crate::pump::MockPump;
    use crate::{
        heater::{Heater, MockHeater},
        pump::Pump,
    };

    #[use_provides(static_inject)]
    use crate::CoffeeMaker;

    // Used for testing
    #[component(
        heater: scoped_instance_bind(H),
        pump: scoped_instance_bind(P),
        maker: static_bind(CoffeeMaker<H, P>) [heater, pump]
    )]
    trait CoffeeShop<H: Heater + 'static, P: Pump + 'static> {
        fn maker(&self) -> CoffeeMaker<H, P>;
    }

    #[test]
    fn test_coffe_maker() {
        // prepare mocks
        let mut heater_mock = MockHeater::new();
        let mut pump_mock = MockPump::new();

        heater_mock.expect_on().once().return_const(());
        pump_mock.expect_pump().once().return_const(());
        heater_mock.expect_off().once().return_const(());

        let coffee_shop = DirkCoffeeShop::builder()
            .heater(heater_mock)
            .pump(pump_mock)
            .build();

        // call function
        coffee_shop.maker().brew();
    }
}

//######################################################################################################################

#[allow(dead_code)]
struct CoffeeMaker<H: Heater, P: Pump> {
    heater: Rc<RefCell<H>>,
    pump: Rc<RefCell<P>>,
}

#[provides]
impl<H: Heater, P: Pump> CoffeeMaker<H, P> {
    #[allow(dead_code)]
    fn new(heater: Rc<RefCell<H>>, pump: Rc<RefCell<P>>) -> Self {
        Self { heater, pump }
    }
}

impl<H: Heater, P: Pump> CoffeeMaker<H, P> {
    #[allow(dead_code)]
    fn brew(&mut self) {
        self.heater.borrow_mut().on();
        self.pump.borrow_mut().pump();
        println!(" [_]P coffee! [_]P ");
        self.heater.borrow_mut().off();
    }
}

mod heater {
    use mockall::automock;

    #[automock]
    pub trait Heater {
        fn on(&mut self);
        fn off(&mut self);
    }
}

mod pump {
    use mockall::mock;

    mock! {
        pub Pump {}

        impl Pump for Pump {
            fn pump(&mut self);
        }
    }

    pub trait Pump {
        fn pump(&mut self);
    }
}
