use anise::analysis::prelude::{
    Condition, DcmExpr, Event, FrameSpec, ScalarExpr, StateSpec, VectorExpr,
};
use anise::astro::Location;
use anise::constants::celestial_objects::MOON;
use anise::constants::frames::{IAU_MOON_FRAME, MOON_J2000};
use anise::constants::orientations::{IAU_MOON, J2000};
use anise::math::rotation::EulerParameter;
use anise::math::Vector3;
use anise::prelude::{Almanac, Frame, Orbit};
use anise::prelude::{FovShape, Instrument};
use anise::structure::{InstrumentDataSet, LocationDataSet};
use core::f64::consts::FRAC_PI_2;
use hifitime::Unit;
use rstest::*;

#[fixture]
fn almanac() -> Almanac {
    use std::path::PathBuf;

    // Define the rotation from the RIC Frame (Body) to the Camera Frame.
    // We want Camera +Z (Boresight) to point along RIC -X (Nadir).
    // We want Camera +X (Width) to point along RIC +Z (Cross-track/East?).
    // Rotation: -90 degrees about Y axis.
    // Arbitrarily specify that the body frame is ID -30100 and the instrument is -30101
    let q_to_i = EulerParameter::about_y(-core::f64::consts::FRAC_PI_2, -30100, -30101);

    let lro_camera = Instrument {
        // The camera is rigidly mounted to the spacecraft body (which we assume aligns with RIC)
        q_to_i,

        // Assume camera is at the center of mass for the test
        offset_i: Vector3::zeros(),

        // A reasonable "Wide Angle" Nav Cam FOV
        fov: FovShape::Rectangular {
            x_half_angle_deg: 15.0, // Total width 30 deg
            y_half_angle_deg: 10.0, // Total height 20 deg
        },
    };

    let mut instrument_kernel = InstrumentDataSet::default();
    instrument_kernel
        .push(lro_camera, Some(1), Some("LRO Camera"))
        .unwrap();

    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string()));

    Almanac::new(
        &manifest_dir
            .clone()
            .join("../data/de440s.bsp")
            .to_string_lossy(),
    )
    .unwrap()
    .load(
        &manifest_dir
            .clone()
            .join("../data/pck08.pca")
            .to_string_lossy(),
    )
    .unwrap()
    .load(
        &manifest_dir
            .clone()
            .join("../data/lro.bsp")
            .to_string_lossy(),
    )
    .unwrap()
    .with_instrument_data(instrument_kernel)
}

#[rstest]
fn lro_camera_fov_from_instrument(almanac: Almanac) {
    let instrument = almanac.instrument_from_id(1).unwrap();
    assert_eq!(
        instrument,
        almanac.instrument_from_name("LRO Camera").unwrap()
    );

    let lro_frame = Frame::new(-85, J2000);
    let start = almanac.spk_domain(-85).unwrap().0;
    let epoch = start + Unit::Day * 15;
    // Fetch the state of the vehicle in the Moon IAU frame
    let lro_state = almanac
        .transform(lro_frame, IAU_MOON_FRAME, epoch, None)
        .unwrap();

    let mut dcm_ric_to_inertial = lro_state.dcm3x3_from_ric_to_inertial().unwrap();
    // For the sake of the test, we are setting the RIC frame as the body frame, which the camera frame references.
    dcm_ric_to_inertial.from = -30100;

    let sc_attitude_to_body = EulerParameter::from(dcm_ric_to_inertial).conjugate();

    let moon_center = almanac.state_of(MOON, IAU_MOON_FRAME, epoch, None).unwrap();
    let fov_margin_to_center = instrument
        .fov_margin_deg(sc_attitude_to_body, lro_state, moon_center)
        .unwrap();
    assert!((fov_margin_to_center - 10.0).abs() < 1e-12);

    // Check that we can see nadir.
    let (lat, long, alt) = almanac
        .transform_to(lro_state, IAU_MOON_FRAME, None)
        .unwrap()
        .latlongalt()
        .unwrap();

    // Rebuild a location there on the surface of the Moon.
    let below = Orbit::try_latlongalt(
        lat,
        long,
        0.0,
        epoch,
        almanac.frame_info(IAU_MOON_FRAME).unwrap(),
    )
    .unwrap();

    let fov_margin_to_nadir = instrument
        .fov_margin_deg(sc_attitude_to_body, lro_state, below)
        .unwrap();
    assert!((fov_margin_to_nadir - 10.0).abs() < 1e-12);

    // Ensure that the same call from the almanac returns the same data.
    let fov_margin_deg2 = almanac
        .instrument_field_of_view_margin(1, sc_attitude_to_body, lro_state, below)
        .unwrap();

    assert_eq!(fov_margin_deg2, fov_margin_to_nadir);

    // IMPORTANT: In this test case, we've grabbed the LRO state in the IAU Moon frame.
    // We're also seeking the footprint in the IAU Moon frame. So the target_orientation_to_fixed
    // quaternion is actually identity.
    // Proof: if we pass in the rotation matrix J2000 to IAU, the footprint computation will raise an error.

    // Grab the rotation of the target.
    let dcm = almanac.rotate(MOON_J2000, IAU_MOON_FRAME, epoch).unwrap();
    let target_orientation_to_fixed = EulerParameter::from(dcm);
    assert!(instrument
        .footprint(
            sc_attitude_to_body,
            lro_state,
            target_orientation_to_fixed,
            36,
        )
        .is_err());

    // But if we pass in identity, then the footprint is correctly computed.
    let footprint = instrument
        .footprint(
            sc_attitude_to_body,
            lro_state,
            EulerParameter::identity(IAU_MOON, IAU_MOON),
            36,
        )
        .unwrap();
    println!("{below}");

    let mut min_lat_deg = f64::MAX;
    let mut max_lat_deg = f64::MIN;
    let mut min_long_deg = f64::MAX;
    let mut max_long_deg = f64::MIN;
    for ray in footprint {
        // Compute the lat/long
        let (lat_deg, long_deg, _alt_km) = ray.latlongalt().unwrap();
        min_lat_deg = min_lat_deg.min(lat_deg);
        min_long_deg = min_long_deg.min(long_deg);
        max_lat_deg = max_lat_deg.max(lat_deg);
        max_long_deg = max_long_deg.max(long_deg);
    }

    println!("LRO lat/long/alt: {lat:.2} deg, {long:.2} deg, {alt:.2} km");
    println!("Camera: {instrument}");
    println!("Latitude footprint span: {min_lat_deg:.2} - {max_lat_deg:.2}");
    println!("Longitude footprint span: {min_long_deg:.2} - {max_long_deg:.2}");

    assert!((min_lat_deg..max_lat_deg).contains(&lat) && (min_lat_deg - 55.0).abs() < 1.0);
    assert!((min_long_deg..max_long_deg).contains(&long) && (min_long_deg - 190.0).abs() < 1.0);

    // --- TEST CASE 1: EDGE OF WIDTH (X-Axis) ---
    // Camera is defined with X_Half_Angle = 15.0 deg.
    // We currently rotate -90 deg about Y to look at Nadir.
    // If we rotate -105 deg (-90 - 15), the Nadir point should shift 15 deg
    // to the edge of the Camera X-axis.

    let q_to_i_edge = EulerParameter::about_y((-90.0 - 15.0_f64).to_radians(), -30100, -30101);

    let cam_edge = Instrument {
        q_to_i: q_to_i_edge,
        ..instrument // Keep other fields same
    };

    let margin_edge = cam_edge
        .fov_margin_deg(sc_attitude_to_body, lro_state, below)
        .unwrap();

    println!("Edge Margin (Expected ~0.0): {margin_edge:.4}");
    assert!((margin_edge).abs() < 1e-3);

    // --- TEST CASE 2: OUTSIDE HEIGHT (Y-Axis) ---
    // Camera Y_Half_Angle = 10.0 deg.
    // In the nominal setup (-90 Y rot), Camera Y aligns with Body Y.
    // To shift the target along Camera Y, we must rotate around Camera X.
    // Camera X aligns with Body Z.
    // Let's rotate the mounting 12 deg about Body Z (Cross-track).

    // We compose rotations: First the Base (-90 Y), THEN a tilt (12 Z).
    // q_tilt = q_z(12) * q_y(-90)
    let base_rot = EulerParameter::about_y(-FRAC_PI_2, -30100, -30101);
    // Note: Temporary frame IDs used for composition logic
    let tilt = EulerParameter::about_x(12.0_f64.to_radians(), -30101, -30102);

    // We manually construct the combined rotation for the instrument struct
    // effectively: q_body_to_new_inst
    let mounting_outside = (tilt * base_rot).unwrap();
    // Fix the IDs manually because the multiplication output will be -30100 -> -30102
    let mut mounting_outside_fixed = mounting_outside;
    mounting_outside_fixed.to = -30101;

    let cam_outside = Instrument {
        q_to_i: mounting_outside_fixed,
        ..instrument
    };

    let margin_outside = cam_outside
        .fov_margin_deg(sc_attitude_to_body, lro_state, below)
        .unwrap();

    println!("Outside Margin (Expected ~ -2.0): {margin_outside:.4}");
    assert!((margin_outside - -2.0).abs() < 1e-3);

    // --- TEST CASE 3: ZENITH (Looking Away) ---
    // Rotate +90 deg about Y (Look at Body +X / Zenith).
    let mounting_zenith = EulerParameter::about_y(FRAC_PI_2, -30100, -30101);

    let cam_zenith = Instrument {
        q_to_i: mounting_zenith,
        ..instrument
    };

    let margin_zenith = cam_zenith
        .fov_margin_deg(sc_attitude_to_body, lro_state, below)
        .unwrap();

    println!("Zenith Margin (Expected negative large): {margin_zenith:.4}",);
    assert!(margin_zenith < -100.0);
    assert!(!cam_zenith
        .is_target_in_fov(sc_attitude_to_body, lro_state, below)
        .unwrap());
}

#[rstest]
fn lro_camera_fov_from_analysis(mut almanac: Almanac) {
    let iau_moon = almanac.frame_info(IAU_MOON_FRAME).unwrap();

    let lro_frame = Frame::new(-85, J2000);
    let start = almanac.spk_domain(-85).unwrap().0;
    let epoch = start + Unit::Day * 15;

    // Define a location on the Moon
    let vikram = Location {
        latitude_deg: -69.373,
        longitude_deg: -32.319,
        height_km: 0.5,
        frame: iau_moon.into(),
        terrain_mask: vec![],
        terrain_mask_ignored: true,
    };

    let mut dataset = LocationDataSet::default();
    dataset.push(vikram.clone(), Some(1), None).unwrap();
    almanac = almanac.with_location_data(dataset);

    // let landmark_vikram3 = Orbit::try_latlongalt(-69.373, -32.319, 0.5, epoch, iau_moon).unwrap();

    let sc_vel_align_rad_clock = DcmExpr::Triad {
        primary_axis: Box::new(VectorExpr::Fixed {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }),
        primary_vec: Box::new(VectorExpr::Velocity(StateSpec {
            target_frame: FrameSpec::Loaded(MOON_J2000),
            observer_frame: FrameSpec::Loaded(lro_frame),
            ab_corr: None,
        })),
        secondary_axis: Box::new(VectorExpr::Fixed {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        }),
        secondary_vec: Box::new(VectorExpr::Radius(StateSpec {
            target_frame: FrameSpec::Loaded(MOON_J2000),
            observer_frame: FrameSpec::Loaded(lro_frame),
            ab_corr: None,
        })),

        to: -30100,
        from: 1,
    };

    let instrument_fov_from_lro = Event {
        scalar: ScalarExpr::FovMarginToLocation {
            instrument_id: 1,
            sc_dcm_to_body: sc_vel_align_rad_clock,
            location_id: 1,
        },
        condition: Condition::GreaterThan(0.0),
        ab_corr: None,
        epoch_precision: Unit::Millisecond * 10,
    };

    let events = almanac
        .report_event_arcs(
            &StateSpec {
                target_frame: FrameSpec::Loaded(lro_frame),
                observer_frame: FrameSpec::Loaded(iau_moon),
                ab_corr: None,
            },
            &instrument_fov_from_lro,
            start,
            epoch,
        )
        .unwrap();
    println!("{}", events[0]);
    println!("{}", events[1]);
    println!("{}", events[2]);

    // Check that at the midpoint, we're roughly above the location
    let lro_mid = almanac
        .state_of(-85, iau_moon, events[0].midpoint_epoch(), None)
        .unwrap();
    let (lat, long, _) = lro_mid.latlongalt().unwrap();

    println!("LRO state = {lat} deg, {long} deg");
}
