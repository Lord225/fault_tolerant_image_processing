use std::error::Error;
use database::repositories::task::InsertableTaskTree;

use clap::Parser;

use engine::run;
use iced::{
    widget::Button, widget::Column, widget::Container, Element, Length, Sandbox, Settings, widget::Text,
};
use iced::theme::{self, Theme};

use nfd::Response;
use std::path::PathBuf;

mod processing;
mod database;
mod tests_common;
mod temp;
mod engine;


use processing::job;

use crate::temp::from_temp;

#[derive(Parser, Debug)]
struct Args {
    /// Reset database
    #[clap(short, long, default_value_t=false )]
    reset: bool,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    // init .env
    dotenvy::dotenv().ok();
    dotenvy::from_filename(".env.local").ok();

    // init logger
    env_logger::init();

    // init cli
    let args = Args::parse();

    // reset db
    if args.reset {
        database::common::try_reset_database();
    }

    let mut db = database::common::open_connection()?;

    database::migration::run_migrations(&mut db);

    db.insert_new_task_tree(
    &InsertableTaskTree {
            data: None,
            status: database::schema::Status::Pending,
            params: job::JobType::new_overlay(10, 10),

            parent_tasks: vec![
                InsertableTaskTree {
                    data: None,
                    status: database::schema::Status::Pending,
                    params: job::JobType::new_resize(512, 512),
                    parent_tasks: vec![
                        InsertableTaskTree::input(&from_temp("in1.jpg")),
                    ]
                }, 
                InsertableTaskTree::input(&from_temp("in2.jpg")),
            ]
        }
    )?;


    run();

    //Styling::run(Settings::default())?;
    MyApp::run(Settings::default())?;

    Ok(())
}

// stan aplikacji - iced 
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
}

// Eventy aplikacji - iced
// #[derive(Debug, Clone)]
// enum Message {
//     InputChanged(String),
//     ButtonPressed,
//     SliderChanged(f32),
//     CheckboxToggled(bool),
//     TogglerToggled(bool),
// }

// impl Sandbox for Styling {
//     type Message = Message;

//     fn new() -> Self {
//         Styling::default()
//     }

//     fn title(&self) -> String {
//         String::from("Styling - Iced")
//     }


//     // update stanu aplikacji
//     fn update(&mut self, message: Message) {
//         match message {
//             Message::InputChanged(value) => self.input_value = value,
//             Message::ButtonPressed => {}
//             Message::SliderChanged(value) => self.slider_value = value,
//             Message::CheckboxToggled(value) => self.checkbox_value = value,
//             Message::TogglerToggled(value) => self.toggler_value = value,
//         }
//     }

//     fn view(&self) -> Element<Message> {
//         // let choose_theme = [ThemeType::Light, ThemeType::Dark]
//         //     .iter()
//         //     .fold(
//         //         column![text("Choose a theme:")].spacing(10),
//         //         |column, theme| {
//         //             column.push(radio(
//         //                 format!("{theme:?}"),
//         //                 *theme,
//         //                 Some(match self.theme {
//         //                     Theme::Light => ThemeType::Light,
//         //                     Theme::Dark => ThemeType::Dark,
//         //                     Theme::Custom { .. } => ThemeType::Light,
//         //                 }),
//         //                 Message::ThemeChanged,
//         //             ))
//         //         },
//         //     );

//         let text_input = text_input("Type something...", &self.input_value)
//             .on_input(Message::InputChanged)
//             .padding(10)
//             .size(20);

//         let button = button("Submit")
//             .padding(10)
//             .on_press(Message::ButtonPressed);

//         let slider = slider(0.0..=100.0, self.slider_value, Message::SliderChanged);

//         let progress_bar = progress_bar(0.0..=100.0, self.slider_value);

//         let scrollable = scrollable(
//             column!["Scroll me!", vertical_space(800), "You did it!"].width(Length::Fill),
//         )
//         .height(100);

//         let checkbox = checkbox("Check me!", self.checkbox_value, Message::CheckboxToggled);

//         let toggler = toggler(
//             String::from("Toggle me!"),
//             self.toggler_value,
//             Message::TogglerToggled,
//         )
//         .width(Length::Shrink)
//         .spacing(10);

//         let content = column![
//             row![text_input, button].spacing(10),
//             slider,
//             progress_bar,
//             row![
//                 scrollable,
//                 vertical_rule(38),
//                 column![checkbox, toggler].spacing(20)
//             ]
//             .spacing(10)
//             .height(100)
//             .align_items(Alignment::Center),
//         ]
//         .spacing(20)
//         .padding(20)
//         .max_width(600);

//         container(content)
//             .width(Length::Fill)
//             .height(Length::Fill)
//             .center_x()
//             .center_y()
//             .into()
//     }

//     fn theme(&self) -> Theme {
//         self.theme.clone()
//     }
// }

struct MyApp {
    selected_file: Option<PathBuf>,
}

#[derive(Debug, Clone)]
enum Message {
    FileSelected(Option<PathBuf>),
    OpenButtonPressed,
}

impl Sandbox for MyApp {
    type Message = Message;

    fn new() -> Self {
        MyApp {
            selected_file: None,
        }
    }

    fn title(&self) -> String {
        String::from("File Chooser")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::FileSelected(path) => {
                self.selected_file = path;
            }
            Message::OpenButtonPressed => {
                if let Ok(Response::Okay(file_path)) = nfd::open_file_dialog(None, None) {
                    self.selected_file = Some(PathBuf::from(file_path));
                } else {
                    self.selected_file = None;
                }
            }
        }
    }

    fn view(self: &MyApp) -> Element<Message> {
        let open_button = Button::new(Text::new("Open"))
            .on_press(Message::OpenButtonPressed);

        let content = match &self.selected_file {
            Some(path) => Column::new()
                .spacing(20)
                .push(Text::new("Selected file:").size(30))
                .push(Text::new(path.to_string_lossy())),
            None => Column::new()
                .spacing(20)
                .push(Text::new("No file selected").size(30)),
        };

        Container::new(
            Column::new()
                .spacing(20)
                .push(open_button)
                .push(content),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}
