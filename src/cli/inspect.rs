use hifitime::Duration;
use tabled::Tabled;

#[derive(Tabled)]
pub struct BpcRow<'a> {
    pub name: &'a str,
    pub start_epoch: String,
    pub end_epoch: String,
    pub duration: Duration,
    pub interpolation_kind: String,
    pub frame: String,
    pub inertial_frame: String,
}

#[derive(Tabled)]
pub struct SpkRow<'a> {
    pub name: &'a str,
    pub target: String,
    pub frame: String,
    pub center: i32,
    pub start_epoch: String,
    pub end_epoch: String,
    pub duration: Duration,
    pub interpolation_kind: String,
}
