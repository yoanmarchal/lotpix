// Formatting of the numbers and dates of the interface, in the current language
use chrono::{Datelike, NaiveDateTime, Timelike};

use crate::i18n::{self, Language};

/// Number writing conventions of a language.
struct NumberStyle {
    decimal: char,
    group: char,
    /// Number of digits from which thousands are separated: Spanish writes "2048" but
    /// "20.480".
    min_grouping_digits: usize,
}

fn number_style(language: Language) -> NumberStyle {
    match language {
        // Narrow no-break space between thousands
        Language::Fr => NumberStyle {
            decimal: ',',
            group: '\u{202f}',
            min_grouping_digits: 4,
        },
        Language::En => NumberStyle {
            decimal: '.',
            group: ',',
            min_grouping_digits: 4,
        },
        Language::Es => NumberStyle {
            decimal: ',',
            group: '.',
            min_grouping_digits: 5,
        },
        Language::De => NumberStyle {
            decimal: ',',
            group: '.',
            min_grouping_digits: 4,
        },
    }
}

/// Number with at most two decimals, with the separators of the current language.
fn format_number(value: f64) -> String {
    let style = number_style(i18n::current());
    let rounded = format!("{:.2}", value.abs());
    let (integer, decimals) = rounded.split_once('.').unwrap_or((&rounded, ""));
    let decimals = decimals.trim_end_matches('0');

    let mut result = String::new();
    // "-0.00" makes no sense: the sign only shows if the rounded value is not zero
    if value < 0.0 && rounded.chars().any(|c| c.is_ascii_digit() && c != '0') {
        result.push('-');
    }
    let grouped = integer.len() >= style.min_grouping_digits;
    for (index, digit) in integer.chars().enumerate() {
        if grouped && index > 0 && (integer.len() - index) % 3 == 0 {
            result.push(style.group);
        }
        result.push(digit);
    }
    if !decimals.is_empty() {
        result.push(style.decimal);
        result.push_str(decimals);
    }
    result
}

/// Readable file size, for example "1,5 Ko" or "1.5 KB".
pub fn format_file_size(bytes: u64) -> String {
    let language = i18n::current();
    if bytes < 1024 {
        // French puts 0 and 1 in the singular, the other languages only 1
        let singular = bytes == 1 || (bytes == 0 && language == Language::Fr);
        let unit = match (language, singular) {
            (Language::Fr, true) => "octet",
            (Language::Fr, false) => "octets",
            (Language::De, _) => "Byte",
            (_, true) => "byte",
            (_, false) => "bytes",
        };
        return format!("{} {unit}", format_number(bytes as f64));
    }
    let units: [&str; 3] = match language {
        Language::Fr => ["Ko", "Mo", "Go"],
        _ => ["KB", "MB", "GB"],
    };
    let mut value = bytes as f64 / 1024.0;
    let mut unit = 0;
    while value >= 1024.0 && unit < units.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{} {}", format_number(value), units[unit])
}

/// Compression saving, for example "91,38 %" or "91.38%".
pub fn format_percent(ratio: f64) -> String {
    match i18n::current() {
        Language::En => format!("{}%", format_number(ratio)),
        _ => format!("{} %", format_number(ratio)),
    }
}

/// Dimensions in pixels, for example "4000 × 3000".
pub fn format_dimensions((width, height): (u32, u32)) -> String {
    format!("{width} × {height}")
}

/// Short date and time, for example "11 oct. 2026, 18:29".
pub fn format_date(date: NaiveDateTime) -> String {
    let language = i18n::current();
    let months: [&str; 12] = match language {
        Language::Fr => [
            "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.",
            "nov.", "déc.",
        ],
        Language::En => [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ],
        Language::Es => [
            "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sept", "oct", "nov", "dic",
        ],
        Language::De => [
            "Jan.", "Feb.", "März", "Apr.", "Mai", "Juni", "Juli", "Aug.", "Sept.", "Okt.", "Nov.",
            "Dez.",
        ],
    };
    let day = match language {
        Language::De => format!("{}.", date.day()),
        _ => date.day().to_string(),
    };
    format!(
        "{day} {} {}, {:02}:{:02}",
        months[date.month0() as usize],
        date.year(),
        date.hour(),
        date.minute()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(month: u32, day: u32, hour: u32, minute: u32) -> NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(2026, month, day)
            .unwrap()
            .and_hms_opt(hour, minute, 0)
            .unwrap()
    }

    #[test]
    fn formats_dates_in_each_language() {
        assert_eq!(format_date(date(10, 11, 18, 29)), "11 oct. 2026, 18:29");
        assert_eq!(format_date(date(3, 1, 9, 5)), "1 mars 2026, 09:05");

        i18n::set_language(Language::En);
        assert_eq!(format_date(date(10, 11, 18, 29)), "11 Oct 2026, 18:29");
        i18n::set_language(Language::Es);
        assert_eq!(format_date(date(1, 5, 8, 0)), "5 ene 2026, 08:00");
        i18n::set_language(Language::De);
        assert_eq!(format_date(date(3, 1, 9, 5)), "1. März 2026, 09:05");
    }

    #[test]
    fn formats_sizes_in_french() {
        assert_eq!(format_file_size(0), "0 octet");
        assert_eq!(format_file_size(1), "1 octet");
        assert_eq!(format_file_size(512), "512 octets");
        assert_eq!(format_file_size(1536), "1,5 Ko");
        assert_eq!(format_file_size(5 * 1024 * 1024), "5 Mo");
    }

    #[test]
    fn formats_sizes_in_other_languages() {
        i18n::set_language(Language::En);
        assert_eq!(format_file_size(0), "0 bytes");
        assert_eq!(format_file_size(1), "1 byte");
        assert_eq!(format_file_size(1536), "1.5 KB");

        i18n::set_language(Language::De);
        assert_eq!(format_file_size(512), "512 Byte");
        assert_eq!(format_file_size(1536), "1,5 KB");

        i18n::set_language(Language::Es);
        assert_eq!(format_file_size(1536), "1,5 KB");
    }

    #[test]
    fn caps_at_the_largest_unit() {
        assert_eq!(format_file_size(2 * 1024u64.pow(4)), "2\u{202f}048 Go");

        i18n::set_language(Language::En);
        assert_eq!(format_file_size(2 * 1024u64.pow(4)), "2,048 GB");
        // Spanish does not separate the thousands of a four-digit number
        i18n::set_language(Language::Es);
        assert_eq!(format_file_size(2 * 1024u64.pow(4)), "2048 GB");
        assert_eq!(format_file_size(20 * 1024u64.pow(4)), "20.480 GB");
    }

    #[test]
    fn formats_percent_in_each_language() {
        assert_eq!(format_percent(91.38), "91,38 %");
        assert_eq!(format_percent(-12.5), "-12,5 %");
        assert_eq!(format_percent(-0.001), "0 %");

        i18n::set_language(Language::En);
        assert_eq!(format_percent(91.38), "91.38%");
        i18n::set_language(Language::De);
        assert_eq!(format_percent(-12.5), "-12,5 %");
    }

    #[test]
    fn formats_dimensions() {
        assert_eq!(format_dimensions((4000, 3000)), "4000 × 3000");
    }
}
