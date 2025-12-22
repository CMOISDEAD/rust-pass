use iced::{
    Event, Length, Subscription, Task, Theme, event, exit,
    keyboard::{self, Key, Modifiers},
    widget::{Container, column, container, operation::focus, text, text_input},
    window::{self, Settings},
};
use std::collections::HashMap;

use crate::{methods, utils};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    PasswordList,
    KeyValueView,
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::PasswordList
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    MoveDown,
    MoveUp,
    Back,
    Enter,
    FocusInput,
    Quit,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(HashMap<String, String>),
    ContentChange(String),
    Submit,
    PasswordLoaded(String),
    KeyPressed(Event),
    Error(String),
}

#[derive(Default)]
pub struct AppState {
    mode: ViewMode,
    index: usize,
    loading: bool,
    content: String,
    entries: HashMap<String, String>,
}

fn key_to_action(key: &Key, modifiers: Modifiers) -> Option<Action> {
    match key {
        keyboard::Key::Named(keyboard::key::Named::Escape) => Some(Action::Back),

        keyboard::Key::Named(keyboard::key::Named::Tab) => Some(Action::FocusInput),

        keyboard::Key::Character(c) if modifiers.control() => match c.as_str() {
            "j" => Some(Action::MoveDown),
            "k" => Some(Action::MoveUp),
            "h" => Some(Action::Back),
            "l" => Some(Action::Enter),
            "q" => Some(Action::Quit),
            _ => None,
        },

        _ => None,
    }
}

impl AppState {
    pub fn title(&self) -> String {
        "pass-rust".into()
    }

    pub fn theme(&self) -> Theme {
        Theme::Moonfly
    }

    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                mode: ViewMode::PasswordList,
                index: 0,
                loading: true,
                ..Default::default()
            },
            Task::perform(async { methods::get_password_files() }, Message::Loaded),
        )
    }

    fn filtered_keys(&self) -> Vec<&String> {
        let mut keys: Vec<_> = self
            .entries
            .keys()
            .filter(|k| k.contains(&self.content))
            .collect();

        keys.sort();
        keys
    }

    fn selected_key(&self) -> Option<&String> {
        self.filtered_keys().get(self.index).copied()
    }

    fn handle_action(&mut self, action: Action) -> Task<Message> {
        let len = self.filtered_keys().len();

        match action {
            Action::MoveDown if len > 0 => {
                self.index = (self.index + 1) % len;
                Task::none()
            }

            Action::MoveUp if len > 0 => {
                self.index = (self.index + len - 1) % len;
                Task::none()
            }

            Action::Enter => {
                let Some(key) = self.selected_key() else {
                    return Task::none();
                };

                match self.mode {
                    ViewMode::PasswordList => {
                        let name = self.entries[key].clone();
                        Task::perform(async move { methods::get_password_content(&name) }, |res| {
                            match res {
                                Ok(p) => Message::PasswordLoaded(p),
                                Err(e) => Message::Error(e),
                            }
                        })
                    }

                    ViewMode::KeyValueView => {
                        let value = self.entries[key].clone();
                        Task::perform(async {}, |_| Message::PasswordLoaded(value))
                    }
                }
            }

            Action::Back => match self.mode {
                ViewMode::KeyValueView => {
                    self.mode = ViewMode::PasswordList;
                    Task::perform(async { methods::get_password_files() }, Message::Loaded)
                }
                ViewMode::PasswordList => exit(),
            },

            Action::FocusInput => focus("input"),
            Action::Quit => exit(),
            _ => Task::none(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Loaded(entries) => {
                self.mode = ViewMode::PasswordList;
                self.entries = entries;
                self.index = 0;
                self.loading = false;
                focus("input")
            }

            Message::ContentChange(value) => {
                self.content = value;
                self.index = 0;
                Task::none()
            }

            Message::Submit => self.handle_action(Action::Enter),

            Message::PasswordLoaded(password) => match methods::parse_kv(&password) {
                methods::ParsedPassword::KeyValue(map) => {
                    self.index = 0;
                    self.entries = map;
                    self.content.clear();
                    self.mode = ViewMode::KeyValueView;
                    focus("input")
                }

                methods::ParsedPassword::Raw(value) => {
                    let _ = utils::copy_to_clipboard(&value);
                    exit()
                }
            },

            Message::KeyPressed(event) => {
                let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event
                else {
                    return Task::none();
                };

                let Some(action) = key_to_action(&key, modifiers) else {
                    return Task::none();
                };

                self.handle_action(action)
            }

            Message::Error(e) => {
                eprintln!("Error: {}", e);
                exit()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::KeyPressed)
    }

    pub fn view(&self) -> Container<'_, Message> {
        let input = text_input("Search password", &self.content)
            .on_input(Message::ContentChange)
            .on_submit(Message::Submit)
            .id("input");

        let results = self
            .filtered_keys()
            .iter()
            .enumerate()
            .fold(column![], |col, (i, key)| {
                let selected = i == self.index;

                let row = container(text(*key).size(16).style(move |theme: &Theme| {
                    iced::widget::text::Style {
                        color: Some(if selected {
                            theme.palette().primary.into()
                        } else {
                            theme.palette().text.into()
                        }),
                    }
                }))
                .padding(6)
                .style(move |theme: &Theme| {
                    if selected {
                        iced::widget::container::Style {
                            background: Some(theme.palette().primary.inverse().into()),
                            text_color: Some(theme.palette().text.into()),
                            ..Default::default()
                        }
                    } else {
                        Default::default()
                    }
                });

                col.push(row)
            });
        container(column![input, results].spacing(10))
            .padding(20)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

pub fn get_window_config() -> Settings {
    window::Settings {
        size: iced::Size::new(400.0, 200.0),
        level: window::Level::AlwaysOnTop,
        position: window::Position::Centered,
        ..Default::default()
    }
}
