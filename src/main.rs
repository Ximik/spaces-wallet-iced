mod app;
mod screen;
mod store;

use app::App;

mod icon {
    include!("../assets/icons.rs");
}

pub fn main() -> iced::Result {
    App::run()
}
