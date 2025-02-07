use anise::{
    constants::{
        frames::{EME2000, IAU_EARTH_FRAME},
        usual_planetary_constants::MEAN_EARTH_ANGULAR_VELOCITY_DEG_S,
    },
    prelude::{Almanac, Orbit},
};
use core::str::FromStr;
use hifitime::Epoch;

// Define location of DSN DSS-65 in Madrid, Spain
const DSS65_LATITUDE_DEG: f64 = 40.427_222;
const DSS65_LONGITUDE_DEG: f64 = 4.250_556;
const DSS65_HEIGHT_KM: f64 = 0.834_939;

/// Validation test for azimuth, elevation, range, and range-rate computation.
/// Importantly, we only test for range-rate here; I forget the set up used for DSS65 in GMAT, and could not export the az/el data.
#[test]
fn validate_aer_vs_gmat_cislunar1() {
    let almanac = Almanac::default()
        .load("../data/earth_latest_high_prec.bpc")
        .unwrap()
        .load("../data/pck11.pca")
        .unwrap()
        .load("../data/de430.bsp")
        .unwrap();

    let eme2k = almanac.frame_from_uid(EME2000).unwrap();

    let states = &[
        Orbit::new(
            58643.769540,
            -61696.435624,
            -36178.745722,
            2.148654,
            -1.202489,
            -0.714016,
            Epoch::from_str("2023-11-16T13:35:30.231999909 UTC").unwrap(),
            eme2k,
        ),
        Orbit::new(
            66932.786920,
            -66232.188134,
            -38873.611383,
            2.040555,
            -1.092316,
            -0.649376,
            Epoch::from_str("2023-11-16T14:41:30.231999930 UTC").unwrap(),
            eme2k,
        ),
        Orbit::new(
            74004.678872,
            -69951.400821,
            -41085.748329,
            1.956606,
            -1.011239,
            -0.601766,
            Epoch::from_str("2023-11-16T15:40:30.231999839 UTC").unwrap(),
            eme2k,
        ),
        Orbit::new(
            80796.572756,
            -73405.951304,
            -43142.418173,
            1.882015,
            -0.942232,
            -0.561216,
            Epoch::from_str("2023-11-16T16:39:30.232000062 UTC").unwrap(),
            eme2k,
        ),
        Orbit::new(
            91643.444941,
            -78707.219860,
            -46302.227968,
            1.773135,
            -0.846264,
            -0.504775,
            Epoch::from_str("2023-11-16T18:18:30.231999937 UTC").unwrap(),
            eme2k,
        ),
    ];

    let observations = &[
        (91457.558, 2.199),
        (99965.056, 2.105),
        (107322.912, 2.056),
        (114551.675, 2.031),
        (126573.919, 2.021),
    ];

    for (rx, (range_km, range_rate_km_s)) in
        states.iter().copied().zip(observations.iter().copied())
    {
        // Rebuild the ground stations
        let tx = Orbit::try_latlongalt(
            DSS65_LATITUDE_DEG,
            DSS65_LONGITUDE_DEG,
            DSS65_HEIGHT_KM,
            MEAN_EARTH_ANGULAR_VELOCITY_DEG_S,
            rx.epoch,
            almanac.frame_from_uid(IAU_EARTH_FRAME).unwrap(),
        )
        .unwrap();

        let aer = almanac
            .azimuth_elevation_range_sez(rx, tx, None, None)
            .unwrap();

        dbg!(aer.range_km - range_km);
        assert!(
            (aer.range_rate_km_s - range_rate_km_s).abs() < 1e-3,
            "more than 1 m/s error!"
        );
    }
}

#[test]
fn validate_aer_vs_gmat_cislunar2() {
    let almanac = Almanac::default()
        .load("../data/earth_latest_high_prec.bpc")
        .unwrap()
        .load("../data/pck11.pca")
        .unwrap()
        .load("../data/de430.bsp")
        .unwrap();

    let eme2k = almanac.frame_from_uid(EME2000).unwrap();

    let states = &[
        Orbit::new(
            102114.297454,
            13933.746232,
            -671.117990,
            2.193540,
            0.906982,
            0.333105,
            Epoch::from_str("2022-11-29T14:39:28.000000216 TAI").unwrap(),
            eme2k,
        ),
        Orbit::new(
            110278.148176,
            17379.224108,
            608.602854,
            2.062036,
            0.887598,
            0.333149,
            Epoch::from_str("2022-11-29T15:43:28.000000160 TAI").unwrap(),
            eme2k,
        ),
        Orbit::new(
            117388.586896,
            20490.340765,
            1786.240391,
            1.957486,
            0.870185,
            0.332038,
            Epoch::from_str("2022-11-29T16:42:28.000000384 TAI").unwrap(),
            eme2k,
        ),
        Orbit::new(
            124151.820782,
            23540.835319,
            2958.593254,
            1.865399,
            0.853363,
            0.330212,
            Epoch::from_str("2022-11-29T17:41:28.000000293 TAI").unwrap(),
            eme2k,
        ),
        Orbit::new(
            131247.969145,
            26834.012939,
            4241.579371,
            1.775455,
            0.835578,
            0.327653,
            Epoch::from_str("2022-11-29T18:46:28.000000433 TAI").unwrap(),
            eme2k,
        ),
    ];

    let observations = &[
        (102060.177, 1.957),
        (109389.490, 1.868),
        (115907.202, 1.820),
        (122305.708, 1.799),
        (129320.821, 1.802),
    ];

    for (rx, (range_km, range_rate_km_s)) in
        states.iter().copied().zip(observations.iter().copied())
    {
        // Rebuild the ground stations
        let tx = Orbit::try_latlongalt(
            DSS65_LATITUDE_DEG,
            DSS65_LONGITUDE_DEG,
            DSS65_HEIGHT_KM,
            MEAN_EARTH_ANGULAR_VELOCITY_DEG_S,
            rx.epoch,
            almanac.frame_from_uid(IAU_EARTH_FRAME).unwrap(),
        )
        .unwrap();

        let aer = almanac
            .azimuth_elevation_range_sez(rx, tx, None, None)
            .unwrap();

        dbg!(aer.range_km - range_km);
        assert!(
            (aer.range_rate_km_s - range_rate_km_s).abs() < 1e-3,
            "more than 1 m/s error!"
        );
    }
}
