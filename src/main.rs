mod methods;
mod utils;

use iced::{
    Length, Task, Theme, exit,
    widget::{Container, column, container, operation::focus, text, text_input},
    window,
};
use methods::ParsedPassword;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    PasswordList, // list all pass elements
    KeyValueView, // list each key in password file
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::PasswordList
    }
}

#[derive(Default)]
struct AppState {
    mode: ViewMode,
    loading: bool,
    content: String,
    all_entries: HashMap<String, String>,
    filtered: HashMap<String, String>,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(HashMap<String, String>),
    ContentChange(String),
    Submit,
    PasswordLoaded(String),
    Error(String),
}

impl AppState {
    fn title(&self) -> String {
        "pass-rust".into()
    }

    fn theme(&self) -> Theme {
        Theme::Moonfly
    }

    fn new() -> (Self, Task<Message>) {
        (
            Self {
                mode: ViewMode::PasswordList,
                loading: true,
                ..Default::default()
            },
            Task::perform(async { methods::get_password_files() }, Message::Loaded),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Loaded(entries) => {
                self.mode = ViewMode::PasswordList;
                self.all_entries = entries.clone();
                self.filtered = entries;
                self.loading = false;
                focus("input")
            }

            Message::ContentChange(value) => {
                self.content = value.clone();

                self.filtered = self
                    .all_entries
                    .clone()
                    .into_iter()
                    .filter(|(k, _v)| k.contains(&value))
                    .collect();

                Task::none()
            }

            Message::Submit => {
                let Some((key, value)) = self.filtered.iter().next() else {
                    return Task::none();
                };

                match self.mode {
                    ViewMode::PasswordList => {
                        let name = key.clone();

                        Task::perform(async move { methods::get_password_content(&name) }, |res| {
                            match res {
                                Ok(p) => Message::PasswordLoaded(p),
                                Err(e) => Message::Error(e),
                            }
                        })
                    }

                    ViewMode::KeyValueView => {
                        let a = value.to_string();
                        Task::perform(async {}, |_| Message::PasswordLoaded(a))
                    }
                }
            }

            Message::PasswordLoaded(password) => {
                match methods::parse_kv(&password) {
                    ParsedPassword::KeyValue(map) => {
                        self.all_entries = map.clone();
                        self.filtered = map;
                        self.content.clear();
                        self.mode = ViewMode::KeyValueView;
                        focus("input")
                    }

                    ParsedPassword::Raw(value) => {
                        let _ = utils::copy_to_clipboard(&value);
                        exit()
                    }
                }
            }

            Message::Error(e) => {
                println!("something goes wrong");
                println!("Error: {}", e);
                exit()
            }
        }
    }

    fn view(&self) -> Container<'_, Message> {
        let input = text_input("Search password", &self.content)
            .on_input(Message::ContentChange)
            .on_submit(Message::Submit)
            .id("input");

        let results = self
            .filtered
            .iter()
            .take(6)
            .fold(column![], |col, entry| col.push(text(entry.0)));

        let content = column![input, results].spacing(10);

        container(content)
            .padding(20)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

fn main() -> iced::Result {
    let window = window::Settings {
        size: iced::Size::new(400.0, 200.0),
        level: window::Level::AlwaysOnTop,
        position: window::Position::Centered,
        ..Default::default()
    };

    iced::application(AppState::new, AppState::update, AppState::view)
        .window(window)
        .theme(AppState::theme)
        .title(AppState::title)
        .run()
}
