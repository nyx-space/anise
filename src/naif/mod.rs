/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod daf;
pub mod spk;

#[macro_export]
macro_rules! parse_bytes_as {
    ($type:ident, $input:expr, $order:expr) => {{
        let (int_bytes, _) = $input.split_at(std::mem::size_of::<$type>());

        match $order {
            Endianness::Little => $type::from_le_bytes(int_bytes.try_into().unwrap()),
            Endianness::Big => $type::from_be_bytes(int_bytes.try_into().unwrap()),
        }
    }};
}
