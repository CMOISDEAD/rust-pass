mod methods;
mod ui;
mod utils;

use ui::AppState;

fn main() -> iced::Result {
    iced::application(AppState::new, AppState::update, AppState::view)
        .window(ui::get_window_config())
        .subscription(AppState::subscription)
        .theme(AppState::theme)
        .title(AppState::title)
        .run()
}
