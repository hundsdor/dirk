use std::sync::Arc;

use heater::{ElectricHeater, Heater};
use pump::{Pump, ThermoSiphon};

use logger::{CoffeeLogger, Logger};
use shaku::{module, Component, HasComponent, Interface};

module! {
    CoffeeShop{
        components = [CoffeeMaker, CoffeeLogger, ElectricHeater, ThermoSiphon],
        providers = []
    }
}

fn main() {
    let coffee_shop = CoffeeShop::builder().build();

    let maker: &dyn Maker = coffee_shop.resolve_ref();
    maker.brew();
    let logger: &dyn Logger = coffee_shop.resolve_ref();
    logger.logs().iter().for_each(|l| println!("{l}"));
}

//######################################################################################################################

trait Maker: Interface {
    fn brew(&mut self);
}

#[derive(Component)]
#[shaku(interface = Maker)]
struct CoffeeMaker {
    #[shaku(inject)]
    logger: Arc<dyn Logger>,
    #[shaku(inject)]
    heater: Arc<dyn Heater>,
    #[shaku(inject)]
    pump: Arc<dyn Pump>,
}

impl Maker for CoffeeMaker {
    fn brew(&mut self) {
        self.heater.on();
        self.pump.pump();
        self.logger.log(" [_]P coffee! [_]P ".to_owned());
        self.heater.off();
    }
}

mod logger {
    use shaku::{Component, Interface};

    pub trait Logger: Interface {
        fn log(&mut self, msg: String);
        fn logs(&self) -> &Vec<String>;
    }

    #[derive(Component)]
    #[shaku(interface = Logger)]
    pub struct CoffeeLogger {
        #[shaku(default)]
        logs: Vec<String>,
    }

    impl Logger for CoffeeLogger {
        fn log(&mut self, msg: String) {
            self.logs.push(msg);
        }

        fn logs(&self) -> &Vec<String> {
            &self.logs
        }
    }
}

mod heater {

    use shaku::{Component, Interface};

    use crate::logger::{CoffeeLogger, Logger};
    use std::sync::{Arc, RwLock};

    pub trait Heater: Interface {
        fn on(&mut self);
        fn off(&mut self);
        fn is_hot(&self) -> bool;
    }

    #[derive(Component)]
    #[shaku(interface = Heater)]
    pub struct ElectricHeater {
        #[shaku(inject)]
        logger: Arc<dyn Logger>,
        #[shaku(default)]
        heating: bool,
    }

    impl Heater for ElectricHeater {
        fn on(&mut self) {
            self.heating = true;
            self.logger.log("~ ~ ~ heating ~ ~ ~".to_owned());
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

    use shaku::{Component, Interface};

    use crate::{
        heater::Heater,
        logger::{CoffeeLogger, Logger},
    };
    use std::{
        cell::RefCell,
        rc::Rc,
        sync::{Arc, RwLock},
    };

    pub trait Pump: Interface {
        fn pump(&mut self);
    }

    #[derive(Component)]
    #[shaku(interface = Pump)]
    pub struct ThermoSiphon {
        #[shaku(inject)]
        logger: Arc<dyn Logger>,
        #[shaku(inject)]
        heater: Arc<dyn Heater>,
    }

    impl Pump for ThermoSiphon {
        fn pump(&mut self) {
            if self.heater.is_hot() {
                self.logger.log("=> => pumping => =>".to_owned());
            }
        }
    }
}
