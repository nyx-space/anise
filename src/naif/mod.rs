/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

pub mod daf;
pub mod spk;
pub mod summaries;

#[macro_export]
macro_rules! parse_bytes_as {
    ($type:ident, $input:expr, $order:expr) => {{
        let (int_bytes, _) = $input.split_at(std::mem::size_of::<$type>());

        match $order {
            Endian::Little => $type::from_le_bytes(int_bytes.try_into().unwrap()),
            Endian::Big => $type::from_be_bytes(int_bytes.try_into().unwrap()),
        }
    }};
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Endian {
    Little,
    Big,
}
