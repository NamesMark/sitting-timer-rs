use iced::alignment;
use iced::executor;
use iced::theme::{self, Theme};
use iced::time;
use iced::widget::{button, column, container, row, text};
use iced::{
    Alignment, Application, Command, Element, Length, Settings, Subscription,
};

use std::time::{Duration, Instant};

const MAX_SITTING_TIME: Duration = Duration::from_secs(1 * 60); // 30 minutes
const MAX_STANDING_TIME: Duration = Duration::from_secs(30 * 60); // 30 minutes

// #[derive(Debug, Clone, Copy)]
// enum Message {
//     Tick(std::time::Duration),
//     SwitchPosition,
// }

pub fn main() -> iced::Result {
    Sitwatch::run(Settings::default())
}

struct Sitwatch {
    standing_duration: Duration,
    sitting_duration: Duration,
    state: State,
}

enum State {
    Sitting { last_tick: Instant },
    Standing { last_tick: Instant },
}

#[derive(Debug, Clone)]
enum Message {
    Toggle,
    Reset,
    Tick(Instant),
}

impl Application for Sitwatch {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Sitwatch, Command<Message>) {
        (
            Sitwatch {
                standing_duration: Duration::default(),
                sitting_duration: Duration::default(),
                state: State::Sitting { last_tick: Instant::now() },
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Don't sit")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Toggle => match self.state {
                State::Sitting { .. } => {
                    self.state = State::Standing {
                        last_tick: Instant::now(),
                    };
                    self.sitting_duration = Duration::default();
                }
                State::Standing { .. } => {
                    self.state = State::Sitting {
                        last_tick: Instant::now(),
                    };
                    self.standing_duration = Duration::default();
                }
            },
            Message::Tick(now) => {
                if let State::Standing { last_tick } = &mut self.state {
                    self.standing_duration += now - *last_tick;
                    *last_tick = now;
                } else if let State::Sitting { last_tick } = &mut self.state {
                    self.sitting_duration += now - *last_tick;
                    *last_tick = now;
                }
            }
            Message::Reset => {
                self.sitting_duration = Duration::default();
                self.standing_duration = Duration::default();
                self.state = State::Sitting { last_tick: Instant::now(), };
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.state {
            State::Sitting { .. } => {
                time::every(Duration::from_millis(10)).map(Message::Tick)
            },
            State::Standing { .. } => {
                time::every(Duration::from_millis(10)).map(Message::Tick)
            }
        }
    }

    fn view(&self) -> Element<Message> {
        const MINUTE: u64 = 60;
        const HOUR: u64 = 60 * MINUTE;

        let standing_seconds = self.standing_duration.as_secs();
        let sitting_seconds = self.sitting_duration.as_secs();

        let state_message = text(format!(
            "You are currently: {}", match self.state {
                State::Sitting { .. } => "SITTING",
                State::Standing { .. } => "STANDING",
            }
        ))
        .size(40);

        let warning_message = if self.sitting_duration > MAX_SITTING_TIME {
            Some(
                text(format!(
                    "You have been sitting for over {} minutes! Get up, lazy!",
                    MAX_SITTING_TIME.as_secs() / 60
                ))
                .size(40),
            )
        } else {
            None
        };

        let sitting_duration = text(format!(
            "{:0>2}:{:0>2}:{:0>2}.{:0>2}",
            sitting_seconds / HOUR,
            ( sitting_seconds % HOUR) / MINUTE,
            sitting_seconds % MINUTE,
            self.sitting_duration.subsec_millis() / 10,
        ))
        .size(40);

        let standing_duration = text(format!(
            "{:0>2}:{:0>2}:{:0>2}.{:0>2}",
            standing_seconds / HOUR,
            (standing_seconds % HOUR) / MINUTE,
            standing_seconds % MINUTE,
            self.standing_duration.subsec_millis() / 10,
        ))
        .size(40);

        let button = |label| {
            button(
                text(label).horizontal_alignment(alignment::Horizontal::Center),
            )
            .padding(10)
            .width(60)
            .height(60)
        };

        let sitting_toggle_button = {
            let label = match self.state {
                State::Standing { .. } => "Start sitting",
                State::Sitting { .. } => "Stop sitting",
            };

            button(label).on_press(Message::Toggle)
        };

        let standing_toggle_button = {
            let label = match self.state {
                State::Sitting { .. }  => "Start standing",
                State::Standing { .. } => "Stop standing",
            };

            button(label).on_press(Message::Toggle)
        };

        let reset_button = button("Reset")
            .style(theme::Button::Destructive)
            .on_press(Message::Reset);

        let controls = row![sitting_toggle_button, standing_toggle_button, reset_button].spacing(20);

        let mut content = column![state_message, sitting_duration, standing_duration, controls]
            .align_items(Alignment::Center)
            .spacing(20);

        if let Some(warning_message) = warning_message {
            content = content.push(warning_message);
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}