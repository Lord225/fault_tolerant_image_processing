use database::common::Database;
use database::repositories::task::{InsertableTaskTree, Task, InsertableTask};
use iced::{Application, Color, Command, Rectangle, Subscription};
use log::{debug, warn};
use processing::worker::WorkerErrorConfig;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::{error::Error, vec};

use clap::Parser;

use engine::{run, ConfigType};
use iced::alignment::{Horizontal, Vertical};
use iced::theme::Theme;
use iced::widget::{pick_list, radio, slider, toggler, Scrollable};

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

use processing::job::{self, BlurJob, BrightnessJob, CropJob, JobType, ResizeJob, OverlayJob};

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

    let (th, config) = run();

    //Styling::run(Settings::default())?;
    MyApp::run(Settings {
        flags: config,
        ..Settings::default()
    })?;

    Ok(())
}

#[derive(Debug, Clone)]
struct TaskElement {
    id: i64,
    name: String,
    job_type: AvalibleActions,
    status: database::schema::Status,
}

impl<'a, Message: 'a> From<TaskElement> for iced::Element<'a, Message> {
    fn from(task: TaskElement) -> Self {
        let TaskElement {
            id: _,
            name,
            status,
            job_type,
        } = task;

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
                let rectangle =
                    iced::widget::canvas::Path::rectangle(iced::Point::ORIGIN, bounds.size());
                let background = iced::widget::canvas::Fill {
                    style: iced::widget::canvas::Style::Solid(self.0),
                    ..iced::widget::canvas::Fill::default()
                };

                let mut frame = iced::widget::canvas::Frame::new(bounds.size());
                frame.fill(&rectangle, background);

                const FONT_SIZE: f32 = 20.0;

                // draw text
                let text = iced::widget::canvas::Text {
                    content: self.1.clone(),
                    // middle of the rectangle
                    position: iced::Point::new(5.0, bounds.height / 2.0 - FONT_SIZE / 2.0),
                    size: FONT_SIZE,
                    color: Color::BLACK,
                    ..iced::widget::canvas::Text::default()
                };

                frame.fill_text(text);

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
        let name = format!("{:?} {:?}", task.task_id, task.params);
        TaskElement {
            id: task.task_id,
            name: name,
            job_type: match task.params {
                JobType::Crop(_) => AvalibleActions::Crop,
                JobType::Brightness(_) => AvalibleActions::Brighten,
                JobType::Resize(_) => AvalibleActions::Resize,
                JobType::Blur(_) => AvalibleActions::Blur,
                JobType::Overlay(_) => AvalibleActions::Crop, //TODO
                JobType::Input => AvalibleActions::Input,
            },
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
    Overlay,
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
enum OverlayActions {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SliderChangedAction {
    Crop(CropActions),
    Brighten(BrightenActions),
    Resize(ResizeActions),
    Blur(BlurActions),
    Overlay(OverlayActions),
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
            AvalibleActions::Overlay => write!(f, "Overlay"),
        }
    }
}

struct MyApp {
    selected_file: Option<PathBuf>,
    items: Vec<TaskElement>,
    panel_state: JobType,
    choosed_input_state: Vec<Option<i64>>,
    current_action: AvalibleActions,
    db: Database,
    config: ConfigType,
    last_config: WorkerErrorConfig,
}

impl MyApp {
    fn fetch_tasks(&mut self) {
        self.items = self
            .db
            .get_all_tasks()
            .unwrap()
            .into_iter()
            .map(|x| x.into())
            .collect();
    }
}

//
// APP layout
//
impl MyApp {
    fn action_to_panel(&self, action: AvalibleActions) -> Element<'_, Message> {
        fn gen_input_list(n: i64, app: &MyApp) -> Element<'_, Message> {
            let mut list = Column::new();
            
            // push pick list with avalible (Completed) tasks
            let completed_tasks = app.items
                                                  .iter()
                                                  .map(|x| x.id).collect::<Vec<_>>(); 

            for i in 0..n {
                let mut pick_list = iced::widget::PickList::new(
                    completed_tasks.clone(),
                    app.choosed_input_state.get(i as usize).unwrap_or(&None).clone(),
                    move |x| Message::InputChoosed(x, i),
                );
                pick_list = pick_list.width(Length::Fill);
                list = list.push(pick_list);
            }

            list.into()
            
        }
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
                    row![gen_input_list(1, self)],
                ]
                .spacing(5)
                .into()
            }
            (AvalibleActions::Brighten, JobType::Brightness(x)) => {
                let value = slider(0.0..=100.0, x.0, |x| {
                    Message::SliderChanged(x, SliderChangedAction::Brighten(BrightenActions::Value))
                });

                column![row![Text::new("value"), value].spacing(5),
                        row![gen_input_list(1, self)],]
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

                column![row![Text::new("width"), width, Text::new("height"), height].spacing(5),row![gen_input_list(1, self)]]
                    .spacing(5)
                    .into()
            }
            (AvalibleActions::Blur, JobType::Blur(x)) => {
                let value = slider(0.0..=100.0, x.0, |x| {
                    Message::SliderChanged(x, SliderChangedAction::Blur(BlurActions::Value))
                });

                column![row![Text::new("amount"), value].spacing(5),row![gen_input_list(1, self)]]
                    .spacing(5)
                    .into()
            },
            (AvalibleActions::Overlay, JobType::Overlay(job)) => {
                let x = slider(0.0..=100.0, job.0 as f32, |x| {
                    Message::SliderChanged(x, SliderChangedAction::Overlay(OverlayActions::X))
                });

                let y = slider(0.0..=100.0, job.1 as f32, |x| {
                    Message::SliderChanged(x, SliderChangedAction::Overlay(OverlayActions::Y))
                });

                

                column![row![Text::new("x"), x, Text::new("y"), y].spacing(5),row![gen_input_list(2, self)]]
                    .spacing(5)
                    .into()
            },
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

    fn config_controls(&self) -> Element<'_, Message> {
        const THROTTLE_RANGE: f32 = 2.0;
        let config = self.last_config.clone();

        let throttle = slider(0.0..=100.0, config.throttle.as_secs_f32() * 100.0 / THROTTLE_RANGE, |x| {
            Message::ThrottleChanged(x / 100.0 * THROTTLE_RANGE)
        });
        let error_chance = slider(0.0..=100.0, config.random_error_chance * 100.0, |x| {
            Message::ErrorChanceChanged(x / 100.0)
        });
        // radio
        let paused = toggler(Some("Paused".into()), config.paused, |x| {
            Message::PausedChanged(x)
        });
        let add_button = Button::new(Text::new("Add Item")).on_press(Message::AddItem);

        column![row![
            Text::new("Throttle"),
            throttle,
            Text::new("Error Chance"),
            error_chance,
            paused,
            add_button
        ]
        .spacing(5),]
        .into()
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
    ErrorChanceChanged(f32),
    ThrottleChanged(f32),
    PausedChanged(bool),
    PeriodicEvent,
    InputChoosed(i64, i64),
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
            SliderChangedAction::Overlay(a) => {
                if let JobType::Overlay(t) = &mut self.panel_state {
                    match a {
                        OverlayActions::X => {
                            t.0 = value as u32;
                        }
                        OverlayActions::Y => {
                            t.1 = value as u32;
                        }
                    }
                } else {
                    self.panel_state = JobType::Overlay(OverlayJob(0, 0));
                }
            }
        }
    }
}

impl Application for MyApp {
    type Message = Message;
    type Flags = ConfigType;
    type Executor = iced::executor::Default;
    type Theme = Theme;

    fn new(settings: Self::Flags) -> (Self, Command<Self::Message>) {
        let last_config = settings.read().unwrap().clone();
        (
            MyApp {
                selected_file: None,
                items: vec![],
                choosed_input_state: vec![],
                panel_state: JobType::Crop(CropJob(0, 0, 0, 0)),
                current_action: AvalibleActions::Crop,
                db: database::common::try_open_connection(),
                config: settings,
                last_config,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Image Processor")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        if let Ok(c) = self.config.try_read() {
            self.last_config = c.clone();
        }

        let mut commands = Command::none();

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
            Message::AddItem => self.fetch_tasks(),
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
                    AvalibleActions::Overlay => self.panel_state = JobType::Overlay(OverlayJob(0, 0)),
                }
            }
            Message::SliderChanged(value, w) => {
                self.update_state_on_slider(w, value);
                debug!("Slider {:?} changed to {:?} ", w, value);
            }
            Message::ConfirmJob => {
                let panel_state = &self.panel_state;
                let action = &self.current_action;
                let inputs = &self.choosed_input_state;
                
                match action {
                    AvalibleActions::Input => {
                        if let Some(path) = &self.selected_file {
                            let path = path.to_str().unwrap();
                            self.db.insert_new_task(&InsertableTask::input(&path)).unwrap();
                        } else {
                            warn!("No input file selected");
                        }
                    }
                    _ => {
                        let input_count = panel_state.input_count();

                        if input_count != inputs.len() {
                            commands = Command::perform(async {}, move |_| Message::ConfirmJob);
                        } else {
                            let inputs = inputs[0..input_count].iter().map(|x| *x).collect::<Option<Vec<_>>>();

                            let task = InsertableTask {
                                parent_ids: inputs.unwrap(),
                                status: database::schema::Status::Pending,
                                data: None,
                                params: *panel_state,
                            };

                            self.db.insert_new_task(&task).unwrap();
                        }
                    }
                }

            }
            Message::ErrorChanceChanged(x) => {
                // update error chance
                self.last_config.random_error_chance = x;
                if let Ok(mut c) = self.config.try_write() {
                    *c = self.last_config;
                } else {
                    commands = Command::perform(async {}, move |_| Message::ErrorChanceChanged(x));
                }
            }
            Message::ThrottleChanged(x) => {
                let duration = Duration::from_secs_f32(x);
                self.last_config.throttle = duration;
                if let Ok(mut c) = self.config.try_write() {
                    *c = self.last_config;
                } else {
                    commands = Command::perform(async {}, move |_| Message::ThrottleChanged(x));
                }
            }
            Message::PausedChanged(value) => {
                self.last_config.paused = value;
                if let Ok(mut c) = self.config.try_write() {
                    *c = self.last_config;
                } else {
                    commands = Command::perform(async {}, move |_| Message::PausedChanged(value));
                }
            }
            Message::PeriodicEvent => self.fetch_tasks(),
            Message::InputChoosed(id, index) => {
                if let Some(state) = self.choosed_input_state.get_mut(index as usize) {
                    *state = Some(id);
                    debug!("Input choosed {:?}", self.choosed_input_state);
                } else {
                    for _ in self.choosed_input_state.len()..=index as usize {
                        self.choosed_input_state.push(None);
                    }
                    commands = Command::perform(async {}, move |_| Message::InputChoosed(id, index));
                }
            }
        }

        commands
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::time;
        time::every(Duration::from_millis(250)).map(|_| Message::PeriodicEvent)
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

        let column = Column::new()
            .push(scrollable)
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

        let config = self.config_controls();
        // Container with two columns - left with scrollable list of items, right with pick list & buttons

        let layout = row![
            column![column].width(300),
            column![pick_list, action_panel, show_job_button]
                .spacing(10)
                .width(Length::Fill),
        ]
        .spacing(20)
        .padding(20);

        Container::new(
            Column::new()
                .push(
                    Container::new(layout)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Left)
                        .align_y(Vertical::Bottom)
                        .padding(20),
                )
                .push(Container::new(config).padding(20).align_y(Vertical::Bottom)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
