use std::convert::TryFrom;

use anise::file_mmap;

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
fn metadata_build_read() {
    use anise::prelude::*;
    use std::fs::File;
    use std::io::Write;
    let comment_str = "Comment demo";
    let publisher_str = "ANISE Toolkit team, v0.1";
    let mut fbb = flatbuffers::FlatBufferBuilder::with_capacity(1024);
    let comments = fbb.create_string(comment_str);
    let publisher = fbb.create_string(publisher_str);
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

    let filename = "test_metadata_build_read.anis";
    // Create the file
    let mut file = File::create(filename).unwrap();
    file.write_all(fbb.finished_data()).unwrap();

    // Read the file from a manually loaded buffer
    let buf = std::fs::read(filename).unwrap();
    // We need the `... as &[u8]` because rustc fails to figure out what we want
    let ctx = Anise::try_from(&buf as &[u8]).unwrap();
    assert_eq!(ctx.metadata().comments().unwrap(), comment_str);
    assert_eq!(ctx.metadata().publisher(), publisher_str);
    assert_eq!(ctx.metadata().publication_date().hi(), 0.0);
    assert_eq!(ctx.metadata().publication_date().lo(), 0.0);

    // Read the file with the mmap macro (no allocation)
    let bytes = file_mmap!(filename).unwrap();
    let ctx = Anise::from_bytes(&bytes);
    assert_eq!(ctx.metadata().comments().unwrap(), comment_str);
    assert_eq!(ctx.metadata().publisher(), publisher_str);
    assert_eq!(ctx.metadata().publication_date().hi(), 0.0);
    assert_eq!(ctx.metadata().publication_date().lo(), 0.0);

    std::fs::remove_file(filename).unwrap();
}
