use std::fmt::Display;

use dirk_macros::{component, provides};

fn main() {
    let user_name = "Bob".to_string();

    let component = <DirkApplicationComponent as dirk::DirkComponent<_>>::builder()
        .application_name("SuperImportantApplication".to_string())
        .user_name(user_name.clone())
        .build();

    let app = component.application();
    app.run();
}

#[component(
    [
        application_name: cloned_instance_bind(T),
        user_name: cloned_instance_bind(T),
        application: static_bind(Application<T>) [application_name, user_name]
    ]
)]
trait ApplicationComponent<T: Display + Clone + 'static> {
    fn application(&self) -> Application<T>;
}

struct Application<T: Display + Clone + 'static> {
    application_name: T,
    user_name: T,
}

#[provides]
impl<T: Display + Clone + 'static> Application<T> {
    fn new(application_name: T, user_name: T) -> Self {
        Self {
            application_name,
            user_name,
        }
    }
}

impl<T: Display + Clone + 'static> Application<T> {
    fn run(&self) {
        println!(
            "Application {} running under user {}",
            self.application_name, self.user_name
        );
    }
}
