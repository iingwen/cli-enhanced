use std::fmt::Display;

pub struct Thousands(pub u64);

impl Display for Thousands {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Thousands(mut value) = *self;
        if value == 0 {
            return write!(formatter, "0");
        }

        let mut buffer = [0u8; 32];
        let mut i_start = 32;
        let mut num_digits = 0;
        while value > 0 {
            i_start -= 1;
            if num_digits > 0 && num_digits % 3 == 0 {
                buffer[i_start] = b',';
                i_start -= 1;
            }
            let (digit, quotient) = (value % 10, value / 10);
            buffer[i_start] = u8::try_from(digit).expect("digits fit in u8") + b'0';
            num_digits += 1;
            value = quotient;
        }
        write!(
            formatter,
            "{}",
            std::str::from_utf8(&buffer[i_start..]).expect("ascii str built manually")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn formatted(value: u64) -> String {
        format!("{}", Thousands(value))
    }

  