use iced::{
    Application,
    Element,
    theme::Theme,
    alignment::{Horizontal, Vertical, Alignment},
    Length,
    Command,
    widget::{column, container, horizontal_rule, radio, row, scrollable, text} 
};
use plotters_iced::{Chart, ChartWidget, DrawingBackend, ChartBuilder};
use plotters::coord::types::RangedDateTime;
use chrono::{Datelike, NaiveDateTime, NaiveDate, NaiveTime, Timelike};
use plotters::prelude::*;
use iced_aw::{number_input::NumberInput, style::NumberInputStyles};

use crate::{SimState, run_simulation};

#[derive(Debug, Clone)]
pub enum Message {
    BatteryCapacityChanged(f32),
    SolarCapacityChanged(f32),
    LoadChanged(f32),
    LatitudeChanged(f32),
    StartDateChanged(f32),
    EndDateChanged(f32),
    ChartEvent(ChartMessage),
    AxisChoiceChanged(SecondAxis)
}

pub struct AppState {
    pub sim_state: SimState,
    pub plot: DateLineChart,
    pub second_axis: SecondAxis
}

impl Application for AppState {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {  
        let mut starting_state = SimState::new();
        starting_state.battery_capacity = 1000.;
        starting_state.solar_nominal_output = 100.;
        starting_state.load = 25.;
        starting_state.latitude = 36.;
        let state = run_simulation(&starting_state); 
        let plot = DateLineChart::new(
            state.history_dates.clone().into_iter().map(|d| d).collect(),
            vec![state.charge_history.clone()],
            Vec::new(),
            vec!["State of Charge".to_string()],
            );    
        (AppState { 
            sim_state: state,
            plot,
            second_axis: SecondAxis::None
            }, 
        Command::none())
    }

    fn title(&self) -> String {
        "Solar Battery Simulation".to_string()
    }

    fn update(&mut self, event: Message) -> Command<Message>{
        match event {
            Message::BatteryCapacityChanged(capacity) => self.sim_state.battery_capacity = capacity,
            Message::SolarCapacityChanged(capacity) => self.sim_state.solar_nominal_output = capacity,
            Message::LoadChanged(load) => self.sim_state.load = load,
            Message::LatitudeChanged(lat) => self.sim_state.latitude = lat,
            Message::StartDateChanged(day) => self.sim_state.start_day = day as u32,
            Message::EndDateChanged(day) => self.sim_state.end_day = day as u32,
            Message::ChartEvent(_) => (),
            Message::AxisChoiceChanged(axis) => self.second_axis = axis,
        }
        self.sim_state = run_simulation(&self.sim_state);
        let mut labels = vec!["State of Charge".to_string()];
        let mut secondary_data = Vec::new();
        match self.second_axis {
            SecondAxis::None => (),
            SecondAxis::SolarPower => {
                labels.push("Solar Output".to_string());
                secondary_data.push(self.sim_state.solar_history.clone());
            },
            SecondAxis::SunlightHours => {
                labels.push("Daylight Hours".to_string());
                secondary_data.push(self.sim_state.daylight_history.clone());
            }
        }
        self.plot = DateLineChart::new(
            self.sim_state.history_dates.clone().into_iter().map(|d| d).collect(),
            vec![self.sim_state.charge_history.clone()],
            secondary_data,
            labels,
        ); 
        Command::none()
    }
    
    fn view(&self) -> Element<Message> {
        let battery_input = NumberInput::new(self.sim_state.battery_capacity, 1000000000000000000., Message::BatteryCapacityChanged)
            .style(NumberInputStyles::Default)
            .step(1.).width(Length::Fixed(80.));
        let solar_input = NumberInput::new(self.sim_state.solar_nominal_output, 1000000000000000000., Message::SolarCapacityChanged)
            .style(NumberInputStyles::Default)
            .step(1.).width(Length::Fixed(80.));

        let load_input = NumberInput::new(self.sim_state.load, 1000000000000000000., Message::LoadChanged)
            .style(NumberInputStyles::Default)
            .step(1.).width(Length::Fixed(80.));

        let lat_input = NumberInput::new(self.sim_state.latitude, 1000000000000000000., Message::LatitudeChanged)
            .style(NumberInputStyles::Default)
            .step(0.1).width(Length::Fixed(80.));

        let start_input = NumberInput::new(self.sim_state.start_day as f32, 365., Message::StartDateChanged)
            .style(NumberInputStyles::Default)
            .step(1.).width(Length::Fixed(80.));

        let end_input = NumberInput::new(self.sim_state.end_day as f32, 365., Message::EndDateChanged)
            .style(NumberInputStyles::Default)
            .step(1.).width(Length::Fixed(80.));

        let choose_axis =
        [SecondAxis::None, SecondAxis::SolarPower, SecondAxis::SunlightHours]
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

        let inputs = scrollable(
            column![
                text("Settings").width(Length::Fill).horizontal_alignment(Horizontal::Center),
                row![text("Battery Capacity [Wh]").width(Length::Fill), battery_input,],
                row![text("Solar Power Nominal [W]").width(Length::Fill), solar_input,],
                row![text("Load [W]").width(Length::Fill), load_input,],
                row![text("Latitude [degrees]").width(Length::Fill), lat_input,],
                horizontal_rule(1),
                row![text("Start Day").width(Length::Fill), start_input,],
                row![text("End Day").width(Length::Fill), end_input,],
                choose_axis,
                ].padding(10)
                .spacing(10)
            .align_items(Alignment::Start)
            .width(Length::Shrink)
        ).width(Length::Fixed(250.));

        let content = row![
            inputs,
            self.plot.view().map(Message::ChartEvent),
            ];

        container(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center).into()
    }
}

#[derive(Debug, Clone)]
pub enum ChartMessage {
    Updated
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SecondAxis {
    None,
    SolarPower,
    SunlightHours,
}

pub struct DateLineChart {
    xs: Vec<NaiveDateTime>,
    ys: Vec<Vec<f32>>,
    ys_secondary: Vec<Vec<f32>>,
    labels: Vec<String> ,   
}

impl Chart<ChartMessage> for DateLineChart {
    type State = ();
    fn build_chart<DB:DrawingBackend>(
        &self, 
        _: &Self::State, 
        mut builder: ChartBuilder<DB>) {
        //use plotters::{prelude::*, style::Color};
        // root.fill(&WHITE).unwrap();

        //const PLOT_LINE_COLOR: RGBColor = RGBColor(0, 175, 255);
        
        let from_date = *self.xs.first().clone().unwrap_or(
            &NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 
            NaiveTime::from_hms_opt(1,0,0).unwrap()));
        let to_date = *self.xs.last().unwrap_or(
            &NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(), 
            NaiveTime::from_hms_opt(1,0,0).unwrap()));
    
        let mut y_max: f32 = self.ys.iter().map(|y| 
            y.clone().into_iter().reduce(f32::max))
            .filter(|i| i.is_some())
            .map(|i| i.unwrap()).reduce(f32::max).unwrap_or(1.);
        if y_max == 0. {
            y_max = 1.
        }

        let y_secondary_max: f32 = if self.ys_secondary.len() == 0 {
            1.
        } else {
            self.ys_secondary.iter().map(|y| 
            y.clone().into_iter().reduce(f32::max))
            .filter(|i| i.is_some())
            .map(|i| i.unwrap()).reduce(f32::max).unwrap()
        };

        let mut chart = builder
            .x_label_area_size(28_i32)
            .y_label_area_size(28_i32)
            .right_y_label_area_size(40)
            .margin(20_i32)
            .build_cartesian_2d(
                RangedDateTime::from(from_date..to_date), 
                0_f32..y_max*1.05).unwrap()
            .set_secondary_coord(
                RangedDateTime::from(from_date..to_date), 
                0_f32..y_secondary_max*1.05);

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
            .axis_desc_style(("sans-serif", 16).into_font().transform(FontTransform::Rotate90))
            .y_desc("Battery Charge")
            .draw()
            .expect("failed to draw chart mesh");
    
        if self.ys_secondary.len() > 0 {
        chart
            .configure_secondary_axes()
            .y_desc(self.labels.last().unwrap())
            .draw().unwrap();
        }
    
        let colors = vec![
            &BLUE, 
            &RED, 
            &BLACK, 
            &RGBColor(0, 128, 0), // green 
            &RGBColor(255, 146, 0), // Orange/brown 
            &RGBColor(0, 153, 230), // light blue
            &RGBColor(180, 0, 180), // Purple
            &RGBColor(255, 150, 150), // pink
        ];
        let mut color_index = 0;
        let n = vec![self.ys.len(), colors.len(), self.labels.len()].iter().min().unwrap_or(&1).clone() as usize;
    
        for i in 0..n {
            let this_data: Vec<(NaiveDateTime,f32)> = self.xs.clone().into_iter()
                .zip(self.ys[i.clone()].clone().into_iter()).collect();
            let this_color = colors[color_index];
            let this_label = self.labels[i].clone();
            chart
            .draw_series(
                LineSeries::new(
                    this_data.iter().cloned(),
                    this_color,
                    //PLOT_LINE_COLOR.mix(0.175),
                )
                //.border_style(ShapeStyle::from(**color).stroke_width(2)),
            )
            .expect("failed to draw chart data")
            .label(this_label)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], this_color.clone()));
            color_index += 1;
        }
    
    
        let n = vec![self.ys_secondary.len(), colors.len(), self.labels.len()].iter().min().unwrap_or(&1).clone() as usize;
    
        for i in 0..n {
            let this_data: Vec<(NaiveDateTime,f32)> = self.xs.clone().into_iter()
                .zip(self.ys_secondary[i.clone()].clone().into_iter()).collect();
            let this_color = colors[color_index];
            let this_label = self.labels[color_index].clone();
            chart
            .draw_secondary_series(
                LineSeries::new(
                    this_data.iter().cloned(),
                    this_color,
                    //PLOT_LINE_COLOR.mix(0.175),
                )
                //.border_style(ShapeStyle::from(**color).stroke_width(2)),
            )
            .expect("failed to draw chart data")
            .label(this_label)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], this_color.clone()));
            color_index += 1;
        }
    
        if self.ys_secondary.len() > 0 {
            chart.configure_series_labels()
            .label_font(("sans-serif", 16))
            .background_style(&WHITE)
            .border_style(&BLACK)
            .draw().expect("Failed to draw legend")    
        }
    }
    
}

impl DateLineChart {
    pub fn new(xs: Vec<NaiveDateTime>, ys: Vec<Vec<f32>>, ys_secondary: Vec<Vec<f32>>, labels:Vec<String>) -> Self {
        DateLineChart {
            xs,
            ys, 
            ys_secondary,
            labels,
        }
    }
    pub fn view(&self)->Element<ChartMessage> {
        ChartWidget::new(self)
            //.width(Length::Fixed(200.))
            //.height(Length::Fixed(200.))
            .into()
    }
}
