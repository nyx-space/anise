/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

extern crate anise;
extern crate flatbuffers;

#[test]
fn it_works() {
    assert_eq!(1 + 1, 2);
}

#[test]
fn metadata_build_read() {
    use anise::prelude::*;
    // use self::anise::anise::{Anise, AniseArgs, Metadata, MetadataArgs};
    use std::fs::File;
    use std::io::Write;
    let mut fbb = flatbuffers::FlatBufferBuilder::with_capacity(1024);
    let comments = fbb.create_string("Comment demo");
    let publisher = fbb.create_string("ANISE Toolkit team, v0.1");
    let metadata = Metadata::create(
        &mut fbb,
        &MetadataArgs {
            comments: Some(comments),
            publisher: Some(publisher),
            publication_date: Some(&AniseEpoch::new(0.0, 0.0)),
            ..Default::default()
        },
    );

    let root = Anise::create(
        &mut fbb,
        &AniseArgs {
            metadata: Some(metadata),
            ..Default::default()
        },
    );
    fbb.finish(root, Some("ANIS"));

    // Create the file
    let mut file = File::create("test_metadata_build_read.anis").unwrap();
    file.write_all(fbb.finished_data()).unwrap();
}
