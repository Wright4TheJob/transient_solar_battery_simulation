use std::f32::consts::PI;
use plotters::prelude::*;
use plotters::coord::types::RangedDateTime;

use chrono::{Datelike, Timelike, Duration, NaiveDateTime, NaiveDate, NaiveTime};

#[derive (Debug, Clone)]
pub struct State {
    pub loads: Vec<f32>, // watts
    pub battery_capacity: f32, // Wh
    pub current_stored_energy: f32, // Wh
    pub solar_nominal_output: f32, // watts
    pub charge_history: Vec<f32>, // Wh
    pub latitude: f32,
    pub ordinal_day: u32,
    pub history_dates: Vec<NaiveDateTime>,
    pub now: NaiveDateTime, 
    pub step_size: Duration
}
impl State {
    pub fn new() -> State {
        State {
            loads: Vec::new(),
            battery_capacity: 0.,
            current_stored_energy: 0.,
            solar_nominal_output: 0.,
            charge_history: Vec::new(),
            latitude: 0.,
            ordinal_day: 0,
            history_dates: Vec::new(),
            now:  NaiveDateTime::new(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), NaiveTime::from_hms_opt(0,0,0).unwrap()),
            step_size: Duration::hours(1)
        }
    }
}

pub fn net_energy(state: &State) -> f32 {
    let net_power = state.solar_nominal_output - total_load(&state);
    net_power*bounded_daylight_hours(state.now, state.now + state.step_size, daylight_hours(state.latitude, state.ordinal_day))
}

#[test]
fn test_net() {
    let mut state = State::new();
    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 100.;
    state.loads = vec![10.,10.,30.];
    let net = net_energy(&state);
    assert_eq!(net, 100.)
}

#[test]
fn test_net_2() {
    let mut state = State::new();
    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 50.;
    state.loads = vec![10.];
    let net = net_energy(&state);
    assert_eq!(net, 80.)
}

pub fn total_load(state: &State) -> f32 {
    state.loads.iter().sum()
}
#[test]
fn test_total_loads() {
    let mut state = State::new();
    state.loads = vec![10.,10.,30.];
    let total = total_load(&state);
    assert_eq!(total, 50.)
}


pub fn step(state: &State) -> State {
    let delta = net_energy(&state);
    let effective_delta = if delta < -state.current_stored_energy {
        -state.current_stored_energy
    } else if delta + state.current_stored_energy > state.battery_capacity {
        state.battery_capacity - state.current_stored_energy
    } else {
        delta
    };
    let mut new_state = state.clone();
    new_state.charge_history.push(state.current_stored_energy + effective_delta);
    new_state.current_stored_energy += effective_delta;
    new_state
}

#[test]
fn test_step_1() {
    let mut state = State::new();
    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 0.;
    state.loads = vec![10.,10.];
    let net = step(&state);
    assert_eq!(net.current_stored_energy, 10.)
}

#[test]
fn test_step_2() {
    let mut state = State::new();
    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 10.;
    state.loads = vec![10.,10.];
    let net = step(&state);
    assert_eq!(net.current_stored_energy, 30.)
}

#[test]
fn test_step_3() {
    let mut state = State::new();
    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 50.;
    state.loads = vec![10.];
    let net = step(&state);
    assert_eq!(net.current_stored_energy, 100.)
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
    let sunrise = start.clone();
    sunrise.with_hour((12. - daylight_hours/2.) as u32);
    let sunset = start.clone();
    sunset.with_hour((12. - daylight_hours/2.) as u32);

    if start < sunrise && end > sunrise && end < sunset {
        end - sunrise
    } else if start > sunrise && end < sunset{
        sunset - sunrise
    } else if start > sunrise && start < sunset && end > sunset {
        sunset - start
    } else {
        Duration::zero()
    }
}

pub fn bounded_daylight_hours(start: NaiveDateTime, end: NaiveDateTime, daylight_hours: f32) -> f32 {
    let dur = bounded_daylight_duration(start, end, daylight_hours);
    dur.num_hours() as f32 + dur.num_hours() as f32 / 60.
}

pub fn solar_power(nominal_power: f32, start: NaiveDateTime, end: NaiveDateTime, daylight_hours: f32) -> f32 {
    nominal_power
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