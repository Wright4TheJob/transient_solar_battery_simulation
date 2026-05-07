use crate::gui::AppState;
use iced::application;
use transient_solar_battery_simulation::*;

use iced_aw::ICED_AW_FONT_BYTES;

pub fn main() -> iced::Result {
    application(AppState::new, AppState::update, AppState::view)
        .font(ICED_AW_FONT_BYTES)
        .run()
}
