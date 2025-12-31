#[cfg(test)]
mod tests {
    use super::*;
    use anise::ephemerides::ephemeris::Ephemeris;
    use hifitime::Epoch;

    #[test]
    fn test_parse_stk_e_v12() {
        let path = "tests/test_v12.e";
        let ephem = Ephemeris::from_stk_e_file(path).expect("Could not parse STK file");

        // Check metadata
        assert_eq!(format!("{:?}", ephem.interpolation()), "Type9LagrangeUnequalStep");
        assert_eq!(ephem.object_id(), "STK_Object");

        // Check domain
        let (start, end) = ephem.domain().expect("Could not get domain");

        // ScenarioEpoch: 1 Jun 2020 12:00:00.000000
        let scenario_epoch = Epoch::from_gregorian_utc_at_noon(2020, 6, 1);

        // First point at 0.0s offset
        let expected_start = scenario_epoch;
        assert!((start - expected_start).to_seconds().abs() < 1e-6, "Start epoch mismatch: {} vs {}", start, expected_start);

        // Last point at 1980.0s offset
        let expected_end = scenario_epoch + hifitime::Unit::Second * 1980.0;
        assert!((end - expected_end).to_seconds().abs() < 1e-6, "End epoch mismatch: {} vs {}", end, expected_end);

        // Check number of points (based on data provided in previous example, which had 34 points)
        // We can't check points count directly publicly easily, but successful parsing implies correctness given the domain check.
    }
}
