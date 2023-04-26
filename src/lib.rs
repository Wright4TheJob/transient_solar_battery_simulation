pub mod gui;

use std::f32::consts::PI;
use plotters::prelude::*;
use plotters::coord::types::RangedDateTime;
use chrono::{Datelike, Timelike, Duration, NaiveDateTime, NaiveDate, NaiveTime};

#[derive (Debug, Clone)]
pub struct SimState {
    pub load: f32, // watts
    pub battery_capacity: f32, // Wh
    pub current_stored_energy: f32, // Wh
    pub solar_nominal_output: f32, // watts
    pub charge_history: Vec<f32>, // Wh
    pub latitude: f32,
    pub history_dates: Vec<NaiveDateTime>,
    pub now: NaiveDateTime, 
    pub step_size: Duration,
    pub start_day: u32,
    pub end_day: u32
}
impl SimState {
    pub fn new() -> SimState {
        SimState {
            load: 0.,
            battery_capacity: 0.,
            current_stored_energy: 0.,
            solar_nominal_output: 0.,
            charge_history: Vec::new(),
            latitude: 0.,
            history_dates: Vec::new(),
            now:  NaiveDateTime::new(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), NaiveTime::from_hms_opt(0,0,0).unwrap()),
            step_size: Duration::hours(1),
            start_day: 1,
            end_day: 365
        }
    }
}

pub fn run_simulation(state: &SimState) -> SimState {
    let mut state = state.clone();
    state.now = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()
    .and_hms_opt(0, 0, 0).unwrap();
    state.charge_history = Vec::new();
    state.history_dates = Vec::new();
    let end = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
        .and_hms_opt(0, 0, 0).unwrap();

    while state.now < end {
        state = step(&state);
    }
    state.clone()
}

pub fn step(state: &SimState) -> SimState {
    let delta = net_energy(&state);

    let unbounded_charge = state.current_stored_energy + delta;

    let mut new_state = state.clone();
    new_state.charge_history.push(state.current_stored_energy);
    new_state.current_stored_energy = if unbounded_charge < 0. {
        0.
    } else if unbounded_charge > state.battery_capacity {
        state.battery_capacity
    } else {
        state.current_stored_energy + delta
    };
    new_state.now = state.now + state.step_size;
    new_state.history_dates.push(state.now);
    new_state
}

#[test]
fn test_step_1() {
    let mut state = SimState::new();
    state.now = NaiveDateTime::new(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), NaiveTime::from_hms_opt(12,0,0).unwrap());
    state.step_size = Duration::hours(2);
    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 0.;
    state.load = 20.;
    let net = step(&state);
    assert_eq!(net.current_stored_energy, 10.)
}

#[test]
fn test_step_2() {
    let mut state = SimState::new();
    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 10.;
    state.load = 20.;
    let net = step(&state);
    assert_eq!(net.current_stored_energy, 30.)
}

#[test]
fn test_step_3() {
    let mut state = SimState::new();
    state.now = NaiveDateTime::new(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), NaiveTime::from_hms_opt(12,0,0).unwrap());

    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 50.;
    state.load = 10.;
    let net = step(&state);
    assert_eq!(net.current_stored_energy, 90.)
}

pub fn net_energy(state: &SimState) -> f32 {
    let actual_solar_energy = solar_power(state.solar_nominal_output, state) * bounded_daylight_hours(
        state.now, 
        state.now + state.step_size, 
        daylight_hours(state.latitude, state.now.ordinal0()));
    let load_energy = state.load * state.step_size.num_minutes() as f32 / 60.;
    actual_solar_energy - load_energy
}

#[test]
fn test_net() {
    let mut state = SimState::new();
    state.now = NaiveDateTime::new(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), NaiveTime::from_hms_opt(12,0,0).unwrap());
    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 80.;
    state.load = 50.;
    let net = net_energy(&state);
    assert_eq!(net, 30.)
}

#[test]
fn test_net_2() {
    let mut state = SimState::new();
    state.now = NaiveDateTime::new(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), NaiveTime::from_hms_opt(12,0,0).unwrap());
    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 50.;
    state.load = 10.;
    let net = net_energy(&state);
    assert_eq!(net, 40.)
}

pub fn daylight_hours(lat: f32, day: u32) -> f32{

    let p = (0.39795*
        (0.2163108 + 2.*
            (0.9671396*
                (0.00860*(day as f32)).tan()
            ).atan()
        ).cos()
    ).asin();

//                           _                                         _
//                          / sin(0.8333*pi/180) + sin(L*pi/180)*sin(P) \
//    D = 24 - (24/pi)*acos{  -----------------------------------------  }
//                          \_          cos(L*pi/180)*cos(P)           _/
    let numerator = (0.8333*PI / 180.).sin() + (lat*PI/180.).sin()*p.sin();
    let denom = (lat*PI/180.).cos()*p.cos();
    let d = (24./PI)*(numerator/denom).acos();
    d
}

#[test]
fn test_daylight_1() {
    let error = (daylight_hours(0., 85) - 12.).abs();
    assert!(error < 0.15)
}

pub fn bounded_daylight_duration(start: NaiveDateTime, end: NaiveDateTime, daylight_hours: f32) -> Duration {
    let sunrise = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(start.year(), start.month(), start.day()).unwrap(), 
        NaiveTime::from_num_seconds_from_midnight_opt(((12. - daylight_hours / 2.)*60.*60.) as u32, 0).unwrap());
    let sunset = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(start.year(), start.month(), start.day()).unwrap(), 
        NaiveTime::from_num_seconds_from_midnight_opt(((12. + daylight_hours / 2.)*60.*60.) as u32, 0).unwrap());
    if end < sunrise || start > sunset {
        Duration::zero()
    } else {
        earlier_of(end, sunset) - later_of(start, sunrise)
    }
}

#[test]
fn test_bounded_daylight_duration_1() {
    let start =  NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 
        NaiveTime::from_hms_opt(12,0,0).unwrap());
    let end =  NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 
        NaiveTime::from_hms_opt(13,0,0).unwrap());
    assert_eq!(bounded_daylight_duration(start, end, 12.), Duration::hours(1))
}

pub fn bounded_daylight_hours(start: NaiveDateTime, end: NaiveDateTime, daylight_hours: f32) -> f32 {
    let dur = bounded_daylight_duration(start, end, daylight_hours);
    dur.num_seconds() as f32 / (60.*60.)
}

#[test]
fn test_bounded_daylight_hours_1() {
    let start =  NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 
        NaiveTime::from_hms_opt(12,0,0).unwrap());
    let end =  NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 
        NaiveTime::from_hms_opt(13,0,0).unwrap());
    assert_eq!(bounded_daylight_hours(start, end, 12.), 1.)
}

#[test]
fn test_bounded_daylight_hours_2() {
    let start =  NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 
        NaiveTime::from_hms_opt(12,0,0).unwrap());
    let dur =  Duration::hours(1);
    assert_eq!(bounded_daylight_hours(start, start + dur, 12.), 1.)
}

#[test]
fn test_bounded_daylight_hours_3() {
    let start =  NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 
        NaiveTime::from_hms_opt(5,15,0).unwrap());
    let end =  NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 
        NaiveTime::from_hms_opt(6,15,0).unwrap());
    assert_eq!(bounded_daylight_hours(start, end, 12.), 0.25)
}

pub fn later_of(a: NaiveDateTime, b: NaiveDateTime) -> NaiveDateTime {
    if a > b {
        a
    } else {
        b
    }
}

#[test]
fn test_time_comparison() {
    let noon =  NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 
        NaiveTime::from_hms_opt(12,0,0).unwrap());
    let nine_am =  NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), 
        NaiveTime::from_hms_opt(12,0,0).unwrap());
    assert_eq!(later_of(noon, nine_am), noon);
    assert_eq!(later_of(nine_am, noon), noon);
    assert_eq!(earlier_of(noon, nine_am), nine_am);
    assert_eq!(earlier_of(nine_am, noon), nine_am);
}

pub fn earlier_of(a: NaiveDateTime, b: NaiveDateTime) -> NaiveDateTime {
    if a < b {        
        a
    } else {
        b
    }
}

pub fn solar_power(nominal_power: f32, state: &SimState) -> f32 {
    let start = state.now;
    let end = state.now + state.step_size;
    let light_hours = daylight_hours(state.latitude, state.now.ordinal0());
    let sunrise = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(start.year(), start.month(), start.day()).unwrap(), 
        NaiveTime::from_num_seconds_from_midnight_opt(((12. - light_hours / 2.)*60.*60.) as u32, 0).unwrap());
    let sunset = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(start.year(), start.month(), start.day()).unwrap(), 
        NaiveTime::from_num_seconds_from_midnight_opt(((12. + light_hours / 2.)*60.*60.) as u32, 0).unwrap());
    if end < sunrise || start > sunset {
        0.
    } else {
        let start_coeff = solar_production_curve(later_of(start, sunrise).time(), light_hours);
        let end_coeff = solar_production_curve(earlier_of(end, sunset).time(), light_hours);
        let avg_coeff = (start_coeff + end_coeff)/2.;
        nominal_power * avg_coeff
    }
}

pub fn solar_production_curve(time: NaiveTime, light_hours: f32) -> f32 {
    let time_scaler = 2.*PI/light_hours;
    let hour = time.hour() as f32 + (time.minute() as f32)/60. + (time.second() as f32)/(60.*60.);
    0.5*(time_scaler*(hour+12.)).cos()+0.5
}

#[test]
fn test_solar_production_1() {
    let noon =  NaiveTime::from_hms_opt(12,0,0).unwrap();
    assert_eq!(solar_production_curve(noon, 12.), 1.);
}
#[test]
fn test_solar_production_2() {
    let six =  NaiveTime::from_hms_opt(6,0,0).unwrap();
    assert_eq!(solar_production_curve(six, 12.), 0.);
}

pub fn chart(
    xs: Vec<NaiveDateTime>, 
    ys: Vec<Vec<f32>>, 
    ys_secondary: Vec<Vec<f32>>,
    labels: Vec<String>, 
    title: Option<String>, 
    show_legend: bool) {

    let output_file = "Energy Plot.png";

    let root = BitMapBackend::new(output_file, (1024, 768)).into_drawing_area();
    let mut builder = ChartBuilder::on(&root);
    //use plotters::{prelude::*, style::Color};
    root.fill(&WHITE).unwrap();

    //const PLOT_LINE_COLOR: RGBColor = RGBColor(0, 175, 255);
    
    let from_date = *xs.first().clone().expect("No dates to display");
    let to_date = *xs.last().expect("No dates to display");

    let y_max: f32 = ys.iter().map(|y| 
        y.clone().into_iter().reduce(f32::max))
        .filter(|i| i.is_some())
        .map(|i| i.unwrap()).reduce(f32::max).unwrap();
    
    let y_secondary_max: f32 = ys_secondary.iter().map(|y| 
        y.clone().into_iter().reduce(f32::max))
        .filter(|i| i.is_some())
        .map(|i| i.unwrap()).reduce(f32::max).unwrap();

    let mut chart = if title.is_some(){
        builder
        .x_label_area_size(28_i32)
        .y_label_area_size(28_i32)
        .right_y_label_area_size(40)
        .margin(20_i32)
        .caption(title.clone().unwrap().as_str(), ("sans-serif", 30.0))
        .build_cartesian_2d(
            RangedDateTime::from(from_date..to_date), 
            0_f32..y_max*1.05).unwrap()
        .set_secondary_coord(
            RangedDateTime::from(from_date..to_date), 
            0_f32..y_secondary_max*1.05)
    } else {
        builder
            .x_label_area_size(28_i32)
            .y_label_area_size(28_i32)
            .right_y_label_area_size(40)
            .margin(20_i32)
            .build_cartesian_2d(
                RangedDateTime::from(from_date..to_date), 
                0_f32..y_max*1.05).unwrap()
            .set_secondary_coord(
                RangedDateTime::from(from_date..to_date), 
                0_f32..y_secondary_max*1.05)
            // .expect("Failed to build chart")
    };

    chart
        .configure_mesh()
        //.bold_line_style(plotters::style::colors::BLUE.mix(0.1))
        //.light_line_style(plotters::style::colors::BLUE.mix(0.05))
        //.axis_style(ShapeStyle::from(plotters::style::colors::BLUE.mix(0.45)).stroke_width(1))
        //.y_labels(10)
        .x_labels(6)
        .x_label_formatter(&|x| format!("{}-{}-{}", x.day(), x.month(), x.year()))
        //.y_label_style(
        //    ("sans-serif", 15)
        //        .into_font()
        //        .color(&plotters::style::colors::BLUE.mix(0.65))
        //        .transform(FontTransform::Rotate90),
        //)
        .y_label_formatter(&|y| format!("{}", y))
        .y_desc("Battery Charge")
        .draw()
        .expect("failed to draw chart mesh");

    chart
        .configure_secondary_axes()
        .y_desc("Daylight Hours")
        .draw().unwrap();

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
    let n = vec![ys.len(), colors.len(), labels.len()].iter().min().unwrap_or(&1).clone() as usize;

    for i in 0..n {
        let this_data: Vec<(NaiveDateTime,f32)> = xs.clone().into_iter()
            .zip(ys[i.clone()].clone().into_iter()).collect();
        let this_color = colors[color_index];
        let this_label = labels[i].clone();
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


    let n = vec![ys_secondary.len(), colors.len(), labels.len()].iter().min().unwrap_or(&1).clone() as usize;

    for i in 0..n {
        let this_data: Vec<(NaiveDateTime,f32)> = xs.clone().into_iter()
            .zip(ys_secondary[i.clone()].clone().into_iter()).collect();
        let this_color = colors[color_index];
        let this_label = labels[color_index].clone();
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

    if show_legend {
        chart.configure_series_labels()
        .background_style(&WHITE)
        .border_style(&BLACK)
        .draw().expect("Failed to draw legend")    
    }
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", output_file);
}