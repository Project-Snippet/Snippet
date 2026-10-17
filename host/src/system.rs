use snippet_protocol::SystemStatus;
use starship_battery::units::ratio::percent;
use starship_battery::{Manager, State};

/// Turn a battery charge (already in percent, e.g. 87.4) into a 0-100 byte.
/// Out-of-range or garbage readings are clamped rather than trusted.
pub fn to_percent(value: f32) -> u8 {
    if value.is_nan() {
        return 0;
    }
    value.round().clamp(0.0, 100.0) as u8
}

/// Only an actively charging battery counts as charging.
pub fn is_charging(state: State) -> bool {
    matches!(state, State::Charging)
}

/// Charge and charging flag of the first battery, or `None` when the machine
/// has no battery (a desktop) or the OS will not tell us.
pub fn battery() -> Option<(u8, bool)> {
    let manager = Manager::new().ok()?;
    let battery = manager.batteries().ok()?.next()?.ok()?;
    let charge = to_percent(battery.state_of_charge().get::<percent>());
    Some((charge, is_charging(battery.state())))
}

/// A snapshot of everything we can read so far.
/// Volume and wifi are placeholders until they are implemented.
pub fn read() -> SystemStatus {
    let battery = battery();
    SystemStatus {
        battery_percent: battery.map(|(charge, _)| charge),
        charging: battery.is_some_and(|(_, charging)| charging),
        volume_percent: 0,
        muted: false,
        wifi_signal_percent: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentages_are_rounded_to_whole_numbers() {
        assert_eq!(to_percent(87.4), 87);
        assert_eq!(to_percent(87.6), 88);
        assert_eq!(to_percent(0.0), 0);
        assert_eq!(to_percent(100.0), 100);
    }

    #[test]
    fn bad_readings_are_clamped_not_trusted() {
        assert_eq!(to_percent(-5.0), 0);
        assert_eq!(to_percent(250.0), 100);
        assert_eq!(to_percent(f32::NAN), 0);
        assert_eq!(to_percent(f32::INFINITY), 100);
    }

    #[test]
    fn only_charging_state_counts_as_charging() {
        assert!(is_charging(State::Charging));
        assert!(!is_charging(State::Discharging));
        assert!(!is_charging(State::Full));
        assert!(!is_charging(State::Unknown));
    }

    #[test]
    fn whatever_the_machine_reports_is_a_valid_status() {
        assert!(read().is_valid());
    }
}
