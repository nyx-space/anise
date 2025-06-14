extern crate pretty_env_logger as pel;

use anise::astro::orbit::ECC_EPSILON;
use anise::constants::frames::{EARTH_J2000, MOON_J2000};
use anise::constants::usual_planetary_constants::MEAN_EARTH_ANGULAR_VELOCITY_DEG_S;
use anise::errors::PhysicsError;
use anise::math::angles::{between_0_360, between_pm_180};
use anise::math::Vector3;
use anise::prelude::*;
use anise::time::{Epoch, TimeSeries, Unit};
use anise::{f64_eq, f64_eq_tol};

const TEST_EPS_DURATION_S: f64 = 1e-2;
const TEST_EPS_RADIUS_KM: f64 = 1e-3;

use rstest::{fixture, rstest};

#[fixture]
fn almanac() -> Almanac {
    Almanac::new("../data/pck08.pca").unwrap()
}

#[fixture]
fn epoch() -> Epoch {
    Epoch::from_gregorian_utc_at_midnight(2000, 1, 1)
}

#[fixture]
fn frame(almanac: Almanac) -> Frame {
    almanac.frame_from_uid(EARTH_J2000).unwrap()
}

#[rstest]
fn val_state_def_circ_inc(almanac: Almanac) {
    // Set the GM value from the GMAT data since we're validating the calculations against GMAT.
    let eme2k = almanac
        .frame_from_uid(EARTH_J2000)
        .unwrap()
        .with_mu_km3_s2(398_600.441_5);

    let epoch = Epoch::from_mjd_tai(21_545.0);
    let cart = Orbit::new(
        -2436.45, -2436.45, 6891.037, 5.088_611, -5.088_611, 0.0, epoch, eme2k,
    );
    let cart2 = Orbit::new(
        -2436.45,
        -2436.45,
        6891.037,
        5.088_611,
        -5.088_611,
        0.0,
        Epoch::from_jde_tai(epoch.to_jde_tai_days()),
        eme2k,
    );
    assert_eq!(
        cart, cart2,
        "different representations of the datetime are not considered equal"
    );
    f64_eq!(cart.radius_km.x, -2436.45, "x");
    f64_eq!(cart.radius_km.y, -2436.45, "y");
    f64_eq!(cart.radius_km.z, 6891.037, "z");
    f64_eq!(cart.velocity_km_s.x, 5.088_611, "vx");
    f64_eq!(cart.velocity_km_s.y, -5.088_611, "vy");
    f64_eq!(cart.velocity_km_s.z, 0.0, "vz");
    f64_eq!(
        cart.energy_km2_s2().unwrap(),
        -25.842_247_282_849_137,
        "energy"
    );

    assert_eq!(
        cart.period().unwrap(),
        6_740.269_063_641 * Unit::Second,
        "period"
    );
    f64_eq!(cart.hx().unwrap(), 35_065.806_679_607_005, "HX");
    f64_eq!(cart.hy().unwrap(), 35_065.806_679_607_005, "HY");
    f64_eq!(cart.hz().unwrap(), 24_796.292_541_9, "HZ");
    f64_eq!(cart.sma_km().unwrap(), 7_712.186_117_895_043, "sma");
    f64_eq!(cart.ecc().unwrap(), 0.000_999_582_831_432_052_5, "ecc");
    f64_eq!(cart.inc_deg().unwrap(), 63.434_003_407_751_14, "inc");
    f64_eq!(cart.raan_deg().unwrap(), 135.0, "raan");
    f64_eq!(cart.aop_deg().unwrap(), 90.0, "aop");
    f64_eq!(cart.ta_deg().unwrap(), 0.0, "ta");
    f64_eq!(cart.tlong_deg().unwrap(), 225.0, "tlong");
    f64_eq!(cart.ea_deg().unwrap(), 0.0, "ea");
    f64_eq!(cart.ma_deg().unwrap(), 0.0, "ma");
    f64_eq!(cart.apoapsis_km().unwrap(), 7_719.895_086_731_299, "apo");
    f64_eq!(cart.periapsis_km().unwrap(), 7_704.477_149_058_786, "peri");
    f64_eq!(
        cart.semi_parameter_km().unwrap(),
        7_712.178_412_142_147,
        "semi parameter"
    );

    let kep = Orbit::keplerian(
        8_191.93, 1e-6, 12.85, 306.614, 314.19, 99.887_7, epoch, eme2k,
    );
    f64_eq!(kep.radius_km.x, 8_057.976_452_202_976, "x");
    f64_eq!(kep.radius_km.y, -0.196_740_370_290_888_9, "y");
    f64_eq!(kep.radius_km.z, 1_475.383_214_274_138, "z");
    f64_eq!(kep.velocity_km_s.x, -0.166_470_488_584_076_31, "vx");
    f64_eq!(kep.velocity_km_s.y, 6.913_868_638_275_646_5, "vy");
    f64_eq!(kep.velocity_km_s.z, 0.910_157_981_443_279_1, "vz");
    f64_eq!(kep.sma_km().unwrap(), 8_191.929_999_999_999, "sma");
    f64_eq!(kep.ecc().unwrap(), 1.000_000_000_388_51e-06, "ecc");
    f64_eq!(kep.inc_deg().unwrap(), 12.849_999_999_999_987, "inc");
    f64_eq!(kep.raan_deg().unwrap(), 306.614, "raan");
    f64_eq!(kep.aop_deg().unwrap(), 314.189_999_994_618_1, "aop");
    f64_eq!(kep.ta_deg().unwrap(), 99.887_700_005_381_9, "ta");
    f64_eq!(
        kep.energy_km2_s2().unwrap(),
        -24.328_848_116_377_95,
        "energy"
    );
    assert_eq!(
        kep.period().unwrap(),
        7_378.877_993_955 * Unit::Second,
        "period"
    );
    f64_eq!(kep.hx().unwrap(), -10_200.784_799_426_574, "HX");
    f64_eq!(kep.hy().unwrap(), -7_579.639_346_783_497, "HY");
    f64_eq!(kep.hz().unwrap(), 55_711.757_929_384_25, "HZ");
    f64_eq!(kep.tlong_deg().unwrap(), 0.691_700_000_000_082_6, "tlong");
    f64_eq!(kep.ea_deg().unwrap(), 99.887_643_560_656_85, "ea");
    f64_eq!(kep.ma_deg().unwrap(), 99.887_587_115_926_96, "ma");
    f64_eq!(kep.apoapsis_km().unwrap(), 8_191.938_191_930_002, "apo");
    f64_eq!(kep.periapsis_km().unwrap(), 8_191.921_808_069_997, "peri");
    f64_eq!(
        kep.semi_parameter_km().unwrap(),
        8_191.929_999_991_808,
        "semi parameter"
    );
    let kep = Orbit::keplerian(
        8_191.93, 0.2, 12.85, 306.614, 314.19, -99.887_7, epoch, eme2k,
    );
    f64_eq!(kep.ta_deg().unwrap(), 260.1123, "ta");

    let ric_delta = kep.ric_difference(&kep).unwrap();
    // Check that the frame is stripped
    assert!(ric_delta.frame.mu_km3_s2.is_none());
    assert!(ric_delta.frame.shape.is_none());
    // Check that the difference in radius magnitude and velocity are both zero
    assert_eq!(ric_delta.rmag_km(), 0.0);
    assert_eq!(ric_delta.vmag_km_s(), 0.0);

    // Check that the RIC computation is reciprocal.
    let dcm = kep.dcm_from_ric_to_inertial().unwrap();
    let kep_ric = (dcm.transpose() * kep).unwrap();
    let kep_rtn = (dcm * kep_ric).unwrap();
    f64_eq!(kep_rtn.rss_radius_km(&kep).unwrap(), 0.0, "RIC RSS radius");
    f64_eq!(
        kep_rtn.rss_velocity_km_s(&kep).unwrap(),
        0.0,
        "RIC RMS velocity"
    );
    f64_eq!(kep_rtn.rms_radius_km(&kep).unwrap(), 0.0, "RIC RSS radius");
    f64_eq!(
        kep_rtn.rms_velocity_km_s(&kep).unwrap(),
        0.0,
        "RIC RMS velocity"
    );

    let vnc_delta = kep.vnc_difference(&kep).unwrap();
    // Check that the frame is stripped
    assert!(vnc_delta.frame.mu_km3_s2.is_none());
    assert!(vnc_delta.frame.shape.is_none());
    // Check that the difference in radius magnitude and velocity are both zero
    assert_eq!(vnc_delta.rmag_km(), 0.0);
    assert_eq!(vnc_delta.vmag_km_s(), 0.0);

    // Check that the VNC computation is reciprocal.
    let dcm = kep.dcm_from_ric_to_inertial().unwrap();
    let kep_ric = (dcm.transpose() * kep).unwrap();
    let kep_rtn = (dcm * kep_ric).unwrap();
    f64_eq!(kep_rtn.rss_radius_km(&kep).unwrap(), 0.0, "VNC RSS radius");
    f64_eq!(
        kep_rtn.rss_velocity_km_s(&kep).unwrap(),
        0.0,
        "VNC RMS velocity"
    );
    f64_eq!(kep_rtn.rms_radius_km(&kep).unwrap(), 0.0, "VNC RSS radius");
    f64_eq!(
        kep_rtn.rms_velocity_km_s(&kep).unwrap(),
        0.0,
        "VNC RMS velocity"
    );

    // Check the relative and absolute differences
    let (abs_pos_km, abs_vel_km_s) = kep_rtn.abs_difference(&kep).unwrap();
    f64_eq!(
        abs_pos_km.abs(),
        0.0,
        "non zero absolute position difference"
    );
    f64_eq!(
        abs_vel_km_s.abs(),
        0.0,
        "non zero absolute velocity difference"
    );

    let (rel_pos, rel_vel) = kep_rtn.rel_difference(&kep).unwrap();
    f64_eq!(rel_pos.abs(), 0.0, "non zero relative position difference");
    f64_eq!(rel_vel.abs(), 0.0, "non zero relative velocity difference");

    // Ensure that setting the radius or velocity to zero causes an error in relative difference
    let mut zero_pos = kep;
    zero_pos.radius_km = Vector3::zeros();
    assert!(
        zero_pos.rel_pos_diff(&kep).is_err(),
        "radius of self must be non zero"
    );
    assert!(
        kep.rel_pos_diff(&zero_pos).is_ok(),
        "only self needs a non zero radius"
    );

    // Ensure that setting the radius or velocity to zero causes an error in relative difference
    let mut zero_vel = kep;
    zero_vel.velocity_km_s = Vector3::zeros();
    assert!(
        zero_vel.rel_vel_diff(&kep).is_err(),
        "velocity of self must be non zero"
    );
    assert!(
        kep.rel_vel_diff(&zero_vel).is_ok(),
        "only self needs a non zero velocity"
    );
}

#[rstest]
fn val_state_def_elliptical(almanac: Almanac) {
    // Set the GM value from the GMAT data since we're validating the calculations against GMAT.
    let eme2k = almanac
        .frame_from_uid(EARTH_J2000)
        .unwrap()
        .with_mu_km3_s2(398_600.441_5);

    let epoch = Epoch::from_mjd_tai(21_545.0);
    let cart = Orbit::new(
        5_946.673_548_288_958,
        1_656.154_606_023_661,
        2_259.012_129_598_249,
        -3.098_683_050_943_824,
        4.579_534_132_135_011,
        6.246_541_551_539_432,
        epoch,
        eme2k,
    );
    f64_eq!(
        cart.energy_km2_s2().unwrap(),
        -25.842_247_282_849_144,
        "energy"
    );
    assert_eq!(
        cart.period().unwrap(),
        6_740.269_063_641 * Unit::Second,
        "period"
    );
    f64_eq!(cart.hx().unwrap(), 0.015_409_898_034_704_383, "HX");
    f64_eq!(cart.hy().unwrap(), -44_146.106_010_690_01, "HY");
    f64_eq!(cart.hz().unwrap(), 32_364.892_694_481_765, "HZ");
    f64_eq!(cart.sma_km().unwrap(), 7_712.186_117_895_041, "sma");
    f64_eq!(cart.ecc().unwrap(), 0.158_999_999_999_999_95, "ecc");
    f64_eq!(cart.inc_deg().unwrap(), 53.753_69, "inc");
    f64_eq!(cart.raan_deg().unwrap(), 1.998_632_864_211_17e-05, "raan");
    f64_eq!(cart.aop_deg().unwrap(), 359.787_880_000_004, "aop");
    f64_eq!(cart.ta_deg().unwrap(), 25.434_003_407_751_188, "ta");
    f64_eq!(cart.tlong_deg().unwrap(), 25.221_903_394_083_824, "tlong");
    f64_eq!(cart.ea_deg().unwrap(), 21.763_052_882_584_79, "ea");
    f64_eq!(cart.ma_deg().unwrap(), 18.385_336_330_516_39, "ma");
    f64_eq!(cart.apoapsis_km().unwrap(), 8_938.423_710_640_353, "apo");
    f64_eq!(cart.periapsis_km().unwrap(), 6_485.948_525_149_73, "peri");
    f64_eq!(
        cart.semi_parameter_km().unwrap(),
        7_517.214_340_648_537,
        "semi parameter"
    );

    let kep = Orbit::keplerian(
        8_191.93, 0.024_5, 12.85, 306.614, 314.19, 99.887_7, epoch, eme2k,
    );
    f64_eq!(kep.radius_km.x, 8_087.161_618_048_522_5, "x");
    f64_eq!(kep.radius_km.y, -0.197_452_943_772_520_73, "y");
    f64_eq!(kep.radius_km.z, 1_480.726_901_246_883, "z");
    f64_eq!(kep.velocity_km_s.x, -0.000_168_592_186_843_952_16, "vx");
    f64_eq!(kep.velocity_km_s.y, 6.886_845_792_370_852, "vy");
    f64_eq!(kep.velocity_km_s.z, 0.936_931_260_302_891_8, "vz");
    f64_eq!(kep.sma_km().unwrap(), 8_191.930_000_000_003, "sma");
    f64_eq!(kep.ecc().unwrap(), 0.024_500_000_000_000_348, "ecc");
    f64_eq!(kep.inc_deg().unwrap(), 12.850_000_000_000_016, "inc");
    f64_eq!(kep.raan_deg().unwrap(), 306.614, "raan");
    f64_eq!(kep.aop_deg().unwrap(), 314.190_000_000_000_4, "aop");
    f64_eq!(kep.ta_deg().unwrap(), 99.887_699_999_999_58, "ta");
    f64_eq!(
        kep.energy_km2_s2().unwrap(),
        -24.328_848_116_377_94,
        "energy"
    );
    assert_eq!(
        kep.period().unwrap(),
        7_378.877_993_955 * Unit::Second,
        "period"
    );
    f64_eq!(kep.hx().unwrap(), -10_197.722_829_337_885, "HX");
    f64_eq!(kep.hy().unwrap(), -7_577.364_166_057_776, "HY");
    f64_eq!(kep.hz().unwrap(), 55_695.034_928_191_49, "HZ");
    f64_eq!(kep.tlong_deg().unwrap(), 0.691_699_999_999_855_2, "tlong");
    f64_eq!(kep.ea_deg().unwrap(), 98.501_748_370_880_22, "ea");
    f64_eq!(kep.ma_deg().unwrap(), 97.113_427_049_323_43, "ma");
    f64_eq!(kep.apoapsis_km().unwrap(), 8_392.632_285_000_007, "apo");
    f64_eq!(kep.periapsis_km().unwrap(), 7_991.227_715_000_001, "peri");
    f64_eq!(
        kep.semi_parameter_km().unwrap(),
        8_187.012_794_017_503,
        "semi parameter"
    );

    let ric_delta = kep.ric_difference(&kep).unwrap();
    // Check that the frame is stripped
    assert!(ric_delta.frame.mu_km3_s2.is_none());
    assert!(ric_delta.frame.shape.is_none());
    // Check that the difference in radius magnitude and velocity are both zero
    assert_eq!(ric_delta.rmag_km(), 0.0);
    assert_eq!(ric_delta.vmag_km_s(), 0.0);

    // Check that the RIC computation is reciprocal.
    let dcm = kep.dcm_from_ric_to_inertial().unwrap();
    let kep_ric = (dcm.transpose() * kep).unwrap();
    let kep_rtn = (dcm * kep_ric).unwrap();
    f64_eq!(kep_rtn.rss_radius_km(&kep).unwrap(), 0.0, "RIC RSS radius");
    f64_eq!(
        kep_rtn.rss_velocity_km_s(&kep).unwrap(),
        0.0,
        "RIC RSS velocity"
    );
}

#[rstest]
fn val_state_def_circ_eq(almanac: Almanac) {
    // Set the GM value from the GMAT data since we're validating the calculations against GMAT.
    let eme2k = almanac
        .frame_from_uid(EARTH_J2000)
        .unwrap()
        .with_mu_km3_s2(398_600.441_5);

    let epoch = Epoch::from_mjd_tai(21_545.0);
    let cart = Orbit::new(
        -38_892.724_449_149_02,
        16_830.384_772_891_86,
        0.722_659_929_135_562_2,
        -1.218_008_333_846_6,
        -2.814_651_172_605_98,
        1.140_294_223_185_661e-5,
        epoch,
        eme2k,
    );
    f64_eq!(
        cart.energy_km2_s2().unwrap(),
        -4.702_902_670_552_006,
        "energy"
    );
    assert_eq!(
        cart.period().unwrap(),
        86_820.776_152_981 * Unit::Second,
        "period"
    );
    f64_eq!(cart.hx().unwrap(), 2.225_951_522_241_969_5, "HX");
    f64_eq!(cart.hy().unwrap(), -0.436_714_326_090_944_6, "HY");
    f64_eq!(cart.hz().unwrap(), 129_969.001_391_865_75, "HZ");
    f64_eq!(cart.sma_km().unwrap(), 42_378.129_999_999_98, "sma");
    f64_eq!(cart.ecc().unwrap(), 9.999_999_809_555_511e-9, "ecc");
    f64_eq!(cart.inc_deg().unwrap(), 0.001_000_000_401_564_538_6, "inc");
    f64_eq!(cart.raan_deg().unwrap(), 78.9, "raan");
    f64_eq!(cart.aop_deg().unwrap(), 65.399_999_847_186_78, "aop");
    f64_eq!(cart.ta_deg().unwrap(), 12.300_000_152_813_197, "ta");
    f64_eq!(cart.tlong_deg().unwrap(), 156.599_999_999_999_97, "tlong");
    f64_eq!(cart.ea_deg().unwrap(), 12.300_000_030_755_777, "ea");
    f64_eq!(cart.ma_deg().unwrap(), 12.299_999_908_698_359, "ma");
    f64_eq!(cart.apoapsis_km().unwrap(), 42_378.130_423_781_27, "apo");
    f64_eq!(cart.periapsis_km().unwrap(), 42_378.129_576_218_69, "peri");
    f64_eq!(
        cart.semi_parameter_km().unwrap(),
        42_378.129_999_999_976,
        "semi parameter"
    );

    let kep = Orbit::keplerian(18191.098, 1e-6, 1e-6, 306.543, 314.32, 98.765, epoch, eme2k);
    f64_eq!(kep.radius_km.x, 18_190.717_357_886_37, "x");
    f64_eq!(kep.radius_km.y, -118.107_162_539_218_69, "y");
    f64_eq!(kep.radius_km.z, 0.000_253_845_647_633_053_35, "z");
    f64_eq!(kep.velocity_km_s.x, 0.030_396_440_130_264_88, "vx");
    f64_eq!(kep.velocity_km_s.y, 4.680_909_107_924_576, "vy");
    f64_eq!(kep.velocity_km_s.z, 4.907_089_816_726_583e-8, "vz");
    f64_eq!(kep.sma_km().unwrap(), 18_191.098_000_000_013, "sma");
    f64_eq!(kep.ecc().unwrap(), 9.999_999_997_416_087e-7, "ecc");
    f64_eq!(kep.inc_deg().unwrap(), 1.207_418_269_725_733_3e-6, "inc");
    f64_eq!(kep.raan_deg().unwrap(), 306.543, "raan");
    f64_eq!(kep.aop_deg().unwrap(), 314.320_000_025_403_66, "aop");
    f64_eq!(kep.ta_deg().unwrap(), 98.764_999_974_596_28, "ta");
    f64_eq!(
        kep.energy_km2_s2().unwrap(),
        -10.955_920_349_063_035,
        "energy"
    );
    assert_eq!(
        kep.period().unwrap(),
        24_417.396_242_566 * Unit::Second,
        "period"
    );
    f64_eq!(kep.hx().unwrap(), -0.001_194_024_028_558_358_7, "HX");
    f64_eq!(kep.hy().unwrap(), -0.000_884_918_835_027_750_6, "HY");
    f64_eq!(kep.hz().unwrap(), 85_152.684_597_507_06, "HZ");
    f64_eq!(kep.tlong_deg().unwrap(), 359.627_999_999_999_93, "tlong");
    f64_eq!(kep.ea_deg().unwrap(), 98.764_943_347_932_57, "ea");
    f64_eq!(kep.ma_deg().unwrap(), 98.764_886_721_264_56, "ma");
    f64_eq!(kep.apoapsis_km().unwrap(), 18_191.116_191_098_008, "apo");
    f64_eq!(kep.periapsis_km().unwrap(), 18_191.079_808_902_017, "peri");
    f64_eq!(
        kep.semi_parameter_km().unwrap(),
        18_191.097_999_981_823,
        "semi parameter"
    );

    let ric_delta = kep.ric_difference(&kep).unwrap();
    // Check that the frame is stripped
    assert!(ric_delta.frame.mu_km3_s2.is_none());
    assert!(ric_delta.frame.shape.is_none());
    // Check that the difference in radius magnitude and velocity are both zero
    assert_eq!(ric_delta.rmag_km(), 0.0);
    assert_eq!(ric_delta.vmag_km_s(), 0.0);

    // Check that the RIC computation is reciprocal.
    let dcm = kep.dcm_from_ric_to_inertial().unwrap();
    let kep_ric = (dcm.transpose() * kep).unwrap();
    let kep_rtn = (dcm * kep_ric).unwrap();
    f64_eq!(kep_rtn.rss_radius_km(&kep).unwrap(), 0.0, "RIC RSS radius");
    f64_eq!(
        kep_rtn.rss_velocity_km_s(&kep).unwrap(),
        0.0,
        "RIC RSS velocity"
    );
}

#[rstest]
fn val_state_def_equatorial(almanac: Almanac) {
    // Set the GM value from the GMAT data since we're validating the calculations against GMAT.
    let eme2k = almanac
        .frame_from_uid(EARTH_J2000)
        .unwrap()
        .with_mu_km3_s2(398_600.441_5);

    let epoch = Epoch::from_mjd_tai(21_545.0);
    let cart = Orbit::new(
        -7273.338970882,
        253.990592670,
        0.022164861,
        -0.258285289,
        -7.396322922,
        -0.000645451,
        epoch,
        eme2k,
    );

    f64_eq!(cart.sma_km().unwrap(), 7278.136188379306, "sma");
    f64_eq!(cart.ecc().unwrap(), 4.99846643158263e-05, "ecc");
    f64_eq!(cart.inc_deg().unwrap(), 0.005000000478594339, "inc");
    f64_eq!(cart.raan_deg().unwrap(), 360.0, "raan");
    f64_eq!(cart.aop_deg().unwrap(), 177.9999736473912, "aop");
    f64_eq!(cart.ta_deg().unwrap(), 2.650_826_247_094_554e-5, "ta");

    let ric_delta = cart.ric_difference(&cart).unwrap();
    // Check that the frame is stripped
    assert!(ric_delta.frame.mu_km3_s2.is_none());
    assert!(ric_delta.frame.shape.is_none());
    // Check that the difference in radius magnitude and velocity are both zero
    assert_eq!(ric_delta.rmag_km(), 0.0);
    assert_eq!(ric_delta.vmag_km_s(), 0.0);

    // Check that the RIC computation is reciprocal.
    let dcm = cart.dcm_from_ric_to_inertial().unwrap();
    let cart_ric = (dcm.transpose() * cart).unwrap();
    let cart_rtn = (dcm * cart_ric).unwrap();
    f64_eq!(
        cart_rtn.rss_radius_km(&cart).unwrap(),
        0.0,
        "RIC RSS radius"
    );
    f64_eq!(
        cart_rtn.rss_velocity_km_s(&cart).unwrap(),
        0.0,
        "RIC RSS velocity"
    );
}

#[rstest]
fn val_state_def_reciprocity(almanac: Almanac) {
    // Set the GM value from the GMAT data since we're validating the calculations against GMAT.
    let eme2k = almanac
        .frame_from_uid(EARTH_J2000)
        .unwrap()
        .with_mu_km3_s2(398_600.441_5);

    let epoch = Epoch::from_mjd_tai(21_545.0);

    assert_eq!(
        Orbit::new(
            -38_892.724_449_149_02,
            16_830.384_772_891_86,
            0.722_659_929_135_562_2,
            -1.218_008_333_846_6,
            -2.814_651_172_605_98,
            1.140_294_223_185_661e-5,
            epoch,
            eme2k
        ),
        Orbit::keplerian(
            42_378.129_999_999_98,
            9.999_999_809_555_511e-9,
            0.001_000_000_401_564_538_6,
            78.9,
            65.399_999_847_186_78,
            12.300_000_152_813_197,
            epoch,
            eme2k
        ),
        "circ_eq"
    );

    assert_eq!(
        Orbit::new(
            5_946.673_548_288_958,
            1_656.154_606_023_661,
            2_259.012_129_598_249,
            -3.098_683_050_943_824,
            4.579_534_132_135_011,
            6.246_541_551_539_432,
            epoch,
            eme2k
        ),
        Orbit::keplerian(
            7_712.186_117_895_041,
            0.158_999_999_999_999_95,
            53.75369,
            1.998_632_864_211_17e-5,
            359.787_880_000_004,
            25.434_003_407_751_188,
            epoch,
            eme2k
        ),
        "elliptical"
    );

    assert_eq!(
        Orbit::new(-2436.45, -2436.45, 6891.037, 5.088_611, -5.088_611, 0.0, epoch, eme2k),
        Orbit::keplerian(
            7_712.186_117_895_043,
            0.000_999_582_831_432_052_5,
            63.434_003_407_751_14,
            135.0,
            90.0,
            0.0,
            epoch,
            eme2k
        ),
        "circ_inc"
    );
}

#[rstest]
fn verif_geodetic_vallado(almanac: Almanac) {
    let eme2k = almanac.frame_from_uid(EARTH_J2000).unwrap();

    let epoch = Epoch::from_mjd_tai(51_545.0);
    // Test case from Vallado, 4th Ed., page 173, Example 3-3
    let ri = 6524.834;
    let ri_val = 6_524.833_999_999_999;
    let rj = 6862.875;
    let rj_val = 6_862.874_999_999_999;
    let rk = 6448.296;
    let lat = 34.352_519_916_935_62; // Valldo: 34.352496
    let long = 46.446_416_856_789_96; // Vallado 46.4464
    let height = 5_085.217_419_357_936; // Vallado: 5085.22
    let r = Orbit::from_position(ri, rj, rk, epoch, eme2k);
    f64_eq!(r.latitude_deg().unwrap(), lat, "latitude (φ)");
    f64_eq!(r.longitude_deg(), long, "longitude (λ)");
    f64_eq!(r.height_km().unwrap(), height, "height");
    // Check that we can compute orbital parameters here, although these will be odd since it's a ground station
    f64_eq!(
        r.sma_altitude_km().unwrap(),
        -649.854_189_724_88,
        "SMA altitude"
    );
    f64_eq!(
        r.apoapsis_altitude_km().unwrap(),
        5_078.431_620_550_23,
        "Apoapsis altitude"
    );
    f64_eq!(
        r.periapsis_altitude_km().unwrap(),
        -6378.14,
        "Periapsis altitude"
    );

    let r = Orbit::try_latlongalt(
        lat,
        long,
        height,
        MEAN_EARTH_ANGULAR_VELOCITY_DEG_S,
        epoch,
        eme2k,
    )
    .unwrap();
    f64_eq!(r.radius_km.x, ri_val, "r_i");
    f64_eq!(r.radius_km.y, rj_val, "r_j");
    f64_eq!(r.radius_km.z, rk, "r_k");

    // Test case from Vallado, 4th Ed., page 173, Example 3-4
    let lat = -7.906_635_7;
    let lat_val = -7.906_635_699_999_994_5;
    let long = 345.5975;
    let height = 56.0e-3;
    let height_val = 0.056_000_000_000_494_765;
    let ri = 6_119.403_233_271_109;
    let rj = -1_571.480_316_600_378_3;
    let rk = -871.560_226_712_024_7;
    let r = Orbit::try_latlongalt(
        lat,
        long,
        height,
        MEAN_EARTH_ANGULAR_VELOCITY_DEG_S,
        epoch,
        eme2k,
    )
    .unwrap();
    f64_eq!(r.radius_km.x, ri, "r_i");
    f64_eq!(r.radius_km.y, rj, "r_j");
    f64_eq!(r.radius_km.z, rk, "r_k");
    let r = Orbit::from_position(ri, rj, rk, epoch, eme2k);
    f64_eq!(r.latitude_deg().unwrap(), lat_val, "latitude (φ)");
    f64_eq!(r.longitude_deg(), long - 360.0, "longitude (λ)");
    f64_eq!(r.longitude_360_deg(), long, "longitude (λ)");
    f64_eq!(r.height_km().unwrap(), height_val, "height");

    // Check reciprocity near poles
    let r = Orbit::try_latlongalt(
        0.1,
        long,
        height_val,
        MEAN_EARTH_ANGULAR_VELOCITY_DEG_S,
        epoch,
        eme2k,
    )
    .unwrap();
    f64_eq!(r.latitude_deg().unwrap(), 0.1, "latitude (φ)");
}

#[rstest]
fn verif_with_init(almanac: Almanac) {
    let eme2k = almanac.frame_from_uid(EARTH_J2000).unwrap();

    let epoch = Epoch::from_gregorian_tai_at_midnight(2021, 3, 4);
    let kep = Orbit::keplerian(
        8_191.93, 0.024_5, 12.85, 306.614, 314.19, 99.887_7, epoch, eme2k,
    );
    for sma_incr in 100..1000 {
        let new_sma = kep.sma_km().unwrap() + f64::from(sma_incr);
        f64_eq!(
            kep.with_sma_km(new_sma).expect("with_*").sma_km().unwrap(),
            new_sma,
            "wrong sma"
        );
    }
    for ecc_incr in 0..100 {
        let new_ecc = kep.ecc().unwrap() + f64::from(ecc_incr) / 100.0;
        let new_state = kep.with_ecc(new_ecc).expect("with_*");
        f64_eq!(
            new_state.ecc().unwrap(),
            new_ecc,
            format!(
                "wrong ecc: got {}\twanted {}",
                new_state.inc_deg().unwrap(),
                new_ecc
            )
        );
    }
    for angle_incr in 0..360 {
        let new_aop = between_0_360(kep.aop_deg().unwrap() + f64::from(angle_incr));
        let new_state = kep.add_aop_deg(f64::from(angle_incr)).unwrap();
        f64_eq!(
            new_state.aop_deg().unwrap(),
            new_aop,
            format!(
                "wrong aop: got {}\twanted {}",
                new_state.aop_deg().unwrap(),
                new_aop
            )
        );
    }
    for angle_incr in 0..360 {
        let new_raan = between_0_360(kep.raan_deg().unwrap() + f64::from(angle_incr));
        let new_state = kep.add_raan_deg(f64::from(angle_incr)).unwrap();
        f64_eq!(
            new_state.raan_deg().unwrap(),
            new_raan,
            format!(
                "wrong raan: got {}\twanted {}",
                new_state.raan_deg().unwrap(),
                new_raan
            )
        );
    }
    for angle_incr in 0..360 {
        let new_ta = between_0_360(kep.ta_deg().unwrap() + f64::from(angle_incr));
        let new_state = kep.with_ta_deg(new_ta).unwrap();
        f64_eq!(
            new_state.ta_deg().unwrap(),
            new_ta,
            format!(
                "wrong ta: got {}\twanted {}",
                new_state.aop_deg().unwrap(),
                new_ta
            )
        );
    }
    for angle_incr in 0..360 {
        // NOTE: Inclination is bounded between 0 and 180, hence the slightly different logic here.
        let new_inc = between_pm_180(kep.inc_deg().unwrap() + f64::from(angle_incr)).abs();
        let new_state = kep.add_inc_deg(f64::from(angle_incr)).unwrap();
        f64_eq!(
            new_state.inc_deg().unwrap(),
            new_inc,
            format!(
                "wrong inc: got {}\twanted {}",
                new_state.inc_deg().unwrap(),
                new_inc
            )
        );
    }
    for apsis_delta in 100..1000 {
        let new_ra = kep.apoapsis_km().unwrap() + f64::from(apsis_delta);
        let new_rp = kep.periapsis_km().unwrap() - f64::from(apsis_delta);
        let new_orbit = kep.with_apoapsis_periapsis_km(new_ra, new_rp).unwrap();
        f64_eq!(new_orbit.apoapsis_km().unwrap(), new_ra, "wrong ra");
        f64_eq!(new_orbit.periapsis_km().unwrap(), new_rp, "wrong rp");
    }
}

#[rstest]
fn verif_orbit_at_epoch(almanac: Almanac) {
    let eme2k = almanac.frame_from_uid(EARTH_J2000).unwrap();
    let epoch = Epoch::from_gregorian_utc_at_midnight(2024, 1, 10);
    let circ_incl = Orbit::keplerian(
        8_191.93, 1e-8, 12.85, 306.614, 314.19, 99.887_7, epoch, eme2k,
    );

    let elliptical = Orbit::keplerian(
        8_191.93, 0.2, 12.85, 306.614, 314.19, -99.887_7, epoch, eme2k,
    );

    let circ_equa = Orbit::keplerian(
        8_191.93, 1e-8, 1e-5, 306.614, 314.19, -99.887_7, epoch, eme2k,
    );

    for (ono, orbit) in [circ_incl, elliptical, circ_equa].iter().enumerate() {
        let future_orbit = orbit
            .at_epoch(epoch + 0.5 * orbit.period().unwrap())
            .unwrap();
        // Check that only the true anomaly has changed
        f64_eq!(
            orbit.sma_km().unwrap(),
            future_orbit.sma_km().unwrap(),
            format!("#{ono}: SMA changed")
        );
        f64_eq_tol!(
            orbit.ecc().unwrap(),
            future_orbit.ecc().unwrap(),
            1e-7,
            format!("#{ono}: ECC changed ")
        );
        f64_eq_tol!(
            orbit.inc_deg().unwrap(),
            future_orbit.inc_deg().unwrap(),
            1e-7,
            format!("#{ono}: INC changed")
        );
        f64_eq!(
            orbit.raan_deg().unwrap(),
            future_orbit.raan_deg().unwrap(),
            format!("#{ono}: RAAN changed")
        );
        f64_eq_tol!(
            orbit.aop_deg().unwrap(),
            future_orbit.aop_deg().unwrap(),
            2e-6,
            format!("#{ono}: AOP changed")
        );
    }
}

#[rstest]
fn b_plane_davis(almanac: Almanac) {
    // This is a simple test from Dr. Davis' IMD class at CU Boulder.
    // Set the GM value from the GMAT data since we're validating the calculations against GMAT.
    let eme2k = almanac
        .frame_from_uid(EARTH_J2000)
        .unwrap()
        .with_mu_km3_s2(398_600.441_5);

    // Hyperbolic orbit
    let orbit = Orbit::new(
        546507.344255845,
        -527978.380486028,
        531109.066836708,
        -4.9220589268733,
        5.36316523097915,
        -5.22166308425181,
        Epoch::from_gregorian_utc_at_midnight(2016, 1, 1),
        eme2k,
    );

    // Check reciprocity between the gravity assist functions.
    let phi = orbit.vinf_turn_angle_deg(300.0).unwrap();
    let rp = orbit.vinf_periapsis_km(phi).unwrap();

    assert!(
        (300.0 - rp).abs() < 1e-10,
        "turn angle to rp reciprocity failed"
    );

    // The following is a regression test.
    assert!(dbg!(orbit.hyperbolic_anomaly_deg().unwrap() - 149.610128737).abs() < 1e-9);
}

#[rstest]
fn gh_regression_340(almanac: Almanac) {
    let moon_j2k = almanac.frame_from_uid(MOON_J2000).unwrap();

    let start = Epoch::from_str("2024-10-16").unwrap();

    let orbit = Orbit::keplerian(
        6142.400, // sma
        0.6, 57.7, 270.0, 270.0, 0.0, start, moon_j2k,
    );

    for epoch in TimeSeries::inclusive(
        start,
        Epoch::from_str("2024-10-17").unwrap(),
        Unit::Minute * 1,
    ) {
        assert!(orbit.at_epoch(epoch).is_ok(), "error on {epoch}");
    }
}

#[rstest]
fn misc_verif(almanac: Almanac) {
    let eme2k = almanac.frame_from_uid(EARTH_J2000).unwrap();

    // Blue Ghost landing day!
    let epoch = Epoch::from_gregorian_utc_at_noon(2025, 3, 2);

    // LTAN
    let prograde = Orbit::new(0.0, 7000.0, 0.0, -1.0, 0.0, 7.5, epoch, eme2k);
    f64_eq!(prograde.ltan_deg().unwrap(), 90.0, "prograde LTAN");

    let retrograde = Orbit::new(0.0, -7000.0, 0.0, -1.0, 0.0, 7.5, epoch, eme2k);
    f64_eq!(retrograde.ltan_deg().unwrap(), 270.0, "retrograde LTAN");

    let equatorial = Orbit::keplerian(7000.0, 1e-4, 1e-12, 270.0, 45.0, 76.0, epoch, eme2k);
    assert!(equatorial.ltan_deg().is_err())
}

fn create_orbit(sma_km: f64, ecc: f64, ta_deg: f64, epoch: Epoch, frame: Frame) -> Orbit {
    Orbit::keplerian(sma_km, ecc, 0.0, 0.0, 0.0, ta_deg, epoch, frame)
}

#[rstest]
fn test_duration_to_radius_elliptical(epoch: Epoch, frame: Frame) {
    let sma_km: f64 = 10000.0;
    let ecc = 0.1;

    let orbit_at_apo = create_orbit(sma_km, ecc, 180.0, epoch, frame);

    let rp_km = orbit_at_apo.periapsis_km().unwrap();
    let period_s = orbit_at_apo.period().unwrap().to_seconds();

    // Test 1a: Time from periapsis (TA=0) to apoapsis
    let orbit_at_peri = create_orbit(sma_km, ecc, 0.0, epoch, frame);
    let res1a = orbit_at_peri.duration_to_radius(orbit_at_peri.apoapsis_km().unwrap());
    assert!(res1a.is_ok(), "Test 1a failed: {:?}", res1a.err());
    f64_eq_tol!(
        res1a.unwrap().to_seconds(),
        period_s / 2.0,
        TEST_EPS_DURATION_S,
        "1a"
    );

    // Test 1b: Orbit at TA=90 deg. Time to apoapsis.
    let orbit_at_ta90 = create_orbit(sma_km, ecc, 90.0, epoch, frame);
    // current r for TA=90: p / (1 + e*cos(90)) = sma(1-e^2) = 10000(1-0.01) = 9900km
    let _current_r_ta90 = orbit_at_ta90.rmag_km(); // approx 9900 km
    let res1b = orbit_at_ta90.duration_to_radius(orbit_at_ta90.apoapsis_km().unwrap());
    assert!(res1b.is_ok(), "Test 1b failed: {:?}", res1b.err());
    f64_eq_tol!(
        res1b.unwrap().to_seconds(),
        2804.257,
        TEST_EPS_DURATION_S,
        "1b"
    );

    // Test 1c: Orbit at TA=0. Target radius_km = rp. Expected duration: 0.0.
    let res1c = orbit_at_peri.duration_to_radius(rp_km);
    assert!(res1c.is_ok(), "Test 1c failed: {:?}", res1c.err());
    f64_eq_tol!(res1c.unwrap().to_seconds(), 0.0, TEST_EPS_DURATION_S, "1c");

    // Test 1d: Orbit at TA=0. Target radius_km = 9000.0 + TEST_EPS_RADIUS (slightly after periapsis).
    // Expected: small positive duration.
    let target_r_slightly_after_peri = rp_km + TEST_EPS_RADIUS_KM;
    let res1d = orbit_at_peri.duration_to_radius(target_r_slightly_after_peri);
    assert!(res1d.is_ok(), "Test 1d failed: {:?}", res1d.err());
    f64_eq_tol!(res1d.unwrap().to_seconds(), 2.01, TEST_EPS_DURATION_S, "1d");

    // Test 1e: Orbit at TA=180 deg (apoapsis). Time to periapsis.
    let res1e = orbit_at_apo.duration_to_radius(rp_km);
    assert!(res1e.is_ok(), "Test 1e failed: {:?}", res1e.err());
    f64_eq_tol!(
        res1e.unwrap().to_seconds(),
        period_s / 2.0,
        TEST_EPS_DURATION_S,
        "1e"
    );
}

#[rstest]
fn test_duration_to_radius_circular(epoch: Epoch, frame: Frame) {
    let sma_km: f64 = 10000.0;

    // Test 2a: Near-circular orbit. Target radius_km = sma_km. Expected: Duration 0.0.
    let ecc_near_circular = ECC_EPSILON / 10.0; // Ensure it's treated as circular by the function's logic
    let circ_orbit = create_orbit(sma_km, ecc_near_circular, 0.0, epoch, frame);
    let res2a = circ_orbit.duration_to_radius(sma_km);
    assert!(res2a.is_ok(), "Test 2a failed: {:?}", res2a.err());
    f64_eq_tol!(res2a.unwrap().to_seconds(), 0.0, TEST_EPS_DURATION_S, "");

    // Test 2b: Target radius_km different from sma_km. Expected: RadiusError.
    let res2b = circ_orbit.duration_to_radius(circ_orbit.apoapsis_km().unwrap() + 1.0);
    assert!(res2b.is_err());
    matches!(res2b.err().unwrap(), PhysicsError::RadiusError { .. });
}

#[rstest]
fn test_duration_to_radius_hyperbolic(epoch: Epoch, frame: Frame) {
    let sma_km: f64 = -5000.0; // Negative for hyperbolic
    let ecc: f64 = 2.0;
    let rp_km = sma_km.abs() * (ecc - 1.0); // 5000 * (2-1) = 5000 km

    // Test 3a: From periapsis (TA=0) to radius_km = 10000 km.
    let orbit_at_peri = create_orbit(sma_km, ecc, 0.0, epoch, frame);
    let res3a = orbit_at_peri.duration_to_radius(10000.0);
    assert!(res3a.is_ok(), "Test 3a failed: {:?}", res3a.err());
    f64_eq_tol!(
        res3a.unwrap().to_seconds(),
        713.237,
        TEST_EPS_DURATION_S,
        "test 3a"
    );

    // Test 3b: Target radius_km = rp_km. Expected duration: 0.0.
    let res3b = orbit_at_peri.duration_to_radius(rp_km);
    assert!(res3b.is_ok(), "Test 3b failed: {:?}", res3b.err());
    f64_eq_tol!(res3b.unwrap().to_seconds(), 0.0, TEST_EPS_DURATION_S, "");

    // Test 3c: Orbit at TA=30 deg. Target radius rp_km. Expected: RadiusError (past).
    let orbit_at_ta30 = create_orbit(sma_km, ecc, 30.0, epoch, frame);
    let res3c = orbit_at_ta30.duration_to_radius(rp_km);
    assert!(res3c.is_err());
    matches!(res3c.err().unwrap(), PhysicsError::RadiusError { .. });
}

#[rstest]
fn test_duration_to_radius_error_conditions(epoch: Epoch, frame: Frame) {
    let sma_elliptical: f64 = 10000.0;
    let ecc_elliptical = 0.1;
    let rp_elliptical = sma_elliptical * (1.0 - ecc_elliptical); // 9000
    let ra_elliptical = sma_elliptical * (1.0 + ecc_elliptical); // 11000
    let elliptical_orbit = create_orbit(sma_elliptical, ecc_elliptical, 0.0, epoch, frame);

    let sma_hyperbolic: f64 = -5000.0;
    let ecc_hyperbolic = 2.0;
    let rp_hyperbolic = sma_hyperbolic.abs() * (ecc_hyperbolic - 1.0); // 5000
    let hyperbolic_orbit = create_orbit(sma_hyperbolic, ecc_hyperbolic, 0.0, epoch, frame);

    // Test 4a: radius_km = -100.0
    let res4a = elliptical_orbit.duration_to_radius(-100.0);
    assert!(res4a.is_err());
    matches!(res4a.err().unwrap(), PhysicsError::RadiusError { .. });

    // Test 4b: Elliptical orbit. Target radius_km less than periapsis.
    let res4b = elliptical_orbit.duration_to_radius(rp_elliptical - 100.0);
    assert!(res4b.is_err());
    matches!(res4b.err().unwrap(), PhysicsError::RadiusError { .. });

    // Test 4c: Elliptical orbit. Target radius_km greater than apoapsis.
    let res4c = elliptical_orbit.duration_to_radius(ra_elliptical + 100.0);
    assert!(res4c.is_err());
    matches!(res4c.err().unwrap(), PhysicsError::RadiusError { .. });

    // Test 4d: Hyperbolic orbit. Target radius_km less than periapsis.
    let res4d = hyperbolic_orbit.duration_to_radius(rp_hyperbolic - 100.0);
    assert!(res4d.is_err());
    matches!(res4d.err().unwrap(), PhysicsError::RadiusError { .. });

    // Test 4e: cos(nu) out of bounds. This is tricky to hit precisely due to prior reachability checks.
    // If ecc=0.1, p = 10000*(1-0.01)=9900. For r=p, cos(nu)=0. For r > p, cos(nu) < 0. For r < p, cos(nu) > 0.
    // To make (p/r - 1)/e out of bounds, e.g. > 1: (p/r - 1) > e => p/r > 1+e => p > r(1+e).
    // If r = p/(1+e+epsilon_small), then (p/r-1)/e = (1+e+eps-1)/e = (e+eps)/e = 1+eps/e. This is > 1.
    // This radius is rp = p/(1+e). So r = rp / (1+eps_small_prime). This is smaller than periapsis.
    // So, this condition is typically caught by reachability checks (4b, 4d) for valid orbits.
    // For example, if target radius is less than periapsis, it's caught by "Radius outside reachable range".
    // If we somehow bypassed that, e.g. by a faulty p or e value not from self, this could be hit.
    // For now, rely on 4b, 4c, 4d to cover most practical cases of this.

    // Test 4f: Nearly parabolic orbits (ecc close to 1.0)
    // Case 1: Elliptical, ecc = 1.0 - 2*ECC_EPSILON
    let ecc_near_para_ell = 1.0 - 2.0 * ECC_EPSILON; // Still elliptical

    // sma = rp / (1-ecc) = 7000 / (2*ECC_EPSILON) -> very large sma
    // Using try_keplerian_apsis_radii to avoid issues with large SMA direct input if not careful
    let rp_near_para = 7000.0;
    // For elliptical, ra = rp * (1+e)/(1-e)
    let ra_near_para_ell = rp_near_para * (1.0 + ecc_near_para_ell) / (1.0 - ecc_near_para_ell);
    let orbit_near_para_ell = Orbit::try_keplerian_apsis_radii(
        ra_near_para_ell,
        rp_near_para,
        0.0,
        0.0,
        0.0,
        0.0,
        epoch,
        frame,
    )
    .expect("Near-para ell orbit");

    let res4f1 = orbit_near_para_ell.duration_to_radius(rp_near_para + 1000.0); // Target radius valid
    assert!(
        res4f1.is_ok(),
        "Test 4f1 (near-para ell) failed: {:?}",
        res4f1.err()
    );
    assert!(res4f1.unwrap().to_seconds() > 0.0);

    // Case 2: Hyperbolic, ecc = 1.0 + 2*ECC_EPSILON
    let ecc_near_para_hyp = 1.0 + 2.0 * ECC_EPSILON; // Still hyperbolic
                                                     // sma = rp / (1-ecc) = 7000 / (-2*ECC_EPSILON) -> large negative sma
                                                     // Using try_keplerian_apsis_radii is for elliptical, need direct try_keplerian for hyperbolic SMA
    let sma_near_para_hyp = rp_near_para / (1.0 - ecc_near_para_hyp); // This will be negative
    let orbit_near_para_hyp = create_orbit(sma_near_para_hyp, ecc_near_para_hyp, 0.0, epoch, frame);

    let res4f2 = orbit_near_para_hyp.duration_to_radius(rp_near_para + 1000.0); // Target radius valid
    assert!(
        res4f2.is_ok(),
        "Test 4f2 (near-para hyp) failed: {:?}",
        res4f2.err()
    );
    assert!(res4f2.unwrap().to_seconds() > 0.0);

    // Test 4f-alternative for InfiniteValue (parabolic orbit sma->inf, n->0)
    // Orbit::try_keplerian ensures (1.0 - ecc).abs() >= ECC_EPSILON, so ecc cannot be exactly 1.0.
    // If sma_km is extremely large (e.g. ecc very close to 1.0), n_rad_s becomes very small.
    // If sma_km were infinite (parabolic), n_rad_s would be 0.
    // The current check is ensure!(n_rad_s.is_finite() && n_rad_s > 0.0, InfiniteValue { ... })
    // This should catch n_rad_s = 0 if sma_km was Inf, but sma_km() itself would likely return Inf.
    // If sma_km.abs().powi(3) overflows to Inf, then n_rad_s is 0.
    let ecc_very_near_1 = 1.0 - ECC_EPSILON * 1.01; // Should pass try_keplerian
    let sma_very_large = rp_near_para / (1.0 - ecc_very_near_1);
    let orbit_large_sma = create_orbit(sma_very_large, ecc_very_near_1, 0.0, epoch, frame);
    let res_large_sma_time = orbit_large_sma.duration_to_radius(rp_near_para + 1.0);
    assert!(res_large_sma_time.is_ok()); // Should be ok, time will just be large or n very small.

    // The InfiniteValue for n_rad_s is more about mu/sma^3 being NaN/Inf or zero due to sma being zero or Inf.
    // If sma_km() returns Inf, then sma_km.abs().powi(3) is Inf, then n_rad_s is 0. Caught.
    // If sma_km() returns some error that leads to NaN sma, then n_rad_s is NaN. Caught.
}
