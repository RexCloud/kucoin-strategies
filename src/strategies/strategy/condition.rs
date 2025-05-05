use strum::Display;

#[derive(Debug, Clone, Display)]
pub enum Condition {
    #[strum(to_string = "&gt {0}")]
    GreaterThan(f64),
    #[strum(to_string = "&lt {0}")]
    LessThan(f64),
}
