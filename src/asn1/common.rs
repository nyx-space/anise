use super::der::Enumerated;

#[derive(Enumerated, Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum InterpolationKind {
    ChebyshevSeries = 0,
    HermiteSeries = 1,
    LagrangeSeries = 2,
    Polynomial = 3,
    Trigonometric = 4, // Sometimes called Fourier Series interpolation
}
