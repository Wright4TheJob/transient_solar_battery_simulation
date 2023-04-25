use transient_solar_battery_simulation::*;
use chrono::{Datelike, DateTime, Duration, NaiveDate, NaiveDateTime};

fn main() {
    // Create state
    let mut state = State::default();
    state.solar_nominal_output = 20.;
    state.battery_capacity = 150.;
    state.loads.push(10.);
    println!("{:?}", state);

    let mut dates = Vec::new();
    let mut durations = Vec::new();

    // Iterate
    let mut t = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()
        .and_hms_opt(0, 0, 0).unwrap();
    let end = NaiveDate::from_ymd_opt(2023, 2, 1).unwrap()
        .and_hms_opt(0, 0, 0).unwrap();

    while t < end {
        state = step(Duration::hours(1), &state);

        dates.push(t.clone());

        t += Duration::hours(1);
        durations.push(daylight_hours(36., t.ordinal0()));
    }
    chart(
        dates, 
        vec![state.charge_history], 
    vec![durations],
        vec!["State of Charge".to_string(), "Daylight Hours".to_string()], 
        None, 
        true);
}
