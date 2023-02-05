/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

// Credit: ChatGPT for 80% of the code to parse the file from the SPICE docs.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::prelude::AniseError;

use super::KPLItem;

#[derive(Debug, PartialEq, Eq)]
enum BlockType {
    Comment,
    Data,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub keyword: String,
    pub value: String,
}

impl Assignment {
    pub fn value_to_vec_f64(&self) -> Vec<f64> {
        let value = &self.value;
        let value = value.replace("(", "").replace(")", "");

        let vec: Vec<&str> = value.split_whitespace().filter(|s| !s.is_empty()).collect();
        vec.iter()
            .map(|s| s.parse::<f64>().unwrap_or(0.0))
            .collect()
    }
}

pub fn parse_file<P: AsRef<Path>, I: KPLItem>(
    file_path: P,
    show_comments: bool,
) -> Result<HashMap<i32, I>, AniseError> {
    let file = File::open(file_path).expect("Failed to open file");
    let reader = BufReader::new(file);

    let mut block_type = BlockType::Comment;
    let mut assignments = vec![];

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let tline = line.trim();

        if tline.starts_with("\\begintext") {
            block_type = BlockType::Comment;
            continue;
        } else if tline.starts_with("\\begindata") {
            block_type = BlockType::Data;
            continue;
        }

        if block_type == BlockType::Comment && show_comments {
            println!("{line}");
        } else if block_type == BlockType::Data {
            let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let keyword = parts[0];
                let value = parts[1];
                let assignment = Assignment {
                    keyword: keyword.to_string(),
                    value: value.to_string(),
                };
                assignments.push(assignment);
            } else if let Some(mut assignment) = assignments.pop() {
                // This is a continuation of the previous line, so let's grab the data and append the value we're reding now.
                // We're adding the full line with all of the extra spaces because the parsing needs those delimiters to not bunch together all of the floats.
                assignment.value += &line;
                assignments.push(assignment);
            }
        }
    }
    // Now let's parse all of the assignments and put it into a pretty hash map.
    let mut map = HashMap::new();
    for item in assignments {
        let key = I::extract_key(&item.keyword);
        if key == -1 {
            // This is metadata
            continue;
        }
        if !map.contains_key(&key) {
            map.insert(key, I::default());
        }
        let body_map = map.get_mut(&key).unwrap();
        body_map.parse(item);
    }
    Ok(map)
}
