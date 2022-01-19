extern crate der;

use der::{asn1::OctetString, Decodable, Decoder, Encodable, Sequence};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ArrayOfDoubles<'a> {
    pub data: OctetString<'a>,
}

// impl<'a> Decodable<'a> for ArrayOfDoubles<'a> {
//     fn decode(decoder: &mut Decoder) -> der::Result<Self> {
//         decoder.sequence(|decoder| {
//             // let is_neg = decoder.decode()?;
//             let mantissa = decoder.decode()?;
//             // let realbase = decoder.decode()?;
//             let exponent = decoder.decode()?;
//             Ok(Self {
//                 // is_neg,
//                 mantissa,
//                 // realbase,
//                 exponent,
//             })
//             // Ok(Self {
//             //     coerced: decoder.decode()?,
//             // })
//         })
//     }
// }

// impl<'a> Sequence<'a> for ArrayOfDoubles<'a> {
//     fn fields<F, T>(&self, field_encoder: F) -> der::Result<T>
//     where
//         F: FnOnce(&[&dyn Encodable]) -> der::Result<T>,
//     {
//         field_encoder(&[&self.mantissa, &self.exponent])
//     }
// }

// impl<'a> Encodable for Real {
//     fn encoded_len(&self) -> der::Result<der::Length> {
//         todo!()
//     }

//     fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
//         todo!()
//     }
// }

// #[test]
// fn demo() {
//     // let real = Real {
//     //     mantissa: 19,
//     //     realbase: 10,
//     //     exponent: -1,
//     // };

//     // let reals = vec![Real {
//     //     is_neg: false,
//     //     mantissa: 19,
//     //     // // realbase: 10,
//     //     exponent: -1,
//     //     // coerced: 1.9_f64.to_bits(),
//     // }];
//     let reals = vec![Real::from(1.9_f64)];

//     // Encode
//     let der_encoded_real = reals.to_vec().unwrap();

//     // Decode
//     let decoded_algorithm_identifier = Vec::<Real>::from_der(&der_encoded_real).unwrap();

//     assert_eq!(reals, decoded_algorithm_identifier);
//     for x in &reals {
//         let val: Real = (*x).into();
//         dbg!(val);
//     }

//     for x in &decoded_algorithm_identifier {
//         // let val: f64 = f64::from_bits(x.coerced);
//         let val: f64 = (*x).into();
//         dbg!(val);
//     }
// }
