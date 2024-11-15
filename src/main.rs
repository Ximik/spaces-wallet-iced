mod app;
mod screen;
mod store;

use app::App;

pub fn main() -> iced::Result {
    let icon =
        iced::window::icon::from_rgba(include_bytes!("../assets/icon.rgba").to_vec(), 64, 64)
            .expect("Failed to load icon");

    App::run(iced::window::Settings {
        size: (800.0, 500.0).into(),
        icon: Some(icon),
        ..Default::default()
    })
}
