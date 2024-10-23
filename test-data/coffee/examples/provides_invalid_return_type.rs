//! An example involving a coffee machine

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

use dirk::{component, component::StaticComponent, provides, use_provides};

use heater::Heater;
use pump::Pump;

use crate::logger::SingletonFactoryOption;
use logger::CoffeeLogger;

#[use_provides(scoped_inject)]
use heater::ElectricHeater;
#[use_provides(scoped_inject)]
use pump::ThermoSiphon;

fn main() {
    let coffee_shop = DirkCoffeeShop::create();
    coffee_shop.maker().brew();
    coffee_shop
        .logger()
        .read()
        .unwrap()
        .as_ref()
        .map(|logger| logger.logs().iter().for_each(|l| println!("{l}")));
}

#[component(
    logger: singleton_bind(Option<CoffeeLogger>),
    heater: scoped_bind(ElectricHeater) [logger],
    pump: scoped_bind(ThermoSiphon<ElectricHeater>) [logger, heater],
    maker: static_bind(CoffeeMaker<ElectricHeater, ThermoSiphon<ElectricHeater>>) [logger, heater, pump]
)]
trait CoffeeShop<H: Heater, P: Pump> {
    fn maker(&self) -> CoffeeMaker<H, P>;
    fn logger(&self) -> Arc<RwLock<Option<CoffeeLogger>>>;
}

//######################################################################################################################

struct CoffeeMaker<H: Heater, P: Pump> {
    logger: Arc<RwLock<Option<CoffeeLogger>>>,
    heater: Rc<RefCell<H>>,
    pump: Rc<RefCell<P>>,
}

#[provides]
impl<H: Heater, P: Pump> CoffeeMaker<H, P> {
    fn new(
        logger: Arc<RwLock<Option<CoffeeLogger>>>,
        heater: Rc<RefCell<H>>,
        pump: Rc<RefCell<P>>,
    ) -> Self {
        Self {
            logger,
            heater,
            pump,
        }
    }
}

impl<H: Heater, P: Pump> CoffeeMaker<H, P> {
    fn brew(&mut self) {
        self.heater.borrow_mut().on();
        self.pump.borrow_mut().pump();
        self.logger
            .write()
            .unwrap()
            .as_mut()
            .map(|logger| logger.log(" [_]P coffee! [_]P ".to_owned()));
        self.heater.borrow_mut().off();
    }
}

mod logger {
    use dirk::provides;

    pub struct CoffeeLogger {
        logs: Vec<String>,
    }

    #[provides(singleton_inject)]
    impl CoffeeLogger {
        fn new() -> Option<Self> {
            Some(Self { logs: Vec::new() })
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
    use dirk::provides;

    use crate::logger::CoffeeLogger;
    use std::sync::{Arc, RwLock};

    pub trait Heater {
        fn on(&mut self);
        fn off(&mut self);
        fn is_hot(&self) -> bool;
    }

    pub struct ElectricHeater {
        logger: Arc<RwLock<Option<CoffeeLogger>>>,
        heating: bool,
    }

    #[provides(scoped_inject)]
    impl ElectricHeater {
        fn new(logger: Arc<RwLock<Option<CoffeeLogger>>>) -> Self {
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
                .write()
                .unwrap()
                .as_mut()
                .map(|logger| logger.log("~ ~ ~ heating ~ ~ ~".to_owned()));
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
    use dirk::provides;

    use crate::{heater::Heater, logger::CoffeeLogger};
    use std::{
        cell::RefCell,
        rc::Rc,
        sync::{Arc, RwLock},
    };

    pub trait Pump {
        fn pump(&mut self);
    }

    pub struct ThermoSiphon<H: Heater> {
        logger: Arc<RwLock<Option<CoffeeLogger>>>,
        heater: Rc<RefCell<H>>,
    }

    #[provides(scoped_inject)]
    impl<H: Heater> ThermoSiphon<H> {
        fn new(logger: Arc<RwLock<Option<CoffeeLogger>>>, heater: Rc<RefCell<H>>) -> Self {
            Self { logger, heater }
        }
    }

    impl<H: Heater> Pump for ThermoSiphon<H> {
        fn pump(&mut self) {
            if self.heater.borrow().is_hot() {
                self.logger
                    .write()
                    .unwrap()
                    .as_mut()
                    .map(|logger| logger.log("=> => pumping => =>".to_owned()));
            }
        }
    }
}
