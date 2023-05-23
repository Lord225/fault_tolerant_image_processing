use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy;
use iced::theme::{self, Theme};
use iced::widget::{
    button, checkbox, column, container, horizontal_rule, progress_bar, radio, row, scrollable,
    slider, text, text_input, toggler, vertical_rule, vertical_space,
};
use iced::{Alignment, Color, Element, Length, Sandbox, Settings};
use diesel::{prelude::*, insert_into};

mod workers;
mod schema;
mod journal;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

fn run_migration(conn: &mut impl MigrationHarness<diesel::pg::Pg>) {
    match conn.run_pending_migrations(MIGRATIONS) {
        Ok(_) => println!("Migrations applied successfully!"),
        Err(e) => println!("Error applying migrations: {}", e),
    }
}

// users table
#[derive(Queryable, Debug)]
#[allow(dead_code)]
struct User {
    id: i32,
    name: String,
}

pub fn main() -> iced::Result {
    // init .env
    dotenvy::dotenv().ok();
    
    // get postgres connection
    let mut conn =
        diesel::pg::PgConnection::establish(&std::env::var("DATABASE_URL").unwrap())
            .expect("Error connecting to postgres");

    // run migrations
    run_migration(&mut conn);
    use schema::users::dsl::*;

    // insert new user to db
    insert_into(users)
        .values(name.eq("John Doe"))
        .execute(&mut conn)
        .expect("Error inserting user");

    // get value from db
    let all_users = users
        .load::<User>(&mut conn)
        .expect("Error loading users");

    println!("Users: {:?}", all_users);

    Styling::run(Settings::default())
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
