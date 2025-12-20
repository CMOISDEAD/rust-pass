mod methods;
mod utils;

use iced::{
    Length, Task, Theme, exit,
    widget::{Container, column, combo_box, container, text},
    window,
};

#[derive(Default)]
struct AppState {
    loading: bool,
    entries: combo_box::State<methods::PasswordFile>,
    selected: Option<methods::PasswordFile>,
}

#[derive(Debug, Clone)]
enum Message {
    Load,
    Loaded(Vec<methods::PasswordFile>),
    Select(methods::PasswordFile),
    PasswordLoaded(String),
    Error(String),
}

impl AppState {
    fn title(&self) -> String {
        String::from("pass-rust")
    }

    fn theme(&self) -> Theme {
        Theme::TokyoNight
    }

    fn new() -> (Self, Task<Message>) {
        (
            Self {
                loading: true,
                ..Self::default()
            },
            Task::perform(async { methods::get_password_files() }, Message::Loaded),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Load => {
                self.loading = true;
                Task::perform(async { methods::get_password_files() }, Message::Loaded)
            }
            Message::Loaded(entries) => {
                self.entries = combo_box::State::new(entries);
                self.loading = false;
                Task::none()
            }
            Message::Select(value) => {
                self.selected = Some(value.clone());

                Task::perform(
                    async move { methods::get_password(&value.relative) },
                    |result| match result {
                        Ok(password) => Message::PasswordLoaded(password),
                        Err(err) => Message::Error(err),
                    },
                )
            }
            Message::PasswordLoaded(password) => {
                let _ = utils::copy_to_clipboard(password).ok();
                exit()
            }
            Message::Error(_) => todo!(),
        }
    }

    fn view(&self) -> Container<'_, Message> {
        let content = column![
            if self.loading {
                text("Loading passwords...")
            } else {
                text("Passwords loaded")
            },
            combo_box(
                &self.entries,
                "Select your password",
                self.selected.as_ref(),
                Message::Select,
            ),
        ]
        .spacing(10);

        container(content)
            .padding(20)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

fn main() -> iced::Result {
    let window = window::Settings {
        size: iced::Size::new(400.0, 400.0),
        level: window::Level::AlwaysOnTop,
        position: window::Position::Centered,
        resizable: (true),
        decorations: (true),
        ..Default::default()
    };

    iced::application(AppState::new, AppState::update, AppState::view)
        .window(window)
        .theme(AppState::theme)
        .title(AppState::title)
        .run()
}
