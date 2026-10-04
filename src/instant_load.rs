use crate::gui::LoadMessage;
use crate::{LoadFrequency, Weekday};
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use iced::widget::{Button, column, pick_list, row, rule, text};
use iced::{Element, Length};
use iced_aw::NumberInput;
#[derive(Debug, Clone)]
pub struct InstantLoad {
    pub time: NaiveTime,
    pub energy: f32, // Wh
    pub frequency: LoadFrequency,
    pub id: usize,
    pub weekday: Weekday,
}

impl InstantLoad {
    #[must_use]
    pub fn energy(&self, now: NaiveDateTime, step: Duration) -> f32 {
        let start = now - step;
        match self.frequency {
            LoadFrequency::Hourly => {
                if start.hour() == now.hour() {
                    0.0
                } else {
                    self.energy
                }
            }
            LoadFrequency::Daily => {
                if start.time() < self.time && self.time <= now.time() {
                    self.energy
                } else {
                    0.0
                }
            }
            LoadFrequency::Weekly => {
                let time_day = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap(),
                    self.time,
                );
                if start < time_day && time_day <= now {
                    self.energy
                } else {
                    0.0
                }
            }
        }
    }

    pub fn view<'a>(&self, _: usize) -> Element<'a, LoadMessage> {
        let frequency_list = [
            LoadFrequency::Hourly,
            LoadFrequency::Daily,
            LoadFrequency::Weekly,
        ];
        let frequency_selector = pick_list(
            frequency_list,
            Some(self.frequency.clone()),
            LoadMessage::FrequencyChanged,
        )
        .width(Length::Fixed(90.));
        let day_list = [
            Weekday::Monday,
            Weekday::Tuesday,
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
            Weekday::Saturday,
            Weekday::Sunday,
        ];
        let day_selector = pick_list(
            day_list,
            Some(self.weekday.clone()),
            LoadMessage::WeekdayChanged,
        )
        .width(Length::Fixed(90.));
        match self.frequency {
            LoadFrequency::Daily => row![
                column![
                    rule::horizontal(1),
                    row![
                        frequency_selector,
                        text("load at ").width(Length::FillPortion(1)),
                        NumberInput::new(&self.time.hour(), 0..24, LoadMessage::TimeChanged)
                            .width(Length::Fixed(50.)),
                        text("oclock of ").width(Length::FillPortion(1)),
                    ]
                    .spacing(5.),
                    row![
                        NumberInput::new(&self.energy, 0. ..1000000., LoadMessage::LoadChanged)
                            .width(Length::Fixed(100.)),
                        text("Wh").width(Length::Fill),
                        Button::new("X")
                            .on_press(LoadMessage::Delete)
                            .width(Length::Shrink),
                    ]
                    .spacing(5.)
                ]
                .spacing(10.)
            ],

            LoadFrequency::Weekly => row![
                column![
                    rule::horizontal(1),
                    row![
                        frequency_selector,
                        text("load at ").width(Length::FillPortion(1)),
                        NumberInput::new(&self.time.hour(), 0..24, LoadMessage::TimeChanged)
                            .width(Length::Fixed(50.)),
                        text("oclock on").width(Length::FillPortion(1))
                    ]
                    .spacing(5.),
                    row![
                        day_selector,
                        text("of ").width(Length::Shrink),
                        NumberInput::new(&self.energy, 0. ..1000000., LoadMessage::LoadChanged)
                            .width(Length::Fixed(100.)),
                        text("Wh").width(Length::Fill),
                        Button::new("X")
                            .on_press(LoadMessage::Delete)
                            .width(Length::Shrink),
                    ]
                    .spacing(5),
                ]
                .spacing(10.)
            ],

            LoadFrequency::Hourly => row![
                column![
                    rule::horizontal(1),
                    row![
                        frequency_selector,
                        text("load of").width(Length::Shrink),
                        NumberInput::new(&self.energy, 0. ..1000000., LoadMessage::LoadChanged)
                            .width(Length::Fixed(100.)),
                        text("Wh").width(Length::Fill),
                        Button::new("X")
                            .on_press(LoadMessage::Delete)
                            .width(Length::Shrink),
                    ]
                    .spacing(5.),
                ]
                .spacing(10.)
            ],
        }
        .into()
    }
    pub fn update(&mut self, message: LoadMessage) {
        match message {
            LoadMessage::FrequencyChanged(f) => self.frequency = f,
            LoadMessage::LoadChanged(e) => self.energy = e,
            LoadMessage::TimeChanged(t) => self.time = NaiveTime::from_hms_opt(t, 0, 0).unwrap(),
            LoadMessage::Delete => (),
            LoadMessage::WeekdayChanged(d) => self.weekday = d,
        }
    }
}
