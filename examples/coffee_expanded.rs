use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

use heater::{ElectricHeater, ElectricHeaterScopedFactory, Heater};
use logger::{CoffeeLogger, CoffeeLoggerSingletonFactory};
use pump::{Pump, ThermoSiphon, ThermoSiphonScopedFactory};

fn main() {
    let coffee_shop = StilettoCoffeeShop::builder().build();
    coffee_shop.maker().brew();
    coffee_shop
        .logger()
        .write()
        .unwrap()
        .logs()
        .iter()
        .for_each(|l| println!("{l}"));
}

//######################################################################################################################
//#  component

struct StilettoCoffeeShopBuilder {}

impl StilettoCoffeeShopBuilder {
    fn build(&self) -> impl CoffeeShop<ElectricHeater, ThermoSiphon<ElectricHeater>> {
        StilettoCoffeeShopImpl::new()
    }
}

struct StilettoCoffeeShopImpl {
    loggerProvider: Rc<dyn Provider<Arc<RwLock<CoffeeLogger>>>>,
    heaterProvider: Rc<dyn Provider<Rc<RefCell<ElectricHeater>>>>,
    pumpProvider: Rc<dyn Provider<Rc<RefCell<ThermoSiphon<ElectricHeater>>>>>,
    makerProvider: Rc<dyn Provider<CoffeeMaker<ElectricHeater, ThermoSiphon<ElectricHeater>>>>,
}

impl StilettoCoffeeShopImpl {
    fn new() -> Self {
        let loggerProvider = Rc::new(CoffeeLoggerSingletonFactory::create());
        let heaterProvider = Rc::new(ElectricHeaterScopedFactory::create(loggerProvider.clone()));
        let pumpProvider = Rc::new(ThermoSiphonScopedFactory::create(
            loggerProvider.clone(),
            heaterProvider.clone(),
        ));
        let makerProvider = Rc::new(CoffeeMakerStaticFactory::create(
            loggerProvider.clone(),
            heaterProvider.clone(),
            pumpProvider.clone(),
        ));
        Self {
            loggerProvider,
            heaterProvider,
            pumpProvider,
            makerProvider,
        }
    }
}

impl CoffeeShop<ElectricHeater, ThermoSiphon<ElectricHeater>> for StilettoCoffeeShopImpl {
    fn maker(&self) -> CoffeeMaker<ElectricHeater, ThermoSiphon<ElectricHeater>> {
        self.makerProvider.get()
    }

    fn logger(&self) -> Arc<RwLock<CoffeeLogger>> {
        self.loggerProvider.get()
    }
}

struct StilettoCoffeeShop {}

impl StilettoCoffeeShop {
    fn builder() -> StilettoCoffeeShopBuilder {
        StilettoCoffeeShopBuilder {}
    }

    fn create() -> impl CoffeeShop<ElectricHeater, ThermoSiphon<ElectricHeater>> {
        StilettoCoffeeShopBuilder {}.build()
    }
}

trait CoffeeShop<H: Heater, P: Pump> {
    fn maker(&self) -> CoffeeMaker<H, P>;
    fn logger(&self) -> Arc<RwLock<CoffeeLogger>>;
}

//#
//######################################################################################################################

pub struct CoffeeMaker<H: Heater, P: Pump> {
    logger: Arc<RwLock<CoffeeLogger>>,
    heater: Rc<RefCell<H>>,
    pump: Rc<RefCell<P>>,
}

//######################################################################################################################
//# static_inject

use stiletto::Provider;

pub struct CoffeeMakerStaticFactory<H: Heater, P: Pump> {
    loggerProvider: Rc<dyn Provider<Arc<RwLock<CoffeeLogger>>>>,
    heaterProvider: Rc<dyn Provider<Rc<RefCell<H>>>>,
    pumpProvider: Rc<dyn Provider<Rc<RefCell<P>>>>,
}

impl<H: Heater, P: Pump> Provider<CoffeeMaker<H, P>> for CoffeeMakerStaticFactory<H, P> {
    fn get(&self) -> CoffeeMaker<H, P> {
        Self::newInstance(
            self.loggerProvider.get(),
            self.heaterProvider.get(),
            self.pumpProvider.get(),
        )
    }
}

impl<H: Heater, P: Pump> CoffeeMakerStaticFactory<H, P> {
    fn new(
        loggerProvider: Rc<dyn Provider<Arc<RwLock<CoffeeLogger>>>>,
        heaterProvider: Rc<dyn Provider<Rc<RefCell<H>>>>,
        pumpProvider: Rc<dyn Provider<Rc<RefCell<P>>>>,
    ) -> Self {
        Self {
            loggerProvider,
            heaterProvider,
            pumpProvider,
        }
    }

    fn create(
        loggerProvider: Rc<dyn Provider<Arc<RwLock<CoffeeLogger>>>>,
        heaterProvider: Rc<dyn Provider<Rc<RefCell<H>>>>,
        pumpProvider: Rc<dyn Provider<Rc<RefCell<P>>>>,
    ) -> Self {
        Self::new(loggerProvider, heaterProvider, pumpProvider)
    }

    fn newInstance(
        logger: Arc<RwLock<CoffeeLogger>>,
        heater: Rc<RefCell<H>>,
        pump: Rc<RefCell<P>>,
    ) -> CoffeeMaker<H, P> {
        CoffeeMaker::new(logger, heater, pump)
    }
}

impl<H: Heater, P: Pump> CoffeeMaker<H, P> {
    fn new(
        logger: Arc<RwLock<CoffeeLogger>>,
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

//#
//######################################################################################################################

impl<H: Heater, P: Pump> CoffeeMaker<H, P> {
    fn brew(&mut self) {
        self.heater.borrow_mut().on();
        self.pump.borrow_mut().pump();
        self.logger
            .write()
            .unwrap()
            .log(" [_]P coffee! [_]P ".to_owned());
        self.heater.borrow_mut().off();
    }
}

mod logger {
    use stiletto_macros::singleton_inject;

    use stiletto::Provider;

    use once_cell::sync::Lazy;
    use std::sync::{Arc, RwLock};

    pub struct CoffeeLogger {
        logs: Vec<String>,
    }

    //######################################################################################################################
    // singleton_inject

    #[derive(Clone)]
    pub struct CoffeeLoggerSingletonFactory {
        singleton: Arc<RwLock<CoffeeLogger>>,
    }

    impl Provider<Arc<RwLock<CoffeeLogger>>> for CoffeeLoggerSingletonFactory {
        fn get(&self) -> Arc<RwLock<CoffeeLogger>> {
            self.singleton.clone()
        }
    }

    impl CoffeeLoggerSingletonFactory {
        fn new() -> Self {
            Self {
                singleton: Self::newInstance(),
            }
        }

        pub fn create() -> Self {
            COFFEE_LOGGER_FACTORY_INSTANCE.clone()
        }

        fn newInstance() -> Arc<RwLock<CoffeeLogger>> {
            Arc::new(RwLock::new(CoffeeLogger::new()))
        }
    }

    static COFFEE_LOGGER_FACTORY_INSTANCE: Lazy<CoffeeLoggerSingletonFactory> =
        Lazy::new(|| CoffeeLoggerSingletonFactory::new());

    impl CoffeeLogger {
        fn new() -> Self {
            Self { logs: Vec::new() }
        }
    }

    //#
    //######################################################################################################################

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
    use stiletto_macros::scoped_inject;

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
        logger: Arc<RwLock<CoffeeLogger>>,
        heating: bool,
    }

    //######################################################################################################################
    //# scoped_inject

    use stiletto::Provider;

    pub struct ElectricHeaterScopedFactory {
        singleton: Rc<RefCell<ElectricHeater>>,
    }

    impl Provider<Rc<RefCell<ElectricHeater>>> for ElectricHeaterScopedFactory {
        fn get(&self) -> Rc<RefCell<ElectricHeater>> {
            self.singleton.clone()
        }
    }

    impl ElectricHeaterScopedFactory {
        fn new(loggerProvider: Rc<dyn Provider<Arc<RwLock<CoffeeLogger>>>>) -> Self {
            Self {
                singleton: Self::newInstance(loggerProvider.get()),
            }
        }

        pub fn create(loggerProvider: Rc<dyn Provider<Arc<RwLock<CoffeeLogger>>>>) -> Self {
            Self::new(loggerProvider)
        }

        fn newInstance(logger: Arc<RwLock<CoffeeLogger>>) -> Rc<RefCell<ElectricHeater>> {
            Rc::new(RefCell::new(ElectricHeater::new(logger)))
        }
    }

    impl ElectricHeater {
        fn new(logger: Arc<RwLock<CoffeeLogger>>) -> Self {
            Self {
                logger,
                heating: false,
            }
        }
    }

    //#
    //######################################################################################################################

    impl Heater for ElectricHeater {
        fn on(&mut self) {
            self.heating = true;
            self.logger
                .write()
                .unwrap()
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
    use stiletto_macros::scoped_inject;

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
        logger: Arc<RwLock<CoffeeLogger>>,
        heater: Rc<RefCell<H>>,
    }

    //######################################################################################################################
    //# scoped_inject

    use stiletto::Provider;

    pub struct ThermoSiphonScopedFactory<H: Heater> {
        singleton: Rc<RefCell<ThermoSiphon<H>>>,
    }

    impl<H: Heater> Provider<Rc<RefCell<ThermoSiphon<H>>>> for ThermoSiphonScopedFactory<H> {
        fn get(&self) -> Rc<RefCell<ThermoSiphon<H>>> {
            self.singleton.clone()
        }
    }

    impl<H: Heater> ThermoSiphonScopedFactory<H> {
        fn new(
            loggerProvider: Rc<dyn Provider<Arc<RwLock<CoffeeLogger>>>>,
            heaterProvider: Rc<dyn Provider<Rc<RefCell<H>>>>,
        ) -> Self {
            Self {
                singleton: Self::newInstance(loggerProvider.get(), heaterProvider.get()),
            }
        }

        pub fn create(
            loggerProvider: Rc<dyn Provider<Arc<RwLock<CoffeeLogger>>>>,
            heaterProvider: Rc<dyn Provider<Rc<RefCell<H>>>>,
        ) -> Self {
            Self::new(loggerProvider, heaterProvider)
        }

        fn newInstance(
            logger: Arc<RwLock<CoffeeLogger>>,
            heater: Rc<RefCell<H>>,
        ) -> Rc<RefCell<ThermoSiphon<H>>> {
            Rc::new(RefCell::new(ThermoSiphon::new(logger, heater)))
        }
    }

    impl<H: Heater> ThermoSiphon<H> {
        fn new(logger: Arc<RwLock<CoffeeLogger>>, heater: Rc<RefCell<H>>) -> Self {
            Self { logger, heater }
        }
    }

    //#
    //######################################################################################################################

    impl<H: Heater> Pump for ThermoSiphon<H> {
        fn pump(&mut self) {
            if self.heater.borrow().is_hot() {
                self.logger
                    .write()
                    .unwrap()
                    .log("=> => pumping => =>".to_owned());
            }
        }
    }
}
