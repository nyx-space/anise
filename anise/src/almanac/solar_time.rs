/*
 * ANISE Toolkit
 * Copyright (C) 2023-onward Google Inc. <opensource@google.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::{Epoch, TimeScale, TimeUnits};
use crate::NaifId;
use crate::errors::SpiceError; // Assuming a general error type, might need to create a more specific one.
use crate::constants::celestial_objects; // For NAIF IDs like EARTH

/// Calculates the Local True Solar Time (LTST) from an Ephemeris Time (ET) and longitude.
///
/// # Arguments
///
/// * `et`: The Ephemeris Time as a hifitime::Epoch.
/// * `longitude_deg`: The longitude in degrees (East positive).
///
/// # Returns
///
/// * `Result<f64, SpiceError>`: The Local True Solar Time in seconds past local midnight,
/// or an error if the calculation fails.
///
/// # Algorithm Details (based on SPICE et2lst):
/// 1. Convert ET to TAI.
/// 2. Convert TAI to UTC.
/// 3. Convert UTC to UT1. (Requires DUT1 = UT1 - UTC, typically from EOP data)
/// 4. Calculate Julian Date for UT1 (JD_UT1).
/// 5. Calculate Greenwich Mean Sidereal Time (GMST) from JD_UT1.
///    GMST (in seconds) = polynomial in T_UT1 (Julian centuries since J2000.0 for UT1)
///    T_UT1 = (JD_UT1 - 2451545.0) / 36525.0
///    Common formula for GMST (approximate, from Astronomical Almanac / Vallado):
///    GMST_seconds = 24110.54841 + 8640184.812866 * T_UT1 + 0.093104 * T_UT1^2 - 6.2e-6 * T_UT1^3
///    This needs to be modulo 86400 seconds.
/// 6. Calculate Local Mean Solar Time (LMST).
///    LMST (radians) = GMST (radians) + longitude (radians)
///    LMST (seconds) = (GMST_seconds + longitude_deg * 240.0) mod 86400
///    (Note: 1 degree longitude = 240 seconds of time; 360 deg = 86400 s)
/// 7. Calculate Mean Anomaly of the Sun.
/// 8. Calculate Equation of Time (EOT).
///    EOT (seconds) = complex calculation involving mean anomaly, solar longitude, etc.
/// 9. Calculate Local True Solar Time (LTST).
///    LTST (seconds) = LMST (seconds) + EOT (seconds)
/// 10. Normalize LTST to be within [0, 86400) seconds.

pub fn et2lst(et: Epoch, longitude_deg: f64) -> Result<f64, SpiceError> {
    // Ensure the input epoch is in Ephemeris Time (ET / TDB)
    if et.time_scale != TimeScale::TDB {
        // hifitime uses TDB for Ephemeris Time. SPICE ET is TDB.
        return Err(SpiceError::BadTimeScale {
            expected: TimeScale::TDB,
            actual: et.time_scale,
        });
    }

    // Step 1: Ensure input epoch is TDB (ET). This is already done by the check above.

    // Step 2: Convert ET (TDB) to UTC using hifitime.
    // `et` is the input Epoch in TDB scale.
    let utc_epoch = et.to_utc(); // This conversion correctly handles leap seconds internally.

    // Step 3: Convert UTC to UT1.
    // Accurate UT1 = UTC + DUT1. DUT1 (UT1 - UTC) must be obtained from Earth Orientation Parameters (EOP).
    // SPICE typically sources DUT1 from a leapseconds kernel (LSK) or a text PCK (e.g., values like BODY399_DUT1).
    // Hifitime itself does not bundle a full EOP dataset or LSK parser for DUT1.
    // TODO: CRITICAL - Implement a mechanism to fetch/query DUT1.
    // This could involve:
    //   a) Looking up values from a `PlanetaryConstants` structure if ANISE parses them from PCKs.
    //   b) Integrating a library or mechanism to fetch/interpolate IERS EOP data.
    // For now, DUT1 is assumed to be 0.0 as a placeholder. This will lead to inaccuracies.
    let dut1_placeholder_seconds = 0.0; // [seconds]

    let ut1_epoch = utc_epoch + dut1_placeholder_seconds.seconds();

    // Step 4: Calculate Julian Date for UT1 (JD_UT1).
    // Check if ANISE's hifitime dependency enables the "ut1" feature.
    // This determines if `to_jd_ut1_days()` is available.
    // The subtask will replace `false` with true if the "ut1" feature is found in Cargo.toml.
    let hifitime_ut1_feature_enabled: bool = false;

    let jd_ut1 = if hifitime_ut1_feature_enabled {
        ut1_epoch.to_jd_ut1_days()
    } else {
        // Fallback if "ut1" feature is not enabled in hifitime.
        // Using JD UTC as an approximation for JD UT1.
        // This adds to the inaccuracy if DUT1 is non-zero.
        // TODO: Replace with proper logging if "ut1" feature remains disabled.
        // println!("Warning: hifitime 'ut1' feature may not be enabled. Using JD UTC as approximation for JD UT1.");
        ut1_epoch.to_jd_utc()
    };

    // Step 5: Calculate Greenwich Mean Sidereal Time (GMST) from JD_UT1.
    // T_UT1 is Julian centuries since J2000.0 (which is JD 2451545.0).
    let jd_j2000 = 2451545.0;
    let t_ut1 = (jd_ut1 - jd_j2000) / 36525.0;

    // Formula for GMST in seconds (e.g., from Astronomical Almanac, Vallado, SPICE `timedai_` routine).
    // This formula gives GMST in seconds for a UT1 day.
    // GMST (seconds) = 24110.54841 + 8640184.812866 * T_UT1 + 0.093104 * T_UT1^2 - 6.2e-6 * T_UT1^3
    // Some sources might use slightly different coefficients or more terms.
    // Using coefficients similar to those found in various astronomical sources.
    let gmst_seconds_unnormalized = 24110.54841
        + 8640184.812866 * t_ut1
        + 0.093104 * t_ut1.powi(2)
        - 6.2e-6 * t_ut1.powi(3);

    // GMST needs to be in the range [0, 86400) seconds.
    // One solar day is 86400 nominal seconds.
    // One sidereal day is approx 86164.0905 solar seconds.
    // The formula above gives GMST related to the UT1 day.
    let seconds_in_day = 86400.0;
    let gmst_seconds = gmst_seconds_unnormalized % seconds_in_day;
    // Ensure result is positive if modulo results in negative
    let gmst_seconds = if gmst_seconds < 0.0 {
        gmst_seconds + seconds_in_day
    } else {
        gmst_seconds
    };

    // Step 6: Calculate Local Mean Solar Time (LMST).
    // LMST (seconds) = (GMST_seconds + longitude_in_time_seconds) mod seconds_in_day
    // 1 degree of longitude corresponds to 240 seconds of time (86400 seconds / 360 degrees).
    let longitude_in_time_seconds = longitude_deg * 240.0;

    let lmst_seconds_unnormalized = gmst_seconds + longitude_in_time_seconds;
    let lmst_seconds = lmst_seconds_unnormalized % seconds_in_day;
    // Ensure result is positive if modulo results in negative
    let lmst_seconds = if lmst_seconds < 0.0 {
        lmst_seconds + seconds_in_day
    } else {
        lmst_seconds
    };

    // Step 7: Calculate Mean Anomaly of the Sun.
    // `d_j2000` is the number of days (including fractional days) since J2000.0 (JD 2451545.0) for UT1.
    let d_j2000 = jd_ut1 - jd_j2000; // jd_j2000 was defined in Step 5

    // Mean anomaly of the Sun (degrees), common formula.
    let mean_anomaly_deg = (357.5291 + 0.98560028 * d_j2000).rem_euclid(360.0);
    let mean_anomaly_rad = mean_anomaly_deg.to_radians();

    // Step 8: Calculate Equation of Time (EOT).
    // EOT accounts for the difference between mean solar time and apparent (true) solar time.
    // Using a common approximation formula for EOT in minutes, then converting to seconds.
    // M (mean_anomaly_rad) is used here.
    // Coefficients from various sources, e.g., Meeus, Wikipedia.
    // EOT_minutes = -7.659 * sin(M) + 9.863 * sin(2*M + 3.5932) // Example, ensure consistency
    // Another common set of coefficients for EOT in minutes:
    // eot_m = -7.655 * sin(M) + 9.873 * sin(2*M + C) where C is an offset.
    // Let's use a well-known one:
    // L0 = (280.4665 + 0.98564736 * d_j2000) % 360  (Mean Longitude of Sun)
    // M = (357.5291 + 0.98560028 * d_j2000) % 360 (Mean Anomaly of Sun - already calculated)
    // E = M + (1.9148 * sin(M_rad) + 0.0200 * sin(2*M_rad) + 0.0003 * sin(3*M_rad)) * (180.0/PI) (Eccentric Anomaly related term for true anomaly)
    // L_true = L0 + (1.9148 * sin(M_rad) + 0.0200 * sin(2*M_rad) + 0.0003 * sin(3*M_rad))
    // These are getting complex and might require solar ephemeris for L0.
    //
    // Sticking to the simpler EOT formula based directly on Mean Anomaly (M):
    // (Formula from Wikipedia / various astronomical resources, often gives EOT in minutes)
    // EOT (minutes) ≈ −7.655 sin(M) + 9.873 sin(2M + 3.588 radians)
    // Note: The phase angle 3.588 rad is approx 205.5 degrees.
    let eot_minutes = -7.655 * mean_anomaly_rad.sin()
                      + 9.873 * (2.0 * mean_anomaly_rad + 3.588).sin();
    let eot_seconds = eot_minutes * 60.0;

    // TODO: Consider if a more precise EOT calculation is needed,
    // potentially using PCK data (e.g., BODYXXX_EQN_OF_TIME coefficients) if ANISE supports it,
    // or a more detailed astronomical series. For now, this approximation is used.

    // Step 9: Calculate Local True Solar Time (LTST).
    // LTST (seconds) = LMST (seconds) + EOT (seconds)
    let ltst_seconds_unnormalized = lmst_seconds + eot_seconds;

    // Step 10: Normalize LTST to be within [0, 86400) seconds.
    let ltst_seconds = ltst_seconds_unnormalized.rem_euclid(seconds_in_day);
    // rem_euclid should already handle negative results correctly for f64,
    // resulting in a value in [0, seconds_in_day) or (seconds_in_day, 0].
    // For positive divisor, it's [0, seconds_in_day).
    // If ltst_seconds_unnormalized is -1.0, ltst_seconds_unnormalized.rem_euclid(86400.0) is 86399.0.

    // Final check on longitude, though initial check should catch most issues.
    // Normalizing longitude_deg to the range [-180, 180) or [0, 360) before use
    // throughout the function would be more robust. For now, assume it's within reasonable bounds
    // or that the user handles extreme out-of-range values based on the initial error check.
    if longitude_deg < -180.0 - 1e-9 || longitude_deg > 360.0 + 1e-9 { // Added tolerance for strict check
        // This is more of a sanity check here, as initial validation should handle it.
        // Or, the longitude could be normalized at the beginning.
        return Err(SpiceError::InvalidLongitude { val: longitude_deg });
    }

    // Log the intermediate values if needed for debugging, especially DUT1 and EOT.
    // log::debug!(
    //     "et2lst calculation: et={:?}, lon_deg={}
"
    //     "  UTC={}, UT1_approx={}, JD_UT1_approx={}
"
    //     "  T_UT1={}, GMST_s={}
"
    //     "  longitude_time_s={}, LMST_s={}
"
    //     "  d_j2000={}, MeanAnomaly_deg={}, MeanAnomaly_rad={}
"
    //     "  EOT_minutes={}, EOT_s={}
"
    //     "  LTST_unnorm_s={}, LTST_s={}",
    //     et, longitude_deg,
    //     utc_epoch, ut1_epoch, jd_ut1,
    //     t_ut1, gmst_seconds,
    //     longitude_in_time_seconds, lmst_seconds,
    //     d_j2000, mean_anomaly_deg, mean_anomaly_rad,
    //     eot_minutes, eot_seconds,
    //     ltst_seconds_unnormalized, ltst_seconds
    // );


    Ok(ltst_seconds)
}

/// Calculates the Ephemeris Time (ET) from Local True Solar Time (LTST), longitude, and body ID.
///
/// # Arguments
///
/// * `lst_seconds`: Local True Solar Time in seconds past local midnight.
/// * `longitude_deg`: The longitude in degrees (East positive).
/// * `body_id`: The NAIF ID of the celestial body.
/// * `reference_et_for_date_estimation`: An approximate ET `Epoch` that is close to the expected output ET.
///                                      This is needed because LST is ambiguous (repeats daily),
///                                      and to resolve this ambiguity and perform date-specific
///                                      calculations (like EOT and UT1-UTC), we need a reference date.
///
/// # Returns
///
/// * `Result<Epoch, SpiceError>`: The Ephemeris Time as a hifitime::Epoch (in TDB time scale),
///                                or an error if the calculation fails.
///
/// # Algorithm Details (inverse of et2lst, simplified):
/// 1. Normalize `lst_seconds` to [0, 86400).
/// 2. Normalize `longitude_deg`.
/// 3. Handle `body_id` (currently primarily supports Earth).
/// 4. Estimate Julian Date (UT1) from `reference_et_for_date_estimation` to calculate date-specific EOT.
///    - Convert reference ET (TDB) to UTC, then to UT1 (using placeholder DUT1 for now).
///    - Calculate JD_UT1_ref from this UT1.
/// 5. Calculate Equation of Time (EOT) using JD_UT1_ref (same method as in `et2lst`).
/// 6. Calculate Local Mean Solar Time (LMST) from LTST:
///    LMST_seconds = (lst_seconds - EOT_seconds) mod 86400
/// 7. Calculate Greenwich Mean Sidereal Time (GMST) from LMST and longitude:
///    GMST_seconds = (LMST_seconds - longitude_time_seconds) mod 86400
/// 8. Calculate Julian Date UT1 (JD_UT1) from GMST. This is the tricky inverse of GMST calculation.
///    Requires an iterative solution or an approximation of T_UT1.
///    Let GMST_hours = GMST_seconds / 3600.0.
///    JD_UT1_approx = JD_at_0h_UT1_of_reference_date + (GMST_hours_UTC_corrected / 24.0) * (sidereal_day_seconds / solar_day_seconds)
///    This needs a reference Julian date at 0h UT1 for the *day* of `reference_et_for_date_estimation`.
/// 9. Convert JD_UT1 to an `Epoch` in UT1 scale.
///10. Convert UT1 Epoch to UTC Epoch (using placeholder DUT1).
///11. Convert UTC Epoch to ET (TDB) Epoch.

pub fn lst2et(
    mut lst_seconds: f64,
    mut longitude_deg: f64,
    body_id: NaifId,
    reference_et_for_date_estimation: Epoch,
) -> Result<Epoch, SpiceError> {
    let seconds_in_day = 86400.0;

    // Step 1: Normalize lst_seconds
    lst_seconds = lst_seconds.rem_euclid(seconds_in_day);

    // Step 2: Basic longitude validation (more robust normalization could be added)
    if longitude_deg < -180.0 - 1e-9 || longitude_deg > 360.0 + 1e-9 {
        return Err(SpiceError::InvalidLongitude { val: longitude_deg });
    }
    // TODO: Consider normalizing longitude_deg to a consistent range (e.g., 0-360 or -180 to 180)

    // Step 3: Handle body_id (currently only Earth is practically supported for EOT/GMST calculations)
    if body_id != celestial_objects::EARTH {
        // TODO: Extend support for other bodies if EOT/GMST calculations can be generalized
        // or if body-specific parameters (like rotation, EOT coefficients) can be fetched.
        return Err(SpiceError::UnsupportedBody { id: body_id });
    }

    // Ensure the reference epoch is in Ephemeris Time (ET / TDB)
    if reference_et_for_date_estimation.time_scale != TimeScale::TDB {
        return Err(SpiceError::BadTimeScale {
            expected: TimeScale::TDB,
            actual: reference_et_for_date_estimation.time_scale,
        });
    }

    // Step 4: Estimate Julian Date (UT1) from `reference_et_for_date_estimation`
    // This is to get a date context for calculating a date-specific EOT.
    let ref_utc_epoch = reference_et_for_date_estimation.to_utc();

    // Using the same placeholder for DUT1 as in et2lst.
    // TODO: CRITICAL - Replace with actual DUT1 lookup.
    let dut1_placeholder_seconds = 0.0;
    let ref_ut1_epoch = ref_utc_epoch + dut1_placeholder_seconds.seconds();

    // Check if ANISE's hifitime dependency enables the "ut1" feature.
    // This was determined during et2lst implementation. For consistency, let's re-evaluate or retrieve.
    // For this subtask, we'll assume it's false as per previous findings.
    // A more robust solution would pass this as a const or check Cargo.toml again.
    let hifitime_ut1_feature_enabled: bool = false; // Based on previous check in et2lst context

    let jd_ut1_ref = if hifitime_ut1_feature_enabled {
        ref_ut1_epoch.to_jd_ut1_days()
    } else {
        // Fallback if "ut1" feature is not enabled.
        ref_ut1_epoch.to_jd_utc()
    };

    // Step 5: Calculate Equation of Time (EOT) for the reference date.
    // This uses the same EOT calculation logic as in et2lst.
    let jd_j2000 = 2451545.0; // Julian Date of J2000.0
    let d_j2000_ref = jd_ut1_ref - jd_j2000;

    let mean_anomaly_deg_ref = (357.5291 + 0.98560028 * d_j2000_ref).rem_euclid(360.0);
    let mean_anomaly_rad_ref = mean_anomaly_deg_ref.to_radians();

    let eot_minutes_ref = -7.655 * mean_anomaly_rad_ref.sin()
                         + 9.873 * (2.0 * mean_anomaly_rad_ref + 3.588).sin();
    let eot_seconds_ref = eot_minutes_ref * 60.0;

    // TODO: Consider if a more precise EOT calculation is needed (consistent with et2lst's TODO).

    // Step 6: Calculate Local Mean Solar Time (LMST) from LTST and EOT_ref.
    // LMST_seconds = (lst_seconds - EOT_seconds_ref) mod seconds_in_day
    let lmst_seconds_unnormalized = lst_seconds - eot_seconds_ref;
    let lmst_seconds = lmst_seconds_unnormalized.rem_euclid(seconds_in_day);

    // Step 7: Calculate Greenwich Mean Sidereal Time (GMST) from LMST and longitude.
    // GMST_seconds = (LMST_seconds - longitude_time_seconds) mod seconds_in_day
    // 1 degree of longitude corresponds to 240 seconds of time (86400 seconds / 360 degrees).
    let longitude_in_time_seconds = longitude_deg * 240.0;

    let gmst_seconds_unnormalized = lmst_seconds - longitude_in_time_seconds;
    let gmst_seconds = gmst_seconds_unnormalized.rem_euclid(seconds_in_day);

    // Step 8: Calculate Julian Date UT1 (JD_UT1) from GMST.
    // This is the inverse of the GMST calculation. We use an approximation based on the reference date.

    // Get Julian Date at 0h UT1 of the reference day
    let jd_ut1_ref_day_floor = jd_ut1_ref.floor(); // jd_ut1_ref is from Step 4
    let t_ut1_ref_day_floor = (jd_ut1_ref_day_floor - jd_j2000) / 36525.0; // jd_j2000 defined in Step 5

    // Calculate GMST at 0h UT1 on this reference UT1 day.
    let gmst_at_0h_ut1_ref_day_unnormalized = 24110.54841
        + 8640184.812866 * t_ut1_ref_day_floor
        + 0.093104 * t_ut1_ref_day_floor.powi(2)
        - 6.2e-6 * t_ut1_ref_day_floor.powi(3);
    let gmst_at_0h_ut1_ref_day = gmst_at_0h_ut1_ref_day_unnormalized.rem_euclid(seconds_in_day);

    // Difference in GMST from 0h on the reference day to the target GMST.
    // This is how much the Earth has rotated (in sidereal seconds) into the current day.
    let delta_gmst_seconds_into_day = (gmst_seconds - gmst_at_0h_ut1_ref_day).rem_euclid(seconds_in_day);

    // Convert this sidereal interval to a UT1 interval.
    // Earth rotates 360 deg in one sidereal day (approx 86164.0905 solar seconds).
    // Earth rotates 360 deg relative to mean sun in one solar day (86400 solar seconds).
    // The ratio of mean sidereal day length to mean solar day length is approx. 0.9972695663.
    // Or, rate of GMST change is approx. 1.00273790935 times the rate of UT1 change.
    // So, delta_UT1_seconds = delta_GMST_seconds / 1.0027379093509
    let ut1_interval_seconds = delta_gmst_seconds_into_day / 1.0027379093509;

    // Approximate JD UT1 for the target LST.
    let jd_ut1_calculated = jd_ut1_ref_day_floor + (ut1_interval_seconds / seconds_in_day);

    // TODO: This JD_UT1 calculation is an approximation. For higher precision,
    // an iterative solver (e.g., Newton-Raphson) for T_UT1 from the GMST formula
    // using `jd_ut1_ref` as an initial guess might be necessary if this approximation
    // proves insufficient. The current approach assumes that the GMST polynomial's
    // coefficients don't change significantly enough over the course of ~1 day
    // to invalidate using `gmst_at_0h_ut1_ref_day` as the daily reference.

    // Step 9: Convert JD_UT1_calculated to an Epoch in UT1 scale.
    // `hifitime_ut1_feature_enabled` was determined in Step 4 based on et2lst's findings (false).
    let ut1_epoch_calculated = if hifitime_ut1_feature_enabled {
        Epoch::from_jd_ut1(jd_ut1_calculated)
    } else {
        // Fallback: Constructing from JD UTC as "ut1" feature is not enabled.
        // This means jd_ut1_calculated is effectively treated as a JD UTC.
        // TODO: Emphasize this approximation if "ut1" feature remains disabled.
        Epoch::from_jd_utc(jd_ut1_calculated)
    };

    // Step 10: Convert UT1 Epoch to UTC Epoch.
    // (Uses the same dut1_placeholder_seconds from Step 4)
    let utc_epoch_calculated = ut1_epoch_calculated - dut1_placeholder_seconds.seconds();

    // Step 11: Convert UTC Epoch to ET (TDB) Epoch.
    // hifitime's TDB is equivalent to SPICE ET.
    let final_et_epoch = utc_epoch_calculated.to_tdb();

    // Log the intermediate values if needed for debugging.
    // log::debug!(
    //     "lst2et calculation: lst_s={}, lon_deg={}, body_id={}, ref_et={:?}
"
    //     "  lst_norm_s={}, lon_norm_deg={}
"
    //     "  ref_utc={}, ref_ut1_approx={}, jd_ut1_ref={}
"
    //     "  d_j2000_ref={}, MeanAnomaly_rad_ref={}, eot_s_ref={}
"
    //     "  lmst_s={}, gmst_s={}
"
    //     "  jd_ut1_ref_day_floor={}, t_ut1_ref_day_floor={}, gmst_at_0h_ref={}
"
    //     "  delta_gmst_s_into_day={}, ut1_interval_s={}, jd_ut1_calc={}
"
    //     "  ut1_epoch_calc={:?}, utc_epoch_calc={:?}, final_et_epoch={:?}",
    //     lst_seconds, longitude_deg, body_id, reference_et_for_date_estimation, // Initial inputs
    //     lst_seconds, longitude_deg, // Normalized inputs (assuming longitude_deg was normalized if logic added)
    //     ref_utc_epoch, ref_ut1_epoch, jd_ut1_ref, // From ref ET
    //     d_j2000_ref, mean_anomaly_rad_ref, eot_seconds_ref, // EOT calc
    //     lmst_seconds, gmst_seconds, // LMST/GMST calc
    //     jd_ut1_ref_day_floor, t_ut1_ref_day_floor, gmst_at_0h_ut1_ref_day, // JD from GMST ref calc
    //     delta_gmst_seconds_into_day, ut1_interval_seconds, jd_ut1_calculated, // JD from GMST calc
    //     ut1_epoch_calculated, utc_epoch_calculated, final_et_epoch // Final conversions
    // );

    Ok(final_et_epoch)
}
