use std::error::Error;
use database::repositories::task::InsertableTaskTree;
use dotenvy;
use clap::Parser;

use iced::theme::{self, Theme};
use iced::widget::{
    button, checkbox, column, container, horizontal_rule, progress_bar, radio, row, scrollable,
    slider, text, text_input, toggler, vertical_rule, vertical_space,
};
use iced::{Alignment, Color, Element, Length, Sandbox, Settings};

mod processing;
mod database;

use processing::job;

use crate::processing::worker::worker1::Worker1Job;
use crate::processing::worker::worker2::Worker2Job;

#[derive(Parser, Debug)]
struct Args {
    /// Reset database
    #[clap(short, long, default_value_t=false )]
    reset: bool,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    // init .env
    dotenvy::dotenv().ok();

    // init cli
    let args = Args::parse();

    // reset db
    if args.reset {
        database::common::reset_database()?;
    }

    let mut db = database::common::open_connection()?;

    database::migration::run_migrations(&mut db);

    db.insert_new_task_tree(
    &InsertableTaskTree {
            data: Some("Main Task".to_string()),
            status: database::schema::Status::Pending,
            params: job::JobType::new_blur(0.0),

            parent_tasks: vec![
                InsertableTaskTree {
                    data: Some("Subtask 1".to_string()),
                    status: database::schema::Status::Pending,
                    params: job::JobType::new_blur(0.0),
                    parent_tasks: vec![
                        InsertableTaskTree {
                            data: Some("Subtask for subtask 1".to_string()),
                            status: database::schema::Status::Pending,
                            params: job::JobType::new_blur(0.0),
        
                            parent_tasks: vec![]
                        },
                    ]
                }, 
                InsertableTaskTree {
                    data: Some("Subtask 2".to_string()),
                    status: database::schema::Status::Pending,
                    params: job::JobType::new_resize(100,100),

                    parent_tasks: vec![]
                }
            ]
        }
    )?;
    
    dbg!(db.get_task_by_id(1)?);
    dbg!(db.get_runnable_tasks()?);
    
    dbg!(db.claim_all_runnable_tasks::<Worker2Job>()?);
    dbg!(db.claim_all_runnable_tasks::<Worker1Job>()?);
    
    Styling::run(Settings::default())?;

    Ok(())
}

#[derive(Default)]
struct Styling {
    theme: Theme,
    input_value: String,
    slider_value: f32,
    checkbox_value: bool,
    toggler_value: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ThemeType {
    Light,
    Dark,
    Custom,
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(ThemeType),
    InputChanged(String),
    ButtonPressed,
    SliderChanged(f32),
    CheckboxToggled(bool),
    TogglerToggled(bool),
}

impl Sandbox for Styling {
    type Message = Message;

    fn new() -> Self {
        Styling::default()
    }

    fn title(&self) -> String {
        String::from("Styling - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeChanged(theme) => {
                self.theme = match theme {
                    ThemeType::Light => Theme::Light,
                    ThemeType::Dark => Theme::Dark,
                    ThemeType::Custom => Theme::custom(theme::Palette {
                        background: Color::from_rgb(1.0, 0.9, 1.0),
                        text: Color::BLACK,
                        primary: Color::from_rgb(0.5, 0.5, 0.0),
                        success: Color::from_rgb(0.0, 1.0, 0.0),
                        danger: Color::from_rgb(1.0, 0.0, 0.0),
                    }),
                }
            }
            Message::InputChanged(value) => self.input_value = value,
            Message::ButtonPressed => {}
            Message::SliderChanged(value) => self.slider_value = value,
            Message::CheckboxToggled(value) => self.checkbox_value = value,
            Message::TogglerToggled(value) => self.toggler_value = value,
        }
    }

    fn view(&self) -> Element<Message> {
        let choose_theme = [ThemeType::Light, ThemeType::Dark, ThemeType::Custom]
            .iter()
            .fold(
                column![text("Choose a theme:")].spacing(10),
                |column, theme| {
                    column.push(radio(
                        format!("{theme:?}"),
                        *theme,
                        Some(match self.theme {
                            Theme::Light => ThemeType::Light,
                            Theme::Dark => ThemeType::Dark,
                            Theme::Custom { .. } => ThemeType::Custom,
                        }),
                        Message::ThemeChanged,
                    ))
                },
            );

        let text_input = text_input("Type something...", &self.input_value)
            .on_input(Message::InputChanged)
            .padding(10)
            .size(20);

        let button = button("Submit")
            .padding(10)
            .on_press(Message::ButtonPressed);

        let slider = slider(0.0..=100.0, self.slider_value, Message::SliderChanged);

        let progress_bar = progress_bar(0.0..=100.0, self.slider_value);

        let scrollable = scrollable(
            column!["Scroll me!", vertical_space(800), "You did it!"].width(Length::Fill),
        )
        .height(100);

        let checkbox = checkbox("Check me!", self.checkbox_value, Message::CheckboxToggled);

        let toggler = toggler(
            String::from("Toggle me!"),
            self.toggler_value,
            Message::TogglerToggled,
        )
        .width(Length::Shrink)
        .spacing(10);

        let content = column![
            choose_theme,
            horizontal_rule(38),
            row![text_input, button].spacing(10),
            slider,
            progress_bar,
            row![
                scrollable,
                vertical_rule(38),
                column![checkbox, toggler].spacing(20)
            ]
            .spacing(10)
            .height(100)
            .align_items(Alignment::Center),
        ]
        .spacing(20)
        .padding(20)
        .max_width(600);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
