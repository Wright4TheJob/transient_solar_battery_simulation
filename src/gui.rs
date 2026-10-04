use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use iced::{
    Element, Length,
    alignment::{Horizontal, Vertical},
    widget::{Button, column, container, keyed_column, radio, row, rule, scrollable, text},
};
use iced_aw::number_input::NumberInput;
use plotters::coord::types::RangedDateTime;
use plotters::prelude::*;
use plotters_iced2::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

use crate::{InstantLoad, LoadFrequency, SimState, Weekday};

#[derive(Debug, Clone)]
pub enum Message {
    BatteryCapacityChanged(f32),
    SolarCapacityChanged(f32),
    LoadChanged(f32),
    LatitudeChanged(f32),
    StartDateChanged(u32),
    EndDateChanged(u32),
    ChartEvent(ChartMessage),
    AxisChoiceChanged(SecondAxis),
    InitialChargeChanged(f32),
    ReducedPowerPercentChanged(usize),
    ReducedPowerDaysChanged(usize),
    ReducedPowerDaysCycleChanged(usize),
    LoadMessage(usize, LoadMessage),
    NewLoad,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadMessage {
    FrequencyChanged(LoadFrequency),
    LoadChanged(f32),
    TimeChanged(u32),
    Delete,
    WeekdayChanged(Weekday),
}

#[derive(Default)]
pub struct AppState {
    pub sim_state: SimState,
    pub plot: DateLineChart,
    pub second_axis: SecondAxis,
    pub next_load_id: usize,
    pub frequency_labels: Vec<String>,
}

impl AppState {
    pub fn new() -> Self {
        let mut state = SimState::new();
        state.run_simulation();
        let frequency_labels = vec![
            "Hourly".to_string(),
            "Daily".to_string(),
            "Weekly".to_string(),
        ];
        let plot = DateLineChart::new(
            state.history_dates.clone().into_iter().map(|d| d).collect(),
            vec![state.charge_history.clone()],
            Vec::new(),
            vec!["State of Charge".to_string()],
        );
        AppState {
            sim_state: state,
            plot,
            second_axis: SecondAxis::None,
            next_load_id: 0,
            frequency_labels: frequency_labels,
        }
    }

    pub fn title(&self) -> String {
        "Solar Battery Simulation".to_string()
    }

    pub fn update(&mut self, event: Message) {
        match event {
            Message::BatteryCapacityChanged(capacity) => self.sim_state.battery_capacity = capacity,
            Message::SolarCapacityChanged(capacity) => {
                self.sim_state.solar_nominal_output = capacity
            }
            Message::LoadChanged(load) => self.sim_state.load = load,
            Message::LatitudeChanged(lat) => self.sim_state.latitude = lat,
            Message::StartDateChanged(day) => self.sim_state.start_day = day as u32,
            Message::EndDateChanged(day) => self.sim_state.end_day = day as u32,
            Message::ChartEvent(_) => (),
            Message::AxisChoiceChanged(axis) => self.second_axis = axis,
            Message::InitialChargeChanged(charge) => self.sim_state.initial_charge = charge,
            Message::ReducedPowerPercentChanged(percent) => {
                self.sim_state.reduced_power_percent = percent as f32 / 100.
            }
            Message::ReducedPowerDaysChanged(days) => self.sim_state.reduced_power_days = days,
            Message::ReducedPowerDaysCycleChanged(cycle) => {
                self.sim_state.reduced_power_days_cycle = cycle
            }
            Message::LoadMessage(id, message) => {
                if message == LoadMessage::Delete {
                    self.sim_state.point_loads.remove(&id);
                }
                if let Some(load) = self.sim_state.point_loads.get_mut(&id) {
                    load.update(message);
                }
            }
            Message::NewLoad => {
                self.sim_state.point_loads.insert(
                    self.next_load_id,
                    InstantLoad {
                        time: NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                        energy: 100.,
                        frequency: LoadFrequency::Daily,
                        id: self.next_load_id,
                        weekday: Weekday::Monday,
                    },
                );
                self.next_load_id += 1;
            }
        }
        self.sim_state.clone().run_simulation();
        let mut labels = vec!["State of Charge".to_string()];
        let mut secondary_data = Vec::new();
        match self.second_axis {
            SecondAxis::None => (),
            SecondAxis::SolarPower => {
                labels.push("Solar Output".to_string());
                secondary_data.push(self.sim_state.solar_history.clone());
            }
            SecondAxis::SunlightHours => {
                labels.push("Daylight Hours".to_string());
                secondary_data.push(self.sim_state.daylight_history.clone());
            }
        }
        self.plot = DateLineChart::new(
            self.sim_state
                .history_dates
                .clone()
                .into_iter()
                .map(|d| d)
                .collect(),
            vec![self.sim_state.charge_history.clone()],
            secondary_data,
            labels,
        );
    }

    pub fn view(&self) -> Element<'_, Message> {
        let battery_input = NumberInput::new(
            &self.sim_state.battery_capacity,
            0 as f32..=1000000000000000000.,
            Message::BatteryCapacityChanged,
        )
        .step(1.);
        let initial_charge_input = NumberInput::new(
            &self.sim_state.initial_charge,
            0 as f32..=100.,
            Message::InitialChargeChanged,
        )
        .step(5.);
        let solar_input = NumberInput::new(
            &self.sim_state.solar_nominal_output,
            0 as f32..=1000000000000000000.,
            Message::SolarCapacityChanged,
        )
        .step(1.);

        let load_input = NumberInput::new(
            &self.sim_state.load,
            0 as f32..=1000000000000000000.,
            Message::LoadChanged,
        )
        .step(1.);

        let lat_input = NumberInput::new(
            &self.sim_state.latitude,
            0 as f32..=1000000000000000000.,
            Message::LatitudeChanged,
        )
        .step(0.1);

        let start_input = NumberInput::new(
            &self.sim_state.start_day,
            0 as u32..=365 as u32,
            Message::StartDateChanged,
        )
        .step(1);

        let end_input = NumberInput::new(
            &self.sim_state.end_day,
            0 as u32..=365 as u32,
            Message::EndDateChanged,
        )
        .step(1);

        let reduced_power_percent_input = NumberInput::new(
            &((self.sim_state.reduced_power_percent.clone() * 100.) as usize),
            0..=100,
            Message::ReducedPowerPercentChanged,
        )
        .step(10);

        let reduced_power_days = NumberInput::new(
            &self.sim_state.reduced_power_days,
            0. as usize..=365 as usize,
            Message::ReducedPowerDaysChanged,
        )
        .step(1);

        let reduced_power_days_cycle = NumberInput::new(
            &self.sim_state.reduced_power_days_cycle,
            0 as usize..=365 as usize,
            Message::ReducedPowerDaysCycleChanged,
        )
        .step(1);

        let choose_axis = [
            SecondAxis::None,
            SecondAxis::SolarPower,
            SecondAxis::SunlightHours,
        ]
        .iter()
        .fold(
            column![text("Choose the secondary axis:")].spacing(10),
            |column, axis| {
                column.push(radio(
                    format!("{axis:?}"),
                    *axis,
                    Some(self.second_axis),
                    Message::AxisChoiceChanged,
                ))
            },
        );
        let new_load_button = Button::new("+")
            .width(Length::Fixed(25.))
            .on_press(Message::NewLoad);
        let loads = keyed_column(self.sim_state.point_loads.iter().map(move |(id, l)| {
            (
                l.id,
                l.view(*id)
                    .map(move |message| Message::LoadMessage(*id, message)),
            )
        }))
        .spacing(10);
        let number_input_width = 100.;
        let inputs = column![
            row![text("Settings").width(Length::Fill)],
            row![
                text("Battery Capacity [Wh]").width(Length::Fill),
                battery_input.width(Length::Fixed(number_input_width)),
            ],
            row![
                text("Initial Charge [%]").width(Length::Fill),
                initial_charge_input.width(Length::Fixed(number_input_width)),
            ],
            row![
                text("Solar Power Nominal [W]").width(Length::Fill),
                solar_input.width(Length::Fixed(number_input_width)),
            ],
            row![
                text("Load [W]").width(Length::Fill),
                load_input.width(Length::Fixed(number_input_width))
            ],
            row![
                text("Latitude [degrees]").width(Length::Fill),
                lat_input.width(Length::Fixed(number_input_width)),
            ],
            rule::horizontal(1),
            row![
                text("Start Day").width(Length::Fill),
                start_input.width(Length::Fixed(number_input_width)),
            ],
            row![
                text("End Day").width(Length::Fill),
                end_input.width(Length::Fixed(number_input_width)),
            ],
            rule::horizontal(1),
            row![
                text("Cloudy Day output [%]").width(Length::Fill),
                reduced_power_percent_input.width(Length::Fixed(number_input_width)),
            ],
            row![
                reduced_power_days.width(Length::Fixed(number_input_width)),
                text("cloudy days")
            ],
            row![
                text("out of every "),
                reduced_power_days_cycle.width(Length::Fixed(number_input_width)),
                text(" days")
            ],
            rule::horizontal(1),
            row![text("Instant Loads").width(Length::Fill), new_load_button],
            loads,
            rule::horizontal(1),
            choose_axis
        ];
        let content = row![
            scrollable(inputs.padding(10).spacing(10).width(Length::Fill))
                .width(Length::Fixed(350.)),
            self.plot.view().map(Message::ChartEvent),
        ];

        container(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }
}

#[derive(Debug, Clone)]
pub enum ChartMessage {
    Updated,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum SecondAxis {
    None,
    SolarPower,
    #[default]
    SunlightHours,
}

#[derive(Default)]
pub struct DateLineChart {
    xs: Vec<NaiveDateTime>,
    ys: Vec<Vec<f32>>,
    ys_secondary: Vec<Vec<f32>>,
    labels: Vec<String>,
}

impl Chart<ChartMessage> for DateLineChart {
    type State = ();
    fn build_chart<DB: DrawingBackend>(&self, _: &Self::State, mut builder: ChartBuilder<DB>) {
        //use plotters::{prelude::*, style::Color};
        // root.fill(&WHITE).unwrap();

        //const PLOT_LINE_COLOR: RGBColor = RGBColor(0, 175, 255);

        let from_date = *self.xs.first().clone().unwrap_or(&NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(1, 0, 0).unwrap(),
        ));
        let to_date = *self.xs.last().unwrap_or(&NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(),
            NaiveTime::from_hms_opt(1, 0, 0).unwrap(),
        ));

        let mut y_max: f32 = self
            .ys
            .iter()
            .map(|y| y.clone().into_iter().reduce(f32::max))
            .filter(|i| i.is_some())
            .map(|i| i.unwrap())
            .reduce(f32::max)
            .unwrap_or(1.);
        if y_max == 0. {
            y_max = 1.
        }

        let y_secondary_max: f32 = if self.ys_secondary.len() == 0 {
            1.
        } else {
            self.ys_secondary
                .iter()
                .map(|y| y.clone().into_iter().reduce(f32::max))
                .filter(|i| i.is_some())
                .map(|i| i.unwrap())
                .reduce(f32::max)
                .unwrap()
        };

        let mut chart = builder
            .x_label_area_size(28_i32)
            .y_label_area_size(28_i32)
            .right_y_label_area_size(40)
            .margin(20_i32)
            .build_cartesian_2d(
                RangedDateTime::from(from_date..to_date),
                0_f32..y_max * 1.05,
            )
            .unwrap()
            .set_secondary_coord(
                RangedDateTime::from(from_date..to_date),
                0_f32..y_secondary_max * 1.05,
            );

        chart
            .configure_mesh()
            // .x_labels(6)
            .x_label_formatter(if (to_date - from_date).num_days() < 5 {
                &|x| format!("{}-{} {}:{:02}", x.day(), x.month(), x.hour(), x.minute())
            } else {
                &|x| format!("{}-{}", x.day(), x.month())
            })
            .y_label_style(
                ("sans-serif", 16)
                    .into_font()
                    //        .color(&plotters::style::colors::BLUE.mix(0.65))
                    .transform(FontTransform::Rotate90),
            )
            .y_label_formatter(&|y| format!("{}", y))
            .axis_desc_style(
                ("sans-serif", 16)
                    .into_font()
                    .transform(FontTransform::Rotate90),
            )
            .y_desc("Battery Charge")
            .draw()
            .expect("failed to draw chart mesh");

        if self.ys_secondary.len() > 0 {
            chart
                .configure_secondary_axes()
                .y_desc(self.labels.last().unwrap())
                .draw()
                .unwrap();
        }

        let colors = [
            &BLUE,
            &RED,
            &BLACK,
            &RGBColor(0, 128, 0),     // green
            &RGBColor(255, 146, 0),   // Orange/brown
            &RGBColor(0, 153, 230),   // light blue
            &RGBColor(180, 0, 180),   // Purple
            &RGBColor(255, 150, 150), // pink
        ];
        let mut color_index = 0;
        let n = [self.ys.len(), colors.len(), self.labels.len()]
            .iter()
            .min()
            .unwrap_or(&1)
            .clone() as usize;

        for i in 0..n {
            let this_data: Vec<(NaiveDateTime, f32)> = self
                .xs
                .clone()
                .into_iter()
                .zip(self.ys[i.clone()].clone().into_iter())
                .collect();
            let this_color = colors[color_index];
            let this_label = self.labels[i].clone();
            chart
                .draw_series(
                    LineSeries::new(
                        this_data.iter().cloned(),
                        this_color,
                        //PLOT_LINE_COLOR.mix(0.175),
                    ), //.border_style(ShapeStyle::from(**color).stroke_width(2)),
                )
                .expect("failed to draw chart data")
                .label(this_label)
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], this_color.clone()));
            color_index += 1;
        }

        let n = [self.ys_secondary.len(), colors.len(), self.labels.len()]
            .iter()
            .min()
            .unwrap_or(&1)
            .clone() as usize;

        for i in 0..n {
            let this_data: Vec<(NaiveDateTime, f32)> = self
                .xs
                .clone()
                .into_iter()
                .zip(self.ys_secondary[i.clone()].clone().into_iter())
                .collect();
            let this_color = colors[color_index];
            let this_label = self.labels[color_index].clone();
            chart
                .draw_secondary_series(
                    LineSeries::new(
                        this_data.iter().cloned(),
                        this_color,
                        //PLOT_LINE_COLOR.mix(0.175),
                    ), //.border_style(ShapeStyle::from(**color).stroke_width(2)),
                )
                .expect("failed to draw chart data")
                .label(this_label)
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], this_color.clone()));
            color_index += 1;
        }

        if self.ys_secondary.len() > 0 {
            chart
                .configure_series_labels()
                .label_font(("sans-serif", 16))
                .background_style(&WHITE)
                .border_style(&BLACK)
                .draw()
                .expect("Failed to draw legend")
        }
    }
}

impl DateLineChart {
    pub fn new(
        xs: Vec<NaiveDateTime>,
        ys: Vec<Vec<f32>>,
        ys_secondary: Vec<Vec<f32>>,
        labels: Vec<String>,
    ) -> Self {
        DateLineChart {
            xs,
            ys,
            ys_secondary,
            labels,
        }
    }
    pub fn view(&self) -> Element<'_, ChartMessage> {
        ChartWidget::new(self).into()
        //.width(Length::Fixed(200.))
        //.height(Length::Fixed(200.))
    }
}
