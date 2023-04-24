use chrono::{Datelike, DateTime, Duration, Utc, NaiveDate, Weekday};

#[derive (Debug, Clone, Default)]
pub struct State {
    loads: Vec<f32>, // watts
    battery_capacity: f32, // Wh
    current_stored_energy: f32, // Wh
    solar_nominal_output: f32, // watts
    charge_history: Vec<f32> // Wh
}

pub fn net_energy(duration: Duration, state: &State) -> f32 {
    let net_power = state.solar_nominal_output - total_load(&state);
    net_power*duration.num_hours() as f32
}

#[test]
fn test_net() {
    let duration = Duration::hours(2);
    let mut state = State::default();
    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 100.;
    state.loads = vec![10.,10.,30.];
    let net = net_energy(duration, &state);
    assert_eq!(net, 100.)
}

#[test]
fn test_net_2() {
    let duration = Duration::hours(2);
    let mut state = State::default();
    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 50.;
    state.loads = vec![10.];
    let net = net_energy(duration, &state);
    assert_eq!(net, 80.)
}

pub fn total_load(state: &State) -> f32 {
    state.loads.iter().sum()
}
#[test]
fn test_total_loads() {
    let mut state = State::default();
    state.loads = vec![10.,10.,30.];
    let total = total_load(&state);
    assert_eq!(total, 50.)
}


pub fn step(duration: Duration, state: &State) -> State {
    let delta = net_energy(duration, &state);
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
    let duration = Duration::hours(2);
    let mut state = State::default();
    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 0.;
    state.loads = vec![10.,10.];
    let net = step(duration, &state);
    assert_eq!(net.current_stored_energy, 10.)
}

#[test]
fn test_step_2() {
    let duration = Duration::hours(2);
    let mut state = State::default();
    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 10.;
    state.loads = vec![10.,10.];
    let net = step(duration, &state);
    assert_eq!(net.current_stored_energy, 30.)
}

#[test]
fn test_step_3() {
    let duration = Duration::hours(2);
    let mut state = State::default();
    state.battery_capacity = 100.;
    state.current_stored_energy = 50.;
    state.solar_nominal_output = 50.;
    state.loads = vec![10.];
    let net = step(duration, &state);
    assert_eq!(net.current_stored_energy, 100.)
}

pub fn daylight_hours() {
//     P = asin[.39795*cos(.2163108 + 2*atan{.9671396*tan[.00860(J-186)]})]

//                           _                                         _
//                          / sin(0.8333*pi/180) + sin(L*pi/180)*sin(P) \
//    D = 24 - (24/pi)*acos{  -----------------------------------------  }
//                          \_          cos(L*pi/180)*cos(P)           _/
}