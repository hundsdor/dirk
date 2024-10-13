use std::{collections::HashMap, fmt::Display};

use dirk::{component, provides};

fn main() {
    let user_name = "Bob".to_string();

    let component = DirkApplicationComponent::builder()
        .cookies(MandatoryCookies {})
        .user_name(user_name.clone())
        .build();

    let app = component.application();
    app.run();
}

#[component(
    cookies: cloned_instance_bind(C),
    user_name: cloned_instance_bind(U),
    application: static_bind(Application<C, U>) [cookies, user_name]
)]
trait ApplicationComponent<C: Cookies + Clone + 'static, U: Display + Clone + 'static> {
    fn cookies(&self) -> impl Cookies + Clone;
    fn application(&self) -> Application<C, U>;
}

struct Application<C: Cookies + Clone + 'static, U: Display + Clone + 'static> {
    cookies: C,
    user_name: U,
}

#[provides]
impl<C: Cookies + Clone + 'static, U: Display + Clone + 'static> Application<C, U> {
    fn new(cookies: C, user_name: U) -> Self {
        Self { cookies, user_name }
    }
}

impl<C: Cookies + Clone + 'static, U: Display + Clone + 'static> Application<C, U> {
    fn run(&self) {
        println!(
            "Application running under user {} with cookies {:?}",
            self.user_name,
            self.cookies.get_cookies()
        );
    }
}

trait Cookies {
    fn get_cookies(&self) -> HashMap<String, String>;
}

struct MandatoryCookies {}

impl Cookies for MandatoryCookies {
    fn get_cookies(&self) -> HashMap<String, String> {
        let mut ret = HashMap::new();
        ret.insert("sess".to_string(), "1234567890".to_string());
        ret
    }
}
