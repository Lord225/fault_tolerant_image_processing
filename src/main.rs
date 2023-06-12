use database::common::Database;
use database::repositories::task::{InsertableTaskTree, Task};
use iced::{Color, Rectangle, Renderer};
use log::debug;
use std::{error::Error, vec};

use clap::Parser;

use engine::run;
use iced::alignment::{Horizontal, Vertical};
use iced::theme::{Palette, Theme};
use iced::widget::{container, pick_list, slider, Row, Scrollable};

use iced::{
    widget::column, widget::row, widget::Button, widget::Column, widget::Container, widget::Text,
    Element, Length, Sandbox, Settings,
};

use nfd::Response;
use std::path::PathBuf;

mod database;
mod engine;
mod processing;
mod temp;
mod tests_common;

use processing::job::{self, BlurJob, BrightnessJob, CropJob, JobType, ResizeJob};

use crate::temp::from_temp;

#[derive(Parser, Debug)]
struct Args {
    /// Reset database
    #[clap(short, long, default_value_t = false)]
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

    db.insert_new_task_tree(&InsertableTaskTree {
        data: None,
        status: database::schema::Status::Pending,
        params: job::JobType::new_overlay(10, 10),

        parent_tasks: vec![
            InsertableTaskTree {
                data: None,
                status: database::schema::Status::Pending,
                params: job::JobType::new_resize(512, 512),
                parent_tasks: vec![InsertableTaskTree::input(&from_temp("in1.jpg"))],
            },
            InsertableTaskTree::input(&from_temp("in2.jpg")),
        ],
    })?;

    run();

    //Styling::run(Settings::default())?;
    MyApp::run(Settings::default())?;

    Ok(())
}

// // stan aplikacji - iced
// #[derive(Default)]
// struct Styling {
//     theme: Theme,
//     input_value: String,
//     slider_value: f32,
//     checkbox_value: bool,
//     toggler_value: bool,
// }

// #[derive(Debug, PartialEq, Eq, Clone, Copy)]
// enum ThemeType {
//     Light,
//     Dark,
// }

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

#[derive(Debug, Clone)]
struct TaskElement {
    id: i64,
    name: String,
    status: database::schema::Status,
}

impl TaskElement {
    fn new() -> Self {
        TaskElement {
            id: 0,
            name: "Hello".into(),
            //random
            status: match rand::random::<u8>() % 4 {
                0 => database::schema::Status::Pending,
                1 => database::schema::Status::Running,
                2 => database::schema::Status::Failed,
                3 => database::schema::Status::Completed,
                _ => database::schema::Status::Pending,
            },
        }
    }
}

impl<'a, Message: 'a> From<TaskElement> for iced::Element<'a, Message> {
    fn from(task: TaskElement) -> Self {
        let TaskElement { id, name, status } = task;

        // display TaskElement - use diffrent backgorund for diffrent status
        // grey
        const PENDING: Color = Color::from_rgb(0.8, 0.8, 0.8);
        // blue - 104 149 212
        const RUNNING: Color = Color::from_rgb(0.4, 0.6, 0.8);
        // red - 204 82 86
        const FAILED: Color = Color::from_rgb(0.8, 0.3, 0.3);
        // green 69 214 90
        const COMPLETED: Color = Color::from_rgb(0.3, 0.8, 0.3);

        let background = match status {
            database::schema::Status::Pending => PENDING,
            database::schema::Status::Running => RUNNING,
            database::schema::Status::Failed => FAILED,
            database::schema::Status::Completed => COMPLETED,
        };

        struct RectangleProgram(Color, String);
        use iced::widget::canvas::Program;

        impl<Message> Program<Message> for RectangleProgram {
            type State = ();

            fn draw(
                &self,
                _: &Self::State,
                _: &Theme,
                bounds: Rectangle,
                _: iced::widget::canvas::Cursor,
            ) -> Vec<iced::widget::canvas::Geometry> {
                let rectangle = iced::widget::canvas::Path::rectangle(iced::Point::ORIGIN, bounds.size());
                let background = iced::widget::canvas::Fill {
                    style: iced::widget::canvas::Style::Solid(self.0),
                    ..iced::widget::canvas::Fill::default()
                };

                let mut frame = iced::widget::canvas::Frame::new(bounds.size());
                frame.fill(
                    &rectangle,
                    background,
                );

                // draw text
                let text = iced::widget::canvas::Text {
                    content: self.1.clone(),
                    position: iced::Point::new(0.0, 0.0),
                    size: 20.0,
                    color: Color::BLACK,
                    ..iced::widget::canvas::Text::default()
                };

                frame.fill_text(
                    text
                );

                vec![frame.into_geometry()]
            }
        }

        let canvas = iced::widget::canvas::Canvas::new(RectangleProgram(background, name))
            .width(Length::Fill)
            .height(50); 


        Container::new(row![canvas]).into()
    }
}

impl From<Task> for TaskElement {
    fn from(task: Task) -> Self {
        TaskElement {
            id: task.task_id,
            name: format!("{:?}", task),
            status: task.status,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AvalibleActions {
    Input,
    Crop,
    Brighten,
    Resize,
    Blur,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CropActions {
    X,
    Y,
    Width,
    Height,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BrightenActions {
    Value,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResizeActions {
    Width,
    Height,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlurActions {
    Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SliderChangedAction {
    Crop(CropActions),
    Brighten(BrightenActions),
    Resize(ResizeActions),
    Blur(BlurActions),
}

static ALL_ACTIONS: &[AvalibleActions] = &[
    AvalibleActions::Crop,
    AvalibleActions::Brighten,
    AvalibleActions::Resize,
    AvalibleActions::Blur,
    AvalibleActions::Input,
];

// display
impl std::fmt::Display for AvalibleActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AvalibleActions::Crop => write!(f, "Crop"),
            AvalibleActions::Brighten => write!(f, "Brighten"),
            AvalibleActions::Resize => write!(f, "Resize"),
            AvalibleActions::Blur => write!(f, "Blur"),
            AvalibleActions::Input => write!(f, "Input"),
        }
    }
}

struct MyApp {
    selected_file: Option<PathBuf>,
    items: Vec<TaskElement>,
    panel_state: JobType,
    current_action: AvalibleActions,
    db: Database,
}

impl MyApp {
    fn get_all_tasks(&mut self) {
        self.items = self.db.get_all_tasks().unwrap().into_iter().map(|x| x.into()).collect();
    }
}

//
// APP layout
//
impl MyApp {
    fn action_to_panel(&self, action: AvalibleActions) -> Element<'_, Message> {
        match (action, self.panel_state) {
            (AvalibleActions::Crop, JobType::Crop(val)) => {
                // crop has: x, y, width, height
                let x = slider(0.0..=100.0, val.0 as f32, |x| {
                    Message::SliderChanged(x, SliderChangedAction::Crop(CropActions::X))
                });
                let y = slider(0.0..=100.0, val.1 as f32, |x| {
                    Message::SliderChanged(x, SliderChangedAction::Crop(CropActions::Y))
                });

                let width = slider(0.0..=100.0, val.2 as f32, |x| {
                    Message::SliderChanged(x, SliderChangedAction::Crop(CropActions::Width))
                });
                let height = slider(0.0..=100.0, val.3 as f32, |x| {
                    Message::SliderChanged(x, SliderChangedAction::Crop(CropActions::Height))
                });

                // put them in rows

                column![
                    row![Text::new("x"), x, Text::new("y"), y].spacing(5),
                    row![Text::new("width"), width, Text::new("height"), height].spacing(5),
                ]
                .spacing(5)
                .into()
            }
            (AvalibleActions::Brighten, JobType::Brightness(x)) => {
                let value = slider(0.0..=100.0, x.0, |x| {
                    Message::SliderChanged(x, SliderChangedAction::Brighten(BrightenActions::Value))
                });

                column![row![Text::new("value"), value].spacing(5),]
                    .spacing(5)
                    .into()
            }
            (AvalibleActions::Resize, JobType::Resize(x)) => {
                let width = slider(0.0..=100.0, x.0 as f32, |x| {
                    Message::SliderChanged(x, SliderChangedAction::Resize(ResizeActions::Width))
                });

                let height = slider(0.0..=100.0, x.1 as f32, |x| {
                    Message::SliderChanged(x, SliderChangedAction::Resize(ResizeActions::Height))
                });

                column![row![Text::new("width"), width, Text::new("height"), height].spacing(5),]
                    .spacing(5)
                    .into()
            }
            (AvalibleActions::Blur, JobType::Blur(x)) => {
                let value = slider(0.0..=100.0, x.0, |x| {
                    Message::SliderChanged(x, SliderChangedAction::Blur(BlurActions::Value))
                });

                column![row![Text::new("amount"), value].spacing(5),]
                    .spacing(5)
                    .into()
            }
            (AvalibleActions::Input, _) => {
                let open_button =
                    Button::new(Text::new("Open")).on_press(Message::OpenButtonPressed);
                let message = match &self.selected_file {
                    Some(path) => Column::new()
                        .spacing(20)
                        .push(Text::new("Selected file:").size(30))
                        .push(Text::new(path.to_string_lossy())),
                    None => Column::new()
                        .spacing(20)
                        .push(Text::new("No file selected").size(30)),
                };

                column![row![open_button, message,]].into()
            }
            (_, _) => column![].into(),
        }
    }
}

//
// STATE UPDATE - MESSAGE HANDLING
//
#[derive(Debug, Clone)]
enum Message {
    FileSelected(Option<PathBuf>),
    OpenButtonPressed,
    AddItem,
    ActionPickChanged(AvalibleActions),
    SliderChanged(f32, SliderChangedAction),
    ConfirmJob,
}
impl MyApp {
    fn update_state_on_slider(&mut self, slider: SliderChangedAction, value: f32) {
        match slider {
            SliderChangedAction::Crop(a) => {
                if let JobType::Crop(t) = &mut self.panel_state {
                    match a {
                        CropActions::X => {
                            t.0 = value as u32;
                        }
                        CropActions::Y => {
                            t.1 = value as u32;
                        }
                        CropActions::Width => {
                            t.2 = value as u32;
                        }
                        CropActions::Height => {
                            t.3 = value as u32;
                        }
                    }
                } else {
                    self.panel_state = JobType::Crop(CropJob(0, 0, 0, 0));
                }
            }
            SliderChangedAction::Brighten(a) => {
                if let JobType::Brightness(t) = &mut self.panel_state {
                    match a {
                        BrightenActions::Value => {
                            t.0 = value;
                        }
                    }
                } else {
                    self.panel_state = JobType::Brightness(BrightnessJob(0.0));
                }
            }
            SliderChangedAction::Resize(a) => {
                if let JobType::Resize(t) = &mut self.panel_state {
                    match a {
                        ResizeActions::Width => {
                            t.0 = value as u32;
                        }
                        ResizeActions::Height => {
                            t.1 = value as u32;
                        }
                    }
                } else {
                    self.panel_state = JobType::Resize(ResizeJob(0, 0));
                }
            }
            SliderChangedAction::Blur(a) => {
                if let JobType::Blur(t) = &mut self.panel_state {
                    match a {
                        BlurActions::Value => {
                            t.0 = value;
                        }
                    }
                } else {
                    self.panel_state = JobType::Blur(BlurJob(0.0));
                }
            }
        }
    }
}

impl Sandbox for MyApp {
    type Message = Message;

    fn new() -> Self {
        MyApp {
            selected_file: None,
            items: vec![],
            panel_state: JobType::Crop(CropJob(0, 0, 0, 0)),
            current_action: AvalibleActions::Crop,
            db: database::common::try_open_connection(),
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
            Message::AddItem => {
                self.items.push(TaskElement::new());
            }
            Message::ActionPickChanged(action) => {
                self.current_action = action;
                match action {
                    AvalibleActions::Crop => self.panel_state = JobType::Crop(CropJob(0, 0, 0, 0)),
                    AvalibleActions::Brighten => {
                        self.panel_state = JobType::Brightness(BrightnessJob(0.0))
                    }
                    AvalibleActions::Resize => self.panel_state = JobType::Resize(ResizeJob(0, 0)),
                    AvalibleActions::Blur => self.panel_state = JobType::Blur(BlurJob(0.0)),
                    AvalibleActions::Input => self.panel_state = JobType::Crop(CropJob(0, 0, 0, 0)),
                }
            }
            Message::SliderChanged(value, w) => {
                self.update_state_on_slider(w, value);
                debug!("Slider {:?} changed to {:?} ", w, value);
            }
            Message::ConfirmJob => {
                debug!("Confirming job - {:?}", self.panel_state);
            }
        }
    }

    fn view(self: &MyApp) -> Element<Message> {
        let scrollable_content = self
            .items
            .iter()
            .fold(Column::new().spacing(10), |column, item| {
                column.push(item.clone())
            });

        let scrollable = Scrollable::new(scrollable_content)
            .width(Length::Fill)
            .height(Length::Fill);

        let add_button = Button::new(Text::new("Add Item")).on_press(Message::AddItem);

        let column = Column::new()
            .push(scrollable)
            .push(add_button)
            .width(Length::Fill)
            .height(Length::Fill);

        // pick list with avalible actions (Blur, Resize, Crop, Brighten)
        let pick_list = pick_list::PickList::new(
            ALL_ACTIONS,
            Some(self.current_action),
            Message::ActionPickChanged,
        )
        .width(Length::Fill);

        let action_panel = self.action_to_panel(self.current_action);

        let show_job_button = Button::new(Text::new("Add task"))
            .on_press(Message::ConfirmJob)
            .width(Length::Fill);

        // Container with two columns - left with scrollable list of items, right with pick list & buttons
        Container::new(
            row![
                column![column].width(300),
                column![pick_list, action_panel, show_job_button]
                    .spacing(10)
                    .width(Length::Fill)
            ]
            .spacing(20)
            .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Left) // Align the content to the start (left) horizontally
        .align_y(Vertical::Bottom) // Align the content to the end (bottom) vertically
        .padding(20)
        .into()
    }
}
