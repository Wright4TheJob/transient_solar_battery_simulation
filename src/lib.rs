pub mod chart;
pub mod gui;
pub mod instant_load;
use crate::instant_load::InstantLoad;
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use std::collections::HashMap;
use std::f32::consts::PI;
#[derive(Debug, Clone)]
pub struct SimState {
    pub load: f32,                  // watts
    pub battery_capacity: f32,      // Wh
    pub current_stored_energy: f32, // Wh
    pub solar_nominal_output: f32,  // watts
    pub charge_history: Vec<f32>,   // Wh
    pub latitude: f32,
    pub history_dates: Vec<NaiveDateTime>,
    pub now: NaiveDateTime,
    pub step_size: Duration,
    pub start_day: u32,
    pub end_day: u32,
    pub solar_history: Vec<f32>,
    pub daylight_history: Vec<f32>,
    pub initial_charge: f32,
    pub reduced_power_percent: f32,
    pub reduced_power_days: usize,
    pub reduced_power_days_cycle: usize,
    day_in_cycle: usize,
    pub point_loads: HashMap<usize, InstantLoad>,
}
impl SimState {
    pub fn new() -> SimState {
        SimState {
            load: 25.,
            battery_capacity: 1000.,
            solar_nominal_output: 100.,
            current_stored_energy: 0.,
            charge_history: Vec::new(),
            latitude: 36.,
            history_dates: Vec::new(),
            now: NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            ),
            step_size: Duration::minutes(45),
            start_day: 1,
            end_day: 364,
            solar_history: Vec::new(),
            daylight_history: Vec::new(),
            initial_charge: 0.,
            reduced_power_percent: 1.0,
            reduced_power_days: 1,
            reduced_power_days_cycle: 2,
            day_in_cycle: 1,
            point_loads: HashMap::new(),
        }
    }

    pub fn read_point_loads(&self) -> f32 {
        self.point_loads.values().map(|l| l.energy).sum::<f32>()
    }
    pub fn run_simulation(&mut self) {
        self.now = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .with_ordinal(match self.start_day {
                0 => 1,
                _ => self.start_day,
            })
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        self.current_stored_energy = self.initial_charge * self.battery_capacity / 100.;
        self.charge_history = Vec::new();
        self.history_dates = Vec::new();
        self.solar_history = Vec::new();
        self.daylight_history = Vec::new();

        let end = NaiveDate::from_ymd_opt(2023, 12, 31)
            .unwrap()
            .with_ordinal(match self.end_day {
                0 => 1,
                _ => self.end_day,
            })
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        while self.now < end {
            self.step();
        }
    }
    pub fn step(&mut self) {
        let delta = self.net_energy();
        let unbounded_charge = self.current_stored_energy + delta;

        self.charge_history.push(self.current_stored_energy);
        self.current_stored_energy = clip(unbounded_charge, 0., self.battery_capacity);
        let last_day = self.now.day();
        self.now += self.step_size;
        if self.now.day() != last_day {
            self.day_in_cycle += 1;
            if self.day_in_cycle > self.reduced_power_days_cycle {
                self.day_in_cycle = 1;
            }
        }
        self.history_dates.push(self.now);
        self.solar_history.push(self.solar_power());
        self.daylight_history
            .push(daylight_hours(self.latitude, self.now.ordinal0()));
    }
    pub fn actual_solar_energy(&self) -> f32 {
        self.solar_power() * bounded_daylight_hours(self.now, self.now + self.step_size)
    }
    pub fn steady_load_energy(&self) -> f32 {
        self.load * self.step_size.num_minutes() as f32 / 60.
    }
    pub fn point_load_energy(&self) -> f32 {
        self.point_loads
            .values()
            .map(|l| l.energy(self.now, self.step_size))
            .sum::<f32>()
    }
    pub fn net_energy(&self) -> f32 {
        self.actual_solar_energy() - self.steady_load_energy() - self.point_load_energy()
    }
    #[must_use]
    pub fn solar_power(&self) -> f32 {
        let start = self.now;
        let end = self.now + self.step_size;

        let start_coeff = solar_production_curve(start, self.latitude);
        let end_coeff = solar_production_curve(end, self.latitude);
        let avg_coeff = f32::midpoint(start_coeff, end_coeff);
        let mut actual_solar_energy = self.solar_nominal_output * avg_coeff;

        if self.day_in_cycle <= self.reduced_power_days {
            actual_solar_energy *= self.reduced_power_percent;
        }
        actual_solar_energy
    }
}

impl Default for SimState {
    fn default() -> Self {
        let mut state = SimState::new();
        state.battery_capacity = 1000.;
        state.solar_nominal_output = 100.;
        state.load = 25.;
        state.latitude = 36.;
        state
    }
}

#[test]
fn test_clipping_min() {
    let lower = 0.;
    let upper = 1.;
    let val = -0.5;
    assert_eq!(clip(val, lower, upper), 0.)
}

#[test]
fn test_clip_max() {
    let lower = 0.;
    let upper = 1.;
    let val = 1.5;
    assert_eq!(clip(val, lower, upper), 1.)
}

#[test]
fn test_clip() {
    let lower = 0.;
    let upper = 1.;
    let val = 0.5;
    assert_eq!(clip(val, lower, upper), 0.5)
}

fn clip<T: PartialOrd>(val: T, lower: T, upper: T) -> T {
    if val < lower {
        lower
    } else if val > upper {
        upper
    } else {
        val
    }
}

#[test]
fn test_step_1() {
    let mut state = SimState::new();
    state.now = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
    );
    state.step_size = Duration::hours(2);
    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 0.;
    state.load = 20.;
    state.step();
    assert_eq!(state.current_stored_energy, 10.)
}

#[test]
fn test_step_2() {
    let mut state = SimState::new();
    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 10.;
    state.load = 20.;
    state.step();
    assert_eq!(state.current_stored_energy, 40.)
}

pub fn daylight_hours(lat: f32, day: u32) -> f32 {
    let p = (0.39795
        * (0.2163108 + 2. * (0.9671396 * (0.00860 * (day as f32)).tan()).atan()).cos())
    .asin();

    //                           _                                         _
    //                          / sin(0.8333*pi/180) + sin(L*pi/180)*sin(P) \
    //    D = 24 - (24/pi)*acos{  -----------------------------------------  }
    //                          \_          cos(L*pi/180)*cos(P)           _/
    let numerator = 0.8333_f32.to_radians().sin() + lat.to_radians().sin() * p.sin();
    let denom = (lat * PI / 180.).cos() * p.cos();
    (24. / PI) * (numerator / denom).acos()
}

#[test]
fn test_daylight_1() {
    let error = (daylight_hours(0., 85) - 12.).abs();
    assert!(error < 0.15)
}

pub fn bounded_daylight_duration(start: NaiveDateTime, end: NaiveDateTime, lat: f32) -> Duration {
    let sunrise = sunrise(start.date(), lat);
    let sunset = sunset(start.date(), lat);
    if end.time() < sunrise || start.time() > sunset {
        Duration::zero()
    } else {
        earlier_of(end.time(), sunset) - later_of(start.time(), sunrise)
    }
}
#[test]
fn test_bounded_daylight_duration_1() {
    let start = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
    );
    let end = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
    );
    assert_eq!(
        bounded_daylight_duration(start, end, 0.),
        Duration::hours(1)
    )
}

pub fn bounded_daylight_hours(start: NaiveDateTime, end: NaiveDateTime) -> f32 {
    let dur = bounded_daylight_duration(start, end, 0.);
    dur.num_seconds() as f32 / (60. * 60.)
}

#[test]
fn test_bounded_daylight_hours_1() {
    let start = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
    );
    let end = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
    );
    assert_eq!(bounded_daylight_hours(start, end), 1.)
}

#[test]
fn test_bounded_daylight_hours_2() {
    let start = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
    );
    let dur = Duration::hours(1);
    assert_eq!(bounded_daylight_hours(start, start + dur), 1.)
}

#[test]
fn test_bounded_daylight_hours_3() {
    let start = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        NaiveTime::from_hms_opt(5, 15, 0).unwrap(),
    );
    let end = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        NaiveTime::from_hms_opt(6, 15, 0).unwrap(),
    );
    assert_eq!(bounded_daylight_hours(start, end), 0.25)
}

pub fn later_of<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

#[test]
fn test_time_comparison() {
    let noon = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
    );
    let nine_am = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
    );
    assert_eq!(later_of(noon, nine_am), noon);
    assert_eq!(later_of(nine_am, noon), noon);
    assert_eq!(earlier_of(noon, nine_am), nine_am);
    assert_eq!(earlier_of(nine_am, noon), nine_am);
}

pub fn earlier_of<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

pub fn sunrise(date: NaiveDate, lat: f32) -> NaiveTime {
    let light_hours = daylight_hours(lat, date.ordinal0());
    NaiveTime::from_num_seconds_from_midnight_opt(
        43200 - ((light_hours / 2.) * 60. * 60.) as u32,
        0,
    )
    .unwrap()
}
#[test]
fn test_sunrise_1() {
    let date = NaiveDate::from_ymd_opt(2023, 3, 15).unwrap();
    assert_eq!(sunrise(date, 45.).hour(), 6)
}
pub fn sunset(date: NaiveDate, lat: f32) -> NaiveTime {
    let light_hours = daylight_hours(lat, date.ordinal0());
    NaiveTime::from_num_seconds_from_midnight_opt(
        43200 + ((light_hours / 2.) * 60. * 60.) as u32,
        0,
    )
    .unwrap()
}

#[test]
fn test_solar_power_2() {
    let mut state = SimState::new();
    state.now = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 3, 1).unwrap(),
        NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
    );
    state.step_size = Duration::seconds(1);
    state.solar_nominal_output = 1.;
    let net = state.solar_power();
    assert!((net - 0.33).abs() < 0.01)
}
#[must_use]
pub fn time_hours(time: NaiveTime) -> f32 {
    time.hour() as f32 + (time.minute() as f32) / 60. + (time.second() as f32) / (60. * 60.)
}
#[test]
fn test_time_hours() {
    let time = NaiveTime::from_hms_opt(1, 30, 0).unwrap();
    assert_eq!(time_hours(time), 1.5)
}
#[must_use]
pub fn solar_production_curve(now: NaiveDateTime, lat: f32) -> f32 {
    let light_hours = daylight_hours(lat, now.ordinal0());
    let rise = sunrise(now.date(), lat);
    let set = sunset(now.date(), lat);
    let hour = time_hours(now.time());

    if now.time() <= rise || now.time() >= set {
        0.
    } else {
        let time_scaler = (2. * PI) / light_hours;
        let cos_part = (time_scaler * (hour - 12.)).cos();
        0.5 * cos_part + 0.5
    }
}

#[test]
fn test_solar_production_curve() {
    let mut i = 0.;
    let mut hist = Vec::new();
    let mut now = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
    );
    while i < 48. {
        hist.push(solar_production_curve(now, 38.));
        now += Duration::minutes(30);
        i += 1.;
    }
    assert!(false)
}

#[test]
fn test_solar_production_2() {
    let six = NaiveDateTime::new(
        NaiveDate::default(),
        NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
    );
    assert_eq!(solar_production_curve(six, 12.), 0.);
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadFrequency {
    Hourly,
    Daily,
    Weekly,
}
impl LoadFrequency {
    #[must_use]
    pub fn index(&self) -> usize {
        match self {
            LoadFrequency::Hourly => 0,
            LoadFrequency::Daily => 1,
            LoadFrequency::Weekly => 2,
        }
    }
}
impl std::fmt::Display for LoadFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Hourly => "Hourly",
            Self::Daily => "Daily",
            Self::Weekly => "Weekly",
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
impl Weekday {
    #[must_use]
    pub fn index(&self) -> usize {
        match self {
            Weekday::Monday => 0,
            Weekday::Tuesday => 1,
            Weekday::Wednesday => 2,
            Weekday::Thursday => 3,
            Weekday::Friday => 4,
            Weekday::Saturday => 5,
            Weekday::Sunday => 6,
        }
    }
}
impl std::fmt::Display for Weekday {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Monday => "Monday",
            Self::Tuesday => "Tuesday",
            Self::Wednesday => "Wednesday",
            Self::Thursday => "Thursday",
            Self::Friday => "Friday",
            Self::Saturday => "Saturday",
            Self::Sunday => "Sunday",
        })
    }
}
