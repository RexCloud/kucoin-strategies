use std::fmt;

#[derive(Debug, Clone)]
pub enum Condition {
    GreaterThan(f64),
    LessThan(f64),
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GreaterThan(value) => write!(f, "&gt {value}"),
            Self::LessThan(value) => write!(f, "&lt {value}"),
        }
    }
}
