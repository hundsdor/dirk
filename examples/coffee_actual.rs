use heater::{ElectricHeater, Heater};
use logger::CoffeeLogger;
use pump::{Pump, ThermoSiphon};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};
use stiletto_macros::component;
use stiletto_macros::static_inject;
fn main() {
    let coffee_shop = StilettoCoffeeShop::builder().build();
    coffee_shop.maker().brew();
    coffee_shop
        .logger()
        .write()
        .unwrap()
        .logs()
        .iter()
        .for_each(|l| {
            println!("{0}\n", l);
        });
}
pub(crate) struct StilettoCoffeeShopBuilder {}
impl StilettoCoffeeShopBuilder {
    fn build(&self) -> impl CoffeeShop<ElectricHeater, ThermoSiphon<ElectricHeater>> {
        StilettoCoffeeShopImpl::new()
    }
}
struct StilettoCoffeeShopImpl {
    loggerProvider:
        std::rc::Rc<dyn stiletto::Provider<std::sync::Arc<std::sync::RwLock<CoffeeLogger>>>>,
    heaterProvider:
        std::rc::Rc<dyn stiletto::Provider<std::rc::Rc<std::cell::RefCell<ElectricHeater>>>>,
    pumpProvider: std::rc::Rc<
        dyn stiletto::Provider<std::rc::Rc<std::cell::RefCell<ThermoSiphon<ElectricHeater>>>>,
    >,
    makerProvider: std::rc::Rc<
        dyn stiletto::Provider<CoffeeMaker<ElectricHeater, ThermoSiphon<ElectricHeater>>>,
    >,
}
impl StilettoCoffeeShopImpl {
    fn new() -> Self {
        let loggerProvider = Rc::new(FactoryCoffeeLogger::create());
        let heaterProvider = Rc::new(FactoryElectricHeater::create(loggerProvider.clone()));
        let pumpProvider = Rc::new(FactoryThermoSiphon::create(
            loggerProvider.clone(),
            heaterProvider.clone(),
        ));
        let makerProvider = Rc::new(FactoryCoffeeMaker::create(
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
    fn logger(&self) -> std::sync::Arc<std::sync::RwLock<CoffeeLogger>> {
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
use crate::heater::FactoryElectricHeater;
use crate::logger::FactoryCoffeeLogger;
use crate::pump::FactoryThermoSiphon;
pub struct CoffeeMaker<H: Heater, P: Pump> {
    logger: Arc<RwLock<CoffeeLogger>>,
    heater: Rc<RefCell<H>>,
    pump: Rc<RefCell<P>>,
}
pub(crate) struct FactoryCoffeeMaker<H: Heater, P: Pump> {
    loggerProvider: std::rc::Rc<dyn stiletto::Provider<Arc<RwLock<CoffeeLogger>>>>,
    heaterProvider: std::rc::Rc<dyn stiletto::Provider<Rc<RefCell<H>>>>,
    pumpProvider: std::rc::Rc<dyn stiletto::Provider<Rc<RefCell<P>>>>,
}
impl<H: Heater, P: Pump> stiletto::Provider<CoffeeMaker<H, P>> for FactoryCoffeeMaker<H, P> {
    fn get(&self) -> CoffeeMaker<H, P> {
        Self::newInstance(
            self.loggerProvider.get(),
            self.heaterProvider.get(),
            self.pumpProvider.get(),
        )
    }
}
impl<H: Heater, P: Pump> FactoryCoffeeMaker<H, P> {
    fn new(
        loggerProvider: std::rc::Rc<dyn stiletto::Provider<Arc<RwLock<CoffeeLogger>>>>,
        heaterProvider: std::rc::Rc<dyn stiletto::Provider<Rc<RefCell<H>>>>,
        pumpProvider: std::rc::Rc<dyn stiletto::Provider<Rc<RefCell<P>>>>,
    ) -> Self {
        Self {
            loggerProvider,
            heaterProvider,
            pumpProvider,
        }
    }
    pub fn create(
        loggerProvider: std::rc::Rc<dyn stiletto::Provider<Arc<RwLock<CoffeeLogger>>>>,
        heaterProvider: std::rc::Rc<dyn stiletto::Provider<Rc<RefCell<H>>>>,
        pumpProvider: std::rc::Rc<dyn stiletto::Provider<Rc<RefCell<P>>>>,
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
    pub struct CoffeeLogger {
        logs: Vec<String>,
    }
    pub(crate) struct FactoryCoffeeLogger {
        singleton: std::sync::Arc<std::sync::RwLock<CoffeeLogger>>,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for FactoryCoffeeLogger {
        #[inline]
        fn clone(&self) -> FactoryCoffeeLogger {
            FactoryCoffeeLogger {
                singleton: ::core::clone::Clone::clone(&self.singleton),
            }
        }
    }
    impl stiletto::Provider<std::sync::Arc<std::sync::RwLock<CoffeeLogger>>> for FactoryCoffeeLogger {
        fn get(&self) -> std::sync::Arc<std::sync::RwLock<CoffeeLogger>> {
            self.singleton.clone()
        }
    }
    impl FactoryCoffeeLogger {
        fn new() -> Self {
            Self {
                singleton: Self::newInstance(),
            }
        }
        pub fn create() -> Self {
            FactoryCoffeeLogger.clone()
        }
        fn newInstance() -> std::sync::Arc<std::sync::RwLock<CoffeeLogger>> {
            std::sync::Arc::new(std::sync::RwLock::new(CoffeeLogger::new()))
        }
    }
    static FactoryCoffeeLogger: stiletto::FactoryInstance<FactoryCoffeeLogger> =
        stiletto::FactoryInstance::new(|| FactoryCoffeeLogger::new());
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
    use crate::logger::CoffeeLogger;
    use std::sync::{Arc, RwLock};
    use stiletto_macros::scoped_inject;
    pub trait Heater {
        fn on(&mut self);
        fn off(&mut self);
        fn is_hot(&self) -> bool;
    }
    pub struct ElectricHeater {
        logger: Arc<RwLock<CoffeeLogger>>,
        heating: bool,
    }
    pub(crate) struct FactoryElectricHeater {
        singleton: std::rc::Rc<std::cell::RefCell<ElectricHeater>>,
    }
    impl stiletto::Provider<std::rc::Rc<std::cell::RefCell<ElectricHeater>>> for FactoryElectricHeater {
        fn get(&self) -> std::rc::Rc<std::cell::RefCell<ElectricHeater>> {
            self.singleton.clone()
        }
    }
    impl FactoryElectricHeater {
        fn new(
            loggerProvider: std::rc::Rc<dyn stiletto::Provider<Arc<RwLock<CoffeeLogger>>>>,
        ) -> Self {
            Self {
                singleton: Self::newInstance(loggerProvider.get()),
            }
        }
        pub fn create(
            loggerProvider: std::rc::Rc<dyn stiletto::Provider<Arc<RwLock<CoffeeLogger>>>>,
        ) -> Self {
            Self::new(loggerProvider)
        }
        fn newInstance(
            logger: Arc<RwLock<CoffeeLogger>>,
        ) -> std::rc::Rc<std::cell::RefCell<ElectricHeater>> {
            std::rc::Rc::new(std::cell::RefCell::new(ElectricHeater::new(logger)))
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
    use crate::{heater::Heater, logger::CoffeeLogger};
    use std::{
        cell::RefCell,
        rc::Rc,
        sync::{Arc, RwLock},
    };
    use stiletto_macros::scoped_inject;
    pub trait Pump {
        fn pump(&mut self);
    }
    pub struct ThermoSiphon<H: Heater> {
        logger: Arc<RwLock<CoffeeLogger>>,
        heater: Rc<RefCell<H>>,
    }
    pub(crate) struct FactoryThermoSiphon<H: Heater> {
        singleton: std::rc::Rc<std::cell::RefCell<ThermoSiphon<H>>>,
    }
    impl<H: Heater> stiletto::Provider<std::rc::Rc<std::cell::RefCell<ThermoSiphon<H>>>>
        for FactoryThermoSiphon<H>
    {
        fn get(&self) -> std::rc::Rc<std::cell::RefCell<ThermoSiphon<H>>> {
            self.singleton.clone()
        }
    }
    impl<H: Heater> FactoryThermoSiphon<H> {
        fn new(
            loggerProvider: std::rc::Rc<dyn stiletto::Provider<Arc<RwLock<CoffeeLogger>>>>,
            heaterProvider: std::rc::Rc<dyn stiletto::Provider<Rc<RefCell<H>>>>,
        ) -> Self {
            Self {
                singleton: Self::newInstance(loggerProvider.get(), heaterProvider.get()),
            }
        }
        pub fn create(
            loggerProvider: std::rc::Rc<dyn stiletto::Provider<Arc<RwLock<CoffeeLogger>>>>,
            heaterProvider: std::rc::Rc<dyn stiletto::Provider<Rc<RefCell<H>>>>,
        ) -> Self {
            Self::new(loggerProvider, heaterProvider)
        }
        fn newInstance(
            logger: Arc<RwLock<CoffeeLogger>>,
            heater: Rc<RefCell<H>>,
        ) -> std::rc::Rc<std::cell::RefCell<ThermoSiphon<H>>> {
            std::rc::Rc::new(std::cell::RefCell::new(ThermoSiphon::new(logger, heater)))
        }
    }
    impl<H: Heater> ThermoSiphon<H> {
        fn new(logger: Arc<RwLock<CoffeeLogger>>, heater: Rc<RefCell<H>>) -> Self {
            Self { logger, heater }
        }
    }
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
