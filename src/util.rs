use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Clone, PartialEq, Debug)]
pub struct Number {
    value: f64,
}

impl Number {}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.to_bits().hash(state);
    }
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Number { value }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
