use anise::{
    analysis::{ReportScalars, StateSpec, FrameSpec, ScalarExpr, OrbitalElement},
    constants::Frames,
    prelude::*,
    time::{Epoch, TimeSeries, Unit},
};
use egui_extras::{Column, TableBuilder};
use eframe::egui;

pub fn analysis_ui(ui: &mut egui::Ui, almanac: &mut Almanac, report: &mut Option<ReportScalars>) {
    ui.heading("Analysis");

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Load BSP File").clicked() {
            if let Some(path_buf) = rfd::FileDialog::new().pick_file() {
                let path = path_buf.to_str().unwrap().to_string();
                *almanac = almanac.clone().load(&path).unwrap();
            }
        }
    });

    ui.separator();

    ui.heading("Scalar Report");

    if almanac.is_empty() {
        ui.label("Load a BSP file to configure a scalar report.");
    } else {
        // Dropdown to select a scalar
        let mut selected_scalar = 0;
        let scalars = [
            "SemiMajorAxis",
            "Eccentricity",
            "Rmag",
            "BetaAngle",
        ];
        egui::ComboBox::from_label("Select Scalar")
            .selected_text(scalars[selected_scalar])
            .show_ui(ui, |ui| {
                for (i, scalar) in scalars.iter().enumerate() {
                    ui.selectable_value(&mut selected_scalar, i, *scalar);
                }
            });

        if ui.button("Add Scalar").clicked() {
            // Here you would add the selected scalar to a list
        }

        if ui.button("Generate Report").clicked() {
            // Dummy state spec for now
            let state_spec = StateSpec {
                target_frame: FrameSpec::Loaded(Frames::MOON_J2000),
                observer_frame: FrameSpec::Loaded(Frames::EARTH_J2000),
                ab_corr: None,
            };
            // Dummy scalars for now
            let scalars = vec![(ScalarExpr::Element(OrbitalElement::SemiMajorAxis), None)];
            *report = Some(ReportScalars::new(scalars, state_spec));
        }
    }

    ui.separator();

    if let Some(report) = report {
        ui.heading("Report Results");

        let series = TimeSeries::inclusive(
            Epoch::from_gregorian_utc_hms(2025, 1, 1, 0, 0, 0),
            Epoch::from_gregorian_utc_hms(2025, 1, 2, 12, 0, 0),
            Unit::Day * 0.5,
        );

        let data = almanac.report_scalars(report.clone(), series);

        if data.is_err() {
            ui.label("Error generating report. Make sure the loaded BSP file covers the time range of the report.");
            return;
        }

        let data = data.unwrap();

        TableBuilder::new(ui)
            .column(Column::auto())
            .column(Column::remainder())
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("Epoch");
                });
                header.col(|ui| {
                    ui.heading("Value");
                });
            })
            .body(|mut body| {
                for (epoch, value) in data.iter() {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.label(epoch.to_gregorian_str(hifitime::TimeScale::UTC));
                        });
                        row.col(|ui| {
                            ui.label(format!("{:.2}", value[0]));
                        });
                    });
                }
            });
    }
}
