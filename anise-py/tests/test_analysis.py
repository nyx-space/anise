from anise import Almanac, Aberration
import anise.analysis as analysis
from anise.time import Epoch, TimeSeries, Unit
from anise.constants import Frames
from anise.astro import Frame
from pathlib import Path


def test_analysis_gen_report():
    """
    Tests the generation of a scalar report with complex, nested expressions.
    """

    data_path = Path(__file__).parent.joinpath("..", "..", "data")
    almanac = (
        Almanac(str(data_path.joinpath("de440s.bsp")))
        .load(str(data_path.joinpath("pck08.pca")))
        .load(str(data_path.joinpath("lro.bsp")))
    )
    almanac.describe(spk=True)

    target_frame = analysis.FrameSpec.Loaded(Frames.EME2000)
    observer_frame = analysis.FrameSpec.Loaded(Frames.MOON_J2000)

    state = analysis.StateSpec(
        target_frame=target_frame,
        observer_frame=observer_frame,
        ab_corr=None,
    )

    # Build the orthogonal VNC frame of the Earth...Moon orbit
    vnc = analysis.OrthogonalFrame.XY(
        x=analysis.VectorExpr.Unit(analysis.VectorExpr.Velocity(state)),
        y=analysis.VectorExpr.Unit(analysis.VectorExpr.OrbitalMomentum(state)),
    )

    sun_state = analysis.StateSpec(
        target_frame=target_frame,
        observer_frame=analysis.FrameSpec.Loaded(Frames.SUN_J2000),
        ab_corr=Aberration("LT"),
    )

    # Project the Earth->Sun vector onto the VNC frame
    proj = analysis.VectorExpr.Project(
        v=analysis.VectorExpr.Negate(
            analysis.VectorExpr.Unit(analysis.VectorExpr.Radius(sun_state))
        ),
        frame=vnc,
        plane=analysis.Plane.XY,
    )

    # Rebuild the Local Solar Time calculation from fundamental expressions
    earth_sun = analysis.StateSpec(
        target_frame=analysis.FrameSpec.Loaded(Frames.SUN_J2000),
        observer_frame=observer_frame,
        ab_corr=Aberration("LT"),
    )
    u = analysis.VectorExpr.Unit(
        analysis.VectorExpr.CrossProduct(
            a=analysis.VectorExpr.Unit(analysis.VectorExpr.Radius(earth_sun)),
            b=analysis.VectorExpr.Unit(analysis.VectorExpr.OrbitalMomentum(state)),
        )
    )
    v = analysis.VectorExpr.CrossProduct(
        a=analysis.VectorExpr.Unit(analysis.VectorExpr.OrbitalMomentum(state)), b=u
    )
    r = analysis.VectorExpr.Radius(state)
    sin_theta = analysis.ScalarExpr.DotProduct(a=v, b=r)
    cos_theta = analysis.ScalarExpr.DotProduct(a=u, b=r)
    theta = analysis.ScalarExpr.Atan2(y=sin_theta, x=cos_theta)
    lst_prod = analysis.ScalarExpr.Mul(
        a=analysis.ScalarExpr.Mul(a=theta, b=analysis.ScalarExpr.Constant(1.0 / 180.0)),
        b=analysis.ScalarExpr.Constant(12.0),
    )
    lst_add = analysis.ScalarExpr.Add(a=lst_prod, b=analysis.ScalarExpr.Constant(6.0))
    lst = analysis.ScalarExpr.Modulo(v=lst_add, m=analysis.ScalarExpr.Constant(24.0))

    # Define all scalars to be calculated
    scalars = [
        analysis.ScalarExpr.Element(analysis.OrbitalElement.SemiMajorAxis),
        analysis.ScalarExpr.Element(analysis.OrbitalElement.Eccentricity),
        analysis.ScalarExpr.Element(analysis.OrbitalElement.Rmag),
        # IMPORTANT: Enums that do not have arguments MUST be called with ()
        # Otherwise PyO3 (the bindings) initializes them differently.
        # This causes a 'type' cannot be converted to ScalarExpr error!
        analysis.ScalarExpr.BetaAngle(),
        analysis.ScalarExpr.SolarEclipsePercentage(eclipsing_frame=Frames.VENUS_J2000),
        analysis.ScalarExpr.Norm(analysis.VectorExpr.Radius(state)),
        analysis.ScalarExpr.DotProduct(
            a=analysis.VectorExpr.EccentricityVector(state),
            b=analysis.VectorExpr.Fixed(x=1.0, y=0.0, z=0.0),
        ),
        analysis.ScalarExpr.VectorX(analysis.VectorExpr.EccentricityVector(state)),
        analysis.ScalarExpr.VectorY(analysis.VectorExpr.EccentricityVector(state)),
        analysis.ScalarExpr.VectorZ(analysis.VectorExpr.EccentricityVector(state)),
        analysis.ScalarExpr.Norm(
            analysis.VectorExpr.CrossProduct(
                a=analysis.VectorExpr.Radius(state),
                b=analysis.VectorExpr.Velocity(state),
            )
        ),
        analysis.ScalarExpr.Element(analysis.OrbitalElement.Hmag),
        analysis.ScalarExpr.AngleBetween(
            a=analysis.VectorExpr.Radius(state), b=analysis.VectorExpr.Velocity(state)
        ),
        analysis.ScalarExpr.AzimuthFromLocation(location_id=123, obstructing_body=None),
        analysis.ScalarExpr.ElevationFromLocation(
            location_id=123, obstructing_body=None
        ),
        analysis.ScalarExpr.RangeFromLocation(location_id=123, obstructing_body=None),
        analysis.ScalarExpr.RangeRateFromLocation(
            location_id=123, obstructing_body=None
        ),
        analysis.ScalarExpr.LocalTimeAscNode(),
        analysis.ScalarExpr.LocalTimeDescNode(),
        analysis.ScalarExpr.VectorX(proj),
        analysis.ScalarExpr.VectorY(proj),
        analysis.ScalarExpr.VectorZ(proj),
        analysis.ScalarExpr.LocalSolarTime(),
        lst,  # Our custom LST calculation
    ]

    # Test S-Expression serialization/deserialization
    proj_s_expr = scalars[-1].to_s_expr()
    assert len(proj_s_expr) > 0

    # Add aliases to some of the scalars for the report header
    scalars_with_aliases = [(s, None) for s in scalars]
    scalars_with_aliases[-5] = (scalars_with_aliases[-5][0], "proj VNC X")
    scalars_with_aliases[-4] = (scalars_with_aliases[-4][0], "proj VNC Y")
    scalars_with_aliases[-3] = (scalars_with_aliases[-3][0], "proj VNC Z")
    scalars_with_aliases[-1] = (scalars_with_aliases[-1][0], "LST (h)")

    # Build the final report object
    report = analysis.ReportScalars(scalars_with_aliases, state)

    # Test report serialization
    report_s_expr = report.to_s_expr()
    report_reloaded = analysis.ReportScalars.from_s_expr(report_s_expr)
    assert report_reloaded.to_s_expr() == report_s_expr

    # Generate the report data
    series = TimeSeries(
        Epoch("2025-01-01 00:00:00 UTC"),
        Epoch("2025-01-02 12:00:00 UTC"),
        Unit.Day * 0.5,
        inclusive=True,
    )
    data = almanac.report_scalars(report, series)

    assert len(data) == 4
    last_row = data["2025-01-02T12:00:00.000000"]
    assert len(last_row) == len(scalars_with_aliases)

    # Validate some computed values
    assert (
        last_row["Hmag (km)"]
        == last_row[
            "|Radius(Earth J2000 -> Moon J2000) ⨯ Velocity(Earth J2000 -> Moon J2000)|"
        ]
    )

    # Check that our manual LST is close to the built-in one
    time_diff = abs(last_row["LST (h)"] - last_row["local solar time (h)"])
    assert time_diff < (Unit.Second * 1).to_unit(Unit.Hour)

    # Check that the projections are valid (components of a unit vector)
    for col_name, value in last_row.items():
        if "proj" in col_name:
            assert abs(value) <= 1.0


def test_analysis_event():
    """
    Tests event finding for apoapsis, periapsis, eclipses, and other criteria.
    """
    data_path = Path(__file__).parent.joinpath("..", "..", "data")
    almanac = (
        Almanac(str(data_path.joinpath("de440s.bsp")))
        .load(str(data_path.joinpath("pck08.pca")))
        .load(str(data_path.joinpath("lro.bsp")))
    )

    lro_frame = Frame(-85, 1)  # LRO NAIF ID

    lro_state_spec = analysis.StateSpec(
        target_frame=analysis.FrameSpec.Loaded(lro_frame),
        observer_frame=analysis.FrameSpec.Loaded(Frames.MOON_J2000),
        ab_corr=None,
    )

    # Define several event criteria
    sunset_nadir = analysis.Event(
        scalar=analysis.ScalarExpr.SunAngle(observer_id=-85),
        desired_value=90.0,
        epoch_precision=Unit.Second * 0.5,
        value_precision=0.1,
        ab_corr=None,
    )
    apolune = analysis.Event.apoapsis()
    perilune = analysis.Event.periapsis()
    eclipse = analysis.Event.eclipse(Frames.MOON_J2000)

    # Test event serialization
    eclipse_s_expr = eclipse.to_s_expr()
    deserialized_eclipse = analysis.Event.from_s_expr(eclipse_s_expr)
    assert deserialized_eclipse.to_s_expr() == eclipse_s_expr

    # Get the time domain for LRO from the loaded ephemeris
    start_epoch, end_epoch = almanac.spk_domain(-85)
    start_orbit = almanac.transform(lro_frame, Frames.MOON_J2000, start_epoch, None)
    period = start_orbit.period()

    # Find apoapsis events
    apo_events = almanac.report_events(
        lro_state_spec, apolune, start_epoch, end_epoch, period * 0.5
    )
    for event in apo_events:
        ta_deg = event.orbit.ta_deg()
        assert abs(ta_deg - 180.0) < apolune.value_precision

    # Find periapsis events
    peri_events = almanac.report_events(
        lro_state_spec, perilune, start_epoch, end_epoch, period * 0.5
    )
    for event in peri_events:
        ta_deg = event.orbit.ta_deg()
        assert (
            ta_deg < perilune.value_precision
            or abs(ta_deg - 360.0) < perilune.value_precision
        )

    # Check that we found one apoapsis/periapsis per orbit
    dts_apo = [
        s.orbit.epoch.timedelta(f.orbit.epoch)
        for f, s in zip(apo_events[:-1], apo_events[1:])
    ]
    assert all((dt - period).abs() < Unit.Minute * 5 for dt in dts_apo)

    dts_peri = [
        s.orbit.epoch.timedelta(f.orbit.epoch)
        for f, s in zip(peri_events[:-1], peri_events[1:])
    ]
    assert all((dt - period).abs() < Unit.Minute * 5 for dt in dts_peri)

    tick = Epoch.system_now()
    # Find sunset/sunrise arcs
    sunset_arcs = almanac.report_event_arcs(
        lro_state_spec, sunset_nadir, start_epoch, end_epoch
    )
    print(f"{len(sunset_arcs)} sunset arcs found in {Epoch.system_now().timedelta(tick)}")
    assert sunset_arcs[1].rise.edge == analysis.EventEdge.Rising
    assert sunset_arcs[1].fall.edge == analysis.EventEdge.Falling
    assert len(sunset_arcs) == 309

    # Find eclipse arcs in a short time span
    tick = Epoch.system_now()
    eclipse_arcs = almanac.report_event_arcs(
        lro_state_spec, eclipse, start_epoch, start_epoch + Unit.Hour * 3
    )
    assert len(eclipse_arcs) == 2
    print("Eclipse arcs found in ", Epoch.system_now().timedelta(tick))

    # Validate the eclipse periods
    for arc in eclipse_arcs:
        assert Unit.Minute * 24 < arc.duration() < Unit.Minute * 40

        # Check points in and around the arc to confirm the event state
        series = TimeSeries(
            arc.start_epoch() - eclipse.epoch_precision,
            arc.end_epoch() + eclipse.epoch_precision,
            Unit.Minute * 0.5,
            inclusive=True,
        )
        for epoch in series:
            orbit = lro_state_spec.evaluate(epoch, almanac)
            eclipse_val = eclipse.eval(orbit, almanac)
            is_in_eclipse = abs(eclipse_val) <= eclipse.value_precision

            if arc.start_epoch() <= epoch < arc.end_epoch():
                assert is_in_eclipse, f"Epoch {epoch} should be in eclipse"
            else:
                # Outside the arc, it should not be in eclipse, or it's a falling value
                assert not is_in_eclipse or eclipse_val < 0.0
