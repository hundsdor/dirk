//! An example involving a coffee machine

use std::rc::Rc;

use heater::Heater;
use logger::Logger;
use pump::Pump;

use waiter_di::{component, profiles, provides, Component, Container, Provider};

fn main() {
    let mut container = Container::<profiles::Default>::new();
    let coffee_shop = Provider::<CoffeeShop>::get(&mut container);
    coffee_shop.maker.brew();
}

#[component]
struct CoffeeShop {
    maker: CoffeeMaker,
}

//######################################################################################################################

trait Maker {}

#[component]
struct CoffeeMaker {
    logger: Rc<dyn Logger>,
    heater: Rc<dyn Heater>,
    pump: Rc<dyn Pump>,
}

#[provides]
impl Maker for CoffeeMaker {}

impl CoffeeMaker {
    fn brew(&self) {
        self.heater.on();
        self.pump.pump();
        self.logger.log(" [_]P coffee! [_]P ".to_owned());
        self.heater.off();
    }
}

mod logger {
    use waiter_di::{component, provides, Component};

    pub trait Logger {
        fn log(&self, msg: String);
    }

    #[component]
    pub struct CoffeeLogger {
        // logs: Vec<String>,
    }

    #[provides]
    impl Logger for CoffeeLogger {
        fn log(&self, msg: String) {
            println!("{}", msg);
        }
    }
}

mod heater {

    use waiter_di::{component, provides, Component};

    use crate::logger::{CoffeeLogger, Logger};
    use std::rc::Rc;

    pub trait Heater {
        fn on(&self);
        fn off(&self);
        // fn is_hot(&self) -> bool;
    }

    #[component]
    pub struct ElectricHeater {
        logger: Rc<dyn Logger>,
        // heating: bool,
    }

    #[provides]
    impl Heater for ElectricHeater {
        fn on(&self) {
            // self.heating = true;
            self.logger.log("~ ~ ~ heating ~ ~ ~".to_owned());
        }

        fn off(&self) {
            // self.heating = false;
        }

        // fn is_hot(&self) -> bool {
        // self.heating
        // }
    }
}

mod pump {

    use waiter_di::{component, provides, Component};

    use crate::{heater::Heater, logger::Logger};
    use std::rc::Rc;

    pub trait Pump {
        fn pump(&self);
    }

    #[component]
    pub struct ThermoSiphon {
        logger: Rc<dyn Logger>,
        heater: Rc<dyn Heater>,
    }

    #[provides]
    impl Pump for ThermoSiphon {
        fn pump(&self) {
            // if self.heater.is_hot() {
            self.logger.log("=> => pumping => =>".to_owned());
            // }
        }
    }
}
