extern crate der;

use der::{Decodable, Decoder, Encodable, Sequence};

/// X.509 `AlgorithmIdentifier` (same as above)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Real {
    // mantissa: i32,
    // realbase: u8,
    // exponent: i32,
    coerced: u64,
}

impl From<f64> for Real {
    fn from(val: f64) -> Self {
        // Using https://math.stackexchange.com/a/1049723/17452 , but rely on Ratio::new() to compute the gcd.
        // Find the max precision of this number
        // Note: the power computations happen in i32 until the end.
        // let mut exponent: i32 = 0;
        // let mut new_val = val;
        // let ten: f64 = 10.0;

        // loop {
        //     if (new_val.floor() - new_val).abs() < f64::EPSILON {
        //         // Yay, we've found the precision of this number
        //         break;
        //     }
        //     // Multiply by the precision
        //     // Note: we multiply by powers of ten to avoid this kind of round error with f32s:
        //     // https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=b760579f103b7192c20413ebbe167b90
        //     exponent += 1;
        //     new_val = val * ten.powi(exponent);
        // }

        // let mantissa = new_val as i32;

        // Real {
        //     mantissa,
        //     // realbase: 10,
        //     exponent,
        // }
        Real {
            coerced: val as u64,
        }
    }
}

impl<'a> Decodable<'a> for Real {
    fn decode(decoder: &mut Decoder) -> der::Result<Self> {
        decoder.sequence(|decoder| {
            // let mantissa = decoder.decode()?;
            // // let realbase = decoder.decode()?;
            // let exponent = decoder.decode()?;
            // Ok(Self {
            //     mantissa,
            //     // realbase,
            //     exponent,
            // })
            Ok(Self {
                coerced: decoder.decode()?,
            })
        })
    }
}

impl<'a> Sequence<'a> for Real {
    fn fields<F, T>(&self, field_encoder: F) -> der::Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> der::Result<T>,
    {
        field_encoder(&[&self.coerced])
    }
}

// impl<'a> Encodable for Real {
//     fn encoded_len(&self) -> der::Result<der::Length> {
//         todo!()
//     }

//     fn encode(&self, encoder: &mut der::Encoder<'_>) -> der::Result<()> {
//         todo!()
//     }
// }

#[test]
fn demo() {
    // let real = Real {
    //     mantissa: 19,
    //     realbase: 10,
    //     exponent: -1,
    // };

    let reals = vec![Real {
        // mantissa: 19,
        // // realbase: 10,
        // exponent: -1,
        coerced: 1.9_f64 as u64,
    }];

    // Encode
    let der_encoded_real = reals.to_vec().unwrap();

    // Decode
    let decoded_algorithm_identifier = Vec::<Real>::from_der(&der_encoded_real).unwrap();

    assert_eq!(reals, decoded_algorithm_identifier);
}
