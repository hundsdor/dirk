use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

use heater::{ElectricHeater, Heater};
use logger::CoffeeLogger;
use pump::{Pump, ThermoSiphon};
use teloc::{inject, Dependency, Resolver, ServiceProvider};

fn main() {
    let container = ServiceProvider::new().add_transient::<CoffeeMaker>();
    let scope = container
        .fork()
        .add_singleton::<Rc<RefCell<CoffeeLogger>>>()
        .add_singleton::<Rc<RefCell<ElectricHeater>>>()
        .add_singleton::<Rc<RefCell<ThermoSiphon>>>();

    let logger: Rc<RefCell<CoffeeLogger>> = scope.resolve();
    let heater: Rc<RefCell<ElectricHeater>> = scope.resolve();
    let pump: Rc<RefCell<ThermoSiphon>> = scope.resolve();
    let maker: CoffeeMaker = scope.resolve();

    let coffee_shop: CoffeeShop = scope.resolve(); // This does not work and I don't know why

    coffee_shop.maker().brew();
    coffee_shop
        .logger()
        .borrow_mut()
        .logs()
        .iter()
        .for_each(|l| println!("{l}"));
}

#[derive(Dependency)]
struct CoffeeShop {
    maker: CoffeeMaker,
    logger: Rc<RefCell<CoffeeLogger>>,
}

impl CoffeeShop {
    fn maker(&self) -> &CoffeeMaker {
        &self.maker
    }
    fn logger(&self) -> &Rc<RefCell<CoffeeLogger>> {
        &self.logger
    }
}

//######################################################################################################################

struct CoffeeMaker {
    logger: Rc<RefCell<CoffeeLogger>>,
    heater: Rc<RefCell<ElectricHeater>>,
    pump: Rc<RefCell<ThermoSiphon>>,
}

#[inject]
impl CoffeeMaker {
    fn new(
        logger: Rc<RefCell<CoffeeLogger>>,
        heater: Rc<RefCell<ElectricHeater>>,
        pump: Rc<RefCell<ThermoSiphon>>,
    ) -> Self {
        Self {
            logger,
            heater,
            pump,
        }
    }
}

impl CoffeeMaker {
    fn brew(&mut self) {
        self.heater.borrow_mut().on();
        self.pump.borrow_mut().pump();
        self.logger
            .borrow_mut()
            .log(" [_]P coffee! [_]P ".to_owned());
        self.heater.borrow_mut().off();
    }
}

mod logger {
    use teloc::inject;

    pub struct CoffeeLogger {
        logs: Vec<String>,
    }

    #[inject]
    impl CoffeeLogger {
        fn new() -> Self {
            Self { logs: Vec::new() }
        }
    }

    impl CoffeeLogger {
        pub fn log(&mut self, msg: String) {
            self.logs.push(msg);
        }

        pub fn logs(&self) -> &Vec<String> {
            &self.logs
        }
    }
}

mod heater {

    use teloc::inject;

    use crate::logger::CoffeeLogger;
    use std::{
        cell::RefCell,
        rc::Rc,
        sync::{Arc, RwLock},
    };

    pub trait Heater {
        fn on(&mut self);
        fn off(&mut self);
        fn is_hot(&self) -> bool;
    }

    pub struct ElectricHeater {
        logger: Rc<RefCell<CoffeeLogger>>,
        heating: bool,
    }

    #[inject]
    impl ElectricHeater {
        fn new(logger: Rc<RefCell<CoffeeLogger>>) -> Self {
            Self {
                logger,
                heating: false,
            }
        }
    }

    impl Heater for ElectricHeater {
        fn on(&mut self) {
            self.heating = true;
            self.logger
                .borrow_mut()
                .log("~ ~ ~ heating ~ ~ ~".to_owned());
        }

        fn off(&mut self) {
            self.heating = false;
        }

        fn is_hot(&self) -> bool {
            self.heating
        }
    }
}

mod pump {

    use teloc::inject;

    use crate::{
        heater::{ElectricHeater, Heater},
        logger::CoffeeLogger,
    };
    use std::{cell::RefCell, rc::Rc};

    pub trait Pump {
        fn pump(&mut self);
    }

    pub struct ThermoSiphon {
        logger: Rc<RefCell<CoffeeLogger>>,
        heater: Rc<RefCell<ElectricHeater>>,
    }

    #[inject]
    impl ThermoSiphon {
        fn new(logger: Rc<RefCell<CoffeeLogger>>, heater: Rc<RefCell<ElectricHeater>>) -> Self {
            Self { logger, heater }
        }
    }

    impl Pump for ThermoSiphon {
        fn pump(&mut self) {
            if self.heater.borrow().is_hot() {
                self.logger
                    .borrow_mut()
                    .log("=> => pumping => =>".to_owned());
            }
        }
    }
}
