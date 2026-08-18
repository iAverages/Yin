use crate::serenity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseDurationError {
    Empty,
    Invalid,
    Overflow,
    Zero,
}

impl std::fmt::Display for ParseDurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Empty => "duration is empty",
            Self::Invalid => "duration must use combined d, h, m, or s units",
            Self::Overflow => "duration is too large",
            Self::Zero => "duration must be greater than zero",
        };
        f.write_str(message)
    }
}

impl std::error::Error for ParseDurationError {}

pub fn parse_duration(input: &str) -> Result<std::time::Duration, ParseDurationError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ParseDurationError::Empty);
    }

    let mut total = 0_u64;
    let mut number = 0_u64;
    let mut has_digits = false;
    for character in input.chars() {
        if character.is_ascii_digit() {
            number = number
                .checked_mul(10)
                .and_then(|value| value.checked_add(character.to_digit(10).unwrap() as u64))
                .ok_or(ParseDurationError::Overflow)?;
            has_digits = true;
            continue;
        }

        if !has_digits {
            return Err(ParseDurationError::Invalid);
        }
        let multiplier = match character.to_ascii_lowercase() {
            'd' => 86_400,
            'h' => 3_600,
            'm' => 60,
            's' => 1,
            _ => return Err(ParseDurationError::Invalid),
        };
        total = total
            .checked_add(
                number
                    .checked_mul(multiplier)
                    .ok_or(ParseDurationError::Overflow)?,
            )
            .ok_or(ParseDurationError::Overflow)?;
        number = 0;
        has_digits = false;
    }

    if has_digits {
        return Err(ParseDurationError::Invalid);
    }
    if total == 0 {
        return Err(ParseDurationError::Zero);
    }
    Ok(std::time::Duration::from_secs(total))
}

pub fn discord_timestamp(timestamp: serenity::Timestamp) -> String {
    let unix = timestamp.unix_timestamp();
    format!("<t:{unix}:F> (<t:{unix}:R>)")
}

pub fn format_duration(duration: std::time::Duration) -> String {
    let seconds = duration.as_secs();
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m {secs}s")
    } else if minutes > 0 {
        format!("{minutes}m {secs}s")
    } else {
        format!("{secs}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_combined_duration_units() {
        assert_eq!(parse_duration("2d3h4m5s").unwrap().as_secs(), 183_845);
    }

    #[test]
    fn accepts_uppercase_and_surrounding_whitespace() {
        assert_eq!(parse_duration(" 1H30M ").unwrap().as_secs(), 5_400);
    }

    #[test]
    fn rejects_bare_numbers_and_zero() {
        assert_eq!(parse_duration("60"), Err(ParseDurationError::Invalid));
        assert_eq!(parse_duration("0s"), Err(ParseDurationError::Zero));
    }

    #[test]
    fn rejects_missing_values_and_unknown_units() {
        assert_eq!(parse_duration("1hms"), Err(ParseDurationError::Invalid));
        assert_eq!(parse_duration("1w"), Err(ParseDurationError::Invalid));
    }
}
