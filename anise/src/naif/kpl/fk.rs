/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::{collections::HashMap, str::FromStr};

use log::warn;

use super::{parser::Assignment, KPLItem, KPLValue, Parameter};

#[derive(Debug, Default)]
pub struct FKItem {
    pub body_id: Option<i32>,
    pub name: Option<String>,
    pub data: HashMap<Parameter, KPLValue>,
}

impl KPLItem for FKItem {
    type Parameter = Parameter;

    /// Returns -1 on unknown tokens, and 0 on the definition token (e.g. FRAME_MOON_PA).
    fn extract_key(data: &Assignment) -> i32 {
        if data.keyword.starts_with("FRAME_") || data.keyword.starts_with("TKFRAME_") {
            let onward = &data.keyword[data.keyword.find('_').unwrap() + 1..];
            match onward.find('_') {
                Some(frm_key_pos) => match onward[..frm_key_pos].parse::<i32>() {
                    Ok(frame_id) => frame_id,
                    Err(_) => {
                        // The frame ID is not in the key, so we must be a name key.
                        data.value.trim().parse::<i32>().unwrap()
                    }
                },
                None => -1,
            }
        } else {
            -1
        }
    }

    fn data(&self) -> &HashMap<Self::Parameter, KPLValue> {
        &self.data
    }

    fn parse(&mut self, data: Assignment) {
        if data.keyword.starts_with("FRAME_") || data.keyword.starts_with("TKFRAME_") {
            match self.body_id {
                None => {
                    // The data always starts with the definition of the frame
                    // So if the body isn't set, it'll be the first item to set
                    let next_ = data.keyword.find('_').unwrap();
                    self.name = Some(data.keyword[next_ + 1..].to_string());
                    self.body_id = Some(data.value.parse::<i32>().unwrap());
                }
                Some(body_id) => {
                    // The parameter starts with the ID of the frame.
                    let param = data
                        .keyword
                        .replace("TKFRAME_", "_")
                        .replace("FRAME_", "_")
                        .replace(&format!("_{body_id}_"), "");
                    if let Ok(param) = Parameter::from_str(&param) {
                        self.data.insert(param, data.to_value());
                    } else {
                        warn!("Unknown parameter `{param}` -- ignoring");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod fk_ut {

    use crate::naif::kpl::parser::convert_fk;

    use super::{FKItem, KPLValue, Parameter};

    #[test]
    fn test_parse_fk() {
        /*
        Frames Specified by this Kernel
        =====================================================================

        Frame Name       Relative to        Type   Frame ID
        --------------   -----------------  -----  --------
        MOON_PA          MOON_PA_DE421      FIXED  31000
        MOON_ME          MOON_ME_DE421      FIXED  31001
        MOON_PA_DE421    ICRF/J2000         PCK    31006
        MOON_ME_DE421    MOON_PA_DE421      FIXED  31007

        In other words, if the class is 2 then it's a PCK based frame, else it's a TPC frame.
          */
        use crate::naif::kpl::parser::parse_file;
        let assignments = parse_file::<_, FKItem>("../data/moon_080317.txt", false).unwrap();

        // One of the `begindata` sections has two entries
        assert_eq!(assignments.len(), 5);

        // Check all of the data from this FK file
        assert_eq!(assignments[&31000].name, Some("MOON_PA".to_string()));
        assert_eq!(assignments[&31000].body_id, Some(31000));
        assert_eq!(
            assignments[&31000].data[&Parameter::Class],
            KPLValue::Integer(4)
        );
        assert_eq!(
            assignments[&31000].data[&Parameter::ClassId],
            KPLValue::Integer(31000)
        );
        assert_eq!(
            assignments[&31000].data[&Parameter::Center],
            KPLValue::Integer(301)
        );
        assert_eq!(
            assignments[&31000].data[&Parameter::Matrix],
            KPLValue::Matrix(vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0])
        );
        assert_eq!(
            assignments[&31000].data[&Parameter::Relative],
            KPLValue::String("MOON_PA_DE421".to_string())
        );

        assert_eq!(assignments[&31000].data.len(), 5);

        assert_eq!(assignments[&31001].name, Some("MOON_ME".to_string()));
        assert_eq!(assignments[&31001].body_id, Some(31001));
        assert_eq!(
            assignments[&31001].data[&Parameter::Class],
            KPLValue::Integer(4)
        );
        assert_eq!(
            assignments[&31001].data[&Parameter::ClassId],
            KPLValue::Integer(31001)
        );
        assert_eq!(
            assignments[&31001].data[&Parameter::Center],
            KPLValue::Integer(301)
        );
        assert_eq!(
            assignments[&31001].data[&Parameter::Matrix],
            KPLValue::Matrix(vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0])
        );
        assert_eq!(
            assignments[&31001].data[&Parameter::Relative],
            KPLValue::String("MOON_ME_DE421".to_string())
        );

        assert_eq!(assignments[&31001].data.len(), 5);

        assert_eq!(assignments[&31006].name, Some("MOON_PA_DE421".to_string()));
        assert_eq!(assignments[&31006].body_id, Some(31006));
        assert_eq!(assignments[&31006].data[&Parameter::Class], 2.into());
        assert_eq!(assignments[&31006].data[&Parameter::ClassId], 31006.into());
        assert_eq!(assignments[&31006].data[&Parameter::Center], 301.into());
        assert_eq!(assignments[&31006].data.len(), 3);

        assert_eq!(assignments[&31002].name, Some("MOON_PA_DE403".to_string()));
        assert_eq!(assignments[&31002].body_id, Some(31002));
        assert_eq!(assignments[&31002].data[&Parameter::Class], 2.into());
        assert_eq!(assignments[&31002].data[&Parameter::ClassId], 31002.into());
        assert_eq!(assignments[&31002].data[&Parameter::Center], 301.into());
        assert_eq!(assignments[&31002].data.len(), 3);

        assert_eq!(assignments[&31007].name, Some("MOON_ME_DE421".to_string()));
        assert_eq!(assignments[&31007].body_id, Some(31007));
        assert_eq!(assignments[&31007].data[&Parameter::Class], 4.into());
        assert_eq!(assignments[&31007].data[&Parameter::ClassId], 31007.into());
        assert_eq!(assignments[&31007].data[&Parameter::Center], 301.into());
        assert_eq!(
            assignments[&31007].data[&Parameter::Units],
            KPLValue::String("ARCSECONDS".to_string())
        );
        assert_eq!(
            assignments[&31007].data[&Parameter::Relative],
            KPLValue::String("MOON_PA_DE421".to_string())
        );
        assert_eq!(
            assignments[&31007].data[&Parameter::Angles],
            KPLValue::Matrix(vec![67.92, 78.56, 0.30])
        );
        assert_eq!(
            assignments[&31007].data[&Parameter::Axes],
            KPLValue::Matrix(vec![3.0, 2.0, 1.0])
        );
        assert_eq!(assignments[&31007].data.len(), 7);
    }

    #[test]
    fn test_convert_fk() {
        use std::path::PathBuf;
        use std::str::FromStr;

        use crate::math::rotation::{r1, r2, r3, DCM};
        let dataset = convert_fk("../data/moon_080317.txt", false).unwrap();

        assert_eq!(dataset.len(), 3, "expected three items");

        // Check that we've correctly set the names.
        let moon_me = dataset.get_by_name("MOON_ME_DE421").unwrap();
        // From the file:
        // TKFRAME_31007_ANGLES = (67.92   78.56   0.30 )
        // TKFRAME_31007_AXES   = (3,      2,      1    )
        // These angles are in arcseconds.
        let expected = r3((67.92 / 3600.0_f64).to_radians())
            * r2((78.56 / 3600.0_f64).to_radians())
            * r1((0.30 / 3600.0_f64).to_radians());
        assert!((DCM::from(moon_me).rot_mat - expected).norm() < 1e-10);
        println!("{}", dataset.crc32());
        dataset
            .save_as(&PathBuf::from_str("../data/moon_fk.epa").unwrap(), true)
            .unwrap();
    }
}
