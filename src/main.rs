use transient_solar_battery_simulation::*;
use chrono::{Datelike, NaiveDate};

fn main() {
    // Create state
    let mut state = State::new();
    state.solar_nominal_output = 43.;
    state.latitude = 20.;
    state.battery_capacity = 300.;
    state.current_stored_energy = 150.;
    state.loads.push(10.);

    let mut durations = Vec::new();

    // Iterate
    let end = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
        .and_hms_opt(0, 0, 0).unwrap();

    while state.now < end {
        state = step(&state);
        durations.push(daylight_hours(state.latitude, state.now.ordinal0()));
    }
    chart(
        state.history_dates, 
        vec![state.charge_history], 
    vec![durations],
        vec!["State of Charge".to_string(), "Daylight Hours".to_string()], 
        None, 
        true);
}
