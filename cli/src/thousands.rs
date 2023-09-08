use std::fmt::Display;

pub struct Thousands(pub u64);

impl Display for Thousands {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Thousands(mut value) = *self;
        if value == 0 {
            return write!(formatter, "0");
     