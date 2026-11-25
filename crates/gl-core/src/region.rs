//! Region formats in use: dates, times, numbers, currency and units, written the way the
//! Region settings say. Month and weekday names follow the interface language.

use chrono::{Datelike, NaiveDateTime, Timelike};
use gl_store::{CurrencyPattern, RegionFormat, RegionSettings};
use std::sync::RwLock;

static CURRENT: RwLock<Option<RegionFormat>> = RwLock::new(None);

/// The country of the operating system's region (`nl-NL` → `NL`), `US` when unknown.
pub fn system_country() -> String {
    let locale = sys_locale::get_locale().unwrap_or_default();
    locale
        .split(['-', '_', '.', '@'])
        .skip(1)
        .find(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_alphabetic()))
        .map(str::to_ascii_uppercase)
        .unwrap_or_else(|| "US".into())
}

/// The formats in force for these settings.
pub fn effective(settings: &RegionSettings) -> RegionFormat {
    settings.effective(&system_country())
}

/// Makes these settings the formats used everywhere.
pub fn set(settings: &RegionSettings) {
    if let Ok(mut g) = CURRENT.write() {
        *g = Some(effective(settings));
    }
}

/// The local date and time, for examples.
pub fn now() -> NaiveDateTime {
    chrono::Local::now().naive_local()
}

/// The formats in force.
pub fn current() -> RegionFormat {
    CURRENT.read().ok().and_then(|g| g.clone()).unwrap_or_default()
}

const EN_M: [&str; 12] =
    ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
const EN_D: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
const ES_M: [&str; 12] =
    ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"];
const ES_D: [&str; 7] = ["lunes", "martes", "miércoles", "jueves", "viernes", "sábado", "domingo"];
const PT_M: [&str; 12] =
    ["janeiro", "fevereiro", "março", "abril", "maio", "junho", "julho", "agosto", "setembro", "outubro", "novembro", "dezembro"];
const PT_D: [&str; 7] = ["segunda-feira", "terça-feira", "quarta-feira", "quinta-feira", "sexta-feira", "sábado", "domingo"];
const FR_M: [&str; 12] =
    ["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"];
const FR_D: [&str; 7] = ["lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi", "dimanche"];
const DE_M: [&str; 12] =
    ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"];
const DE_D: [&str; 7] = ["Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag", "Sonntag"];
const IT_M: [&str; 12] =
    ["gennaio", "febbraio", "marzo", "aprile", "maggio", "giugno", "luglio", "agosto", "settembre", "ottobre", "novembre", "dicembre"];
const IT_D: [&str; 7] = ["lunedì", "martedì", "mercoledì", "giovedì", "venerdì", "sabato", "domenica"];
const NL_M: [&str; 12] =
    ["januari", "februari", "maart", "april", "mei", "juni", "juli", "augustus", "september", "oktober", "november", "december"];
const NL_D: [&str; 7] = ["maandag", "dinsdag", "woensdag", "donderdag", "vrijdag", "zaterdag", "zondag"];

fn names(lang: &str) -> (&'static [&'static str; 12], &'static [&'static str; 7]) {
    match lang.split('-').next().unwrap_or("en") {
        "es" => (&ES_M, &ES_D),
        "pt" => (&PT_M, &PT_D),
        "fr" => (&FR_M, &FR_D),
        "de" => (&DE_M, &DE_D),
        "it" => (&IT_M, &IT_D),
        "nl" => (&NL_M, &NL_D),
        _ => (&EN_M, &EN_D),
    }
}

/// The name of a weekday (Monday = 0) in the interface language.
pub fn weekday_name(index: usize) -> &'static str {
    names(crate::i18n::language()).1.get(index).copied().unwrap_or("")
}

fn short(name: &str) -> String {
    name.chars().take(3).collect()
}

/// Writes a date and time with a pattern: `yyyy`, `yy`, `MMMM`, `MMM`, `MM`, `M`, `dddd`, `ddd`,
/// `dd`, `d`, `HH`, `H`, `hh`, `h`, `mm`, `m`, `ss`, `s`, `sss` (milliseconds) and `a` (AM/PM).
/// Text in single quotes is kept as it is; anything else is copied.
pub fn pattern(pattern: &str, dt: NaiveDateTime) -> String {
    pattern_in(crate::i18n::language(), pattern, dt)
}

fn pattern_in(lang: &str, pattern: &str, dt: NaiveDateTime) -> String {
    let (months, days) = names(lang);
    let chars: Vec<char> = pattern.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' {
            if chars.get(i + 1) == Some(&'\'') {
                out.push('\'');
                i += 2;
                continue;
            }
            i += 1;
            while i < chars.len() {
                if chars[i] == '\'' {
                    if chars.get(i + 1) == Some(&'\'') {
                        out.push('\'');
                        i += 2;
                        continue;
                    }
                    break;
                }
                out.push(chars[i]);
                i += 1;
            }
            i += 1;
            continue;
        }
        if !"yMdHhmsa".contains(c) {
            out.push(c);
            i += 1;
            continue;
        }
        let n = chars[i..].iter().take_while(|x| **x == c).count();
        i += n;
        let hour12 = match dt.hour() % 12 {
            0 => 12,
            h => h,
        };
        let month = months.get(dt.month0() as usize).copied().unwrap_or("");
        let day = days.get(dt.weekday().num_days_from_monday() as usize).copied().unwrap_or("");
        let text = match (c, n) {
            ('y', 2) => format!("{:02}", dt.year().rem_euclid(100)),
            ('y', 1) => dt.year().to_string(),
            ('y', _) => format!("{:04}", dt.year()),
            ('M', 1) => dt.month().to_string(),
            ('M', 2) => format!("{:02}", dt.month()),
            ('M', 3) => short(month),
            ('M', _) => month.to_string(),
            ('d', 1) => dt.day().to_string(),
            ('d', 2) => format!("{:02}", dt.day()),
            ('d', 3) => short(day),
            ('d', _) => day.to_string(),
            ('H', 1) => dt.hour().to_string(),
            ('H', _) => format!("{:02}", dt.hour()),
            ('h', 1) => hour12.to_string(),
            ('h', _) => format!("{hour12:02}"),
            ('m', 1) => dt.minute().to_string(),
            ('m', _) => format!("{:02}", dt.minute()),
            ('s', 1) => dt.second().to_string(),
            ('s', 2) => format!("{:02}", dt.second()),
            ('s', _) => format!("{:03}", dt.nanosecond() / 1_000_000 % 1000),
            _ => if dt.hour() < 12 { "AM" } else { "PM" }.to_string(),
        };
        out.push_str(&text);
    }
    out
}

/// `9 999 999,99`: a number with the decimal and thousands symbols of the region.
pub fn number(f: &RegionFormat, value: f64, decimals: usize) -> String {
    let abs = format!("{:.*}", decimals, value.abs());
    let (int, frac) = abs.split_once('.').unwrap_or((abs.as_str(), ""));
    let sep = if f.thousands == " " { "\u{a0}" } else { f.thousands.as_str() };
    let len = int.len();
    let mut grouped = String::new();
    for (i, ch) in int.chars().enumerate() {
        let left = len - i; // digits from here to the end
        let boundary = if f.indian_grouping { left >= 3 && (left - 3) % 2 == 0 } else { left % 3 == 0 };
        if i > 0 && boundary {
            grouped.push_str(sep);
        }
        grouped.push(ch);
    }
    if !frac.is_empty() {
        grouped.push_str(&f.decimal);
        grouped.push_str(frac);
    }
    let zero = abs.chars().all(|c| c == '0' || c == '.');
    if value < 0.0 && !zero { format!("-{grouped}") } else { grouped }
}

/// An amount in the currency of the region, with the region's pattern and decimals.
pub fn currency(f: &RegionFormat, value: f64) -> String {
    let text = number(f, value.abs(), usize::from(f.currency_decimals));
    let zero = text.chars().all(|c| !c.is_ascii_digit() || c == '0');
    let negative = value < 0.0 && !zero;
    if f.currency_pattern == CurrencyPattern::Cifrao && f.currency_decimals > 0 {
        // The symbol takes the place of the decimal mark: 1 000$00.
        if let Some((int, frac)) = text.rsplit_once(f.decimal.as_str()) {
            return format!("{}{int}{}{frac}", if negative { "-" } else { "" }, f.currency_symbol);
        }
    }
    f.currency_pattern.apply(&f.currency_symbol, &text, negative)
}

/// `68,0 °F`
pub fn temperature(f: &RegionFormat, celsius: f64) -> String {
    format!("{} {}", number(f, f.temperature.from_celsius(celsius), 1), f.temperature.symbol())
}

/// `21,59 cm`
pub fn length(f: &RegionFormat, cm: f64) -> String {
    format!("{} {}", number(f, f.length.from_cm(cm), 2), f.length.symbol())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use gl_store::region::country_defaults;

    fn dt() -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, 23).unwrap().and_hms_milli_opt(21, 5, 9, 42).unwrap()
    }

    #[test]
    fn patterns() {
        assert_eq!(pattern_in("en-US", "dd/MM/yyyy", dt()), "23/11/2026");
        assert_eq!(pattern_in("en-US", "dddd, MMMM d, yyyy", dt()), "Monday, November 23, 2026");
        assert_eq!(pattern_in("es-ES", "dddd, d 'de' MMMM 'de' yyyy", dt()), "lunes, 23 de noviembre de 2026");
        assert_eq!(pattern_in("de-DE", "dddd, d. MMMM yyyy", dt()), "Montag, 23. November 2026");
        assert_eq!(pattern_in("nl-NL", "d MMM yy", dt()), "23 nov 26");
        assert_eq!(pattern_in("en-US", "h:mm a", dt()), "9:05 PM");
        assert_eq!(pattern_in("en-US", "hh:mm a", dt().with_hour(0).unwrap()), "12:05 AM");
        assert_eq!(pattern_in("en-US", "HH:mm:ss.sss", dt()), "21:05:09.042");
        assert_eq!(pattern_in("en-US", "yyyy年M月d日", dt()), "2026年11月23日");
        assert_eq!(pattern_in("en-US", "''yy 'o''clock'", dt()), "'26 o'clock");
    }

    #[test]
    fn numbers() {
        let nl = country_defaults("NL");
        let us = country_defaults("US");
        let ch = country_defaults("CH");
        let fr = country_defaults("FR");
        assert_eq!(number(&nl, 9_999_999.99, 2), "9.999.999,99");
        assert_eq!(number(&fr, 9_999_999.99, 2), "9\u{a0}999\u{a0}999,99");
        assert_eq!(number(&ch, 1_234_567.5, 2), "1'234'567.50");
        assert_eq!(number(&us, -1234.5, 2), "-1,234.50");
        assert_eq!(number(&us, 999.0, 0), "999");
        assert_eq!(number(&us, -0.001, 2), "0.00");
        assert_eq!(currency(&nl, -1234.5), "€ -1.234,50");
        assert_eq!(currency(&ch, -1234.5), "CHF-1'234.50");
        assert_eq!(currency(&us, 5.0), "$5.00");
        assert_eq!(currency(&country_defaults("JP"), 1_000_000.0), "￥1,000,000");
        assert_eq!(currency(&country_defaults("KW"), 5.0), "5.000 د.ك.");
        assert_eq!(number(&country_defaults("IN"), 1_000_000.0, 2), "10,00,000.00");
        assert_eq!(number(&country_defaults("IN"), 12_345.0, 0), "12,345");
        assert_eq!(currency(&country_defaults("CV"), -1_000_000.0), "-1\u{a0}000\u{a0}000$00");
        assert_eq!(currency(&fr, -5.0), "-5,00 €");
    }

    #[test]
    fn units() {
        let us = country_defaults("US");
        assert_eq!(temperature(&us, 20.0), "68.0 °F");
        assert_eq!(length(&us, 2.54), "1.00 in");
    }

    #[test]
    fn system_country_has_two_letters() {
        assert_eq!(system_country().len(), 2);
    }

    /// Compares every country with the example strings of the workbook the data comes from
    /// (set `GLYPHRA_REGION_EXPECTED` to the exported JSON; skipped when it is not set).
    #[test]
    fn matches_workbook_examples() {
        let Ok(path) = std::env::var("GLYPHRA_REGION_EXPECTED") else { return };
        let rows: Vec<serde_json::Value> = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let at = NaiveDate::from_ymd_opt(2026, 11, 23).unwrap().and_hms_opt(15, 45, 30).unwrap();
        let norm = |s: &str| s.replace(['\u{a0}', '\u{202f}'], " ");
        let mut bad = Vec::new();
        for r in &rows {
            let code = r["code"].as_str().unwrap();
            let f = country_defaults(code);
            let get = |k: &str| r[k].as_str().unwrap().to_string();
            let mut check = |what: &str, got: String, want: String| {
                if norm(&got) != norm(&want) {
                    bad.push(format!("{code} {what}: got {got:?}, want {want:?}"));
                }
            };
            check("number", number(&f, 1_000_000.0, 2), get("number"));
            // Austria writes plain numbers with spaces but currency amounts with dots; Glyphra has
            // one thousands symbol per region, so Austria's amounts follow its numbers.
            if code != "AT" {
                check("positive", currency(&f, 1_000_000.0), get("pos"));
                check("negative", currency(&f, -1_000_000.0), get("neg"));
            }
            check("short date", pattern_in("en-US", &f.short_date, at), get("short_date"));
            if get("clock") == "24-hour" {
                check("short time", pattern_in("en-US", &f.short_time, at), get("short_time"));
            }
        }
        assert!(bad.is_empty(), "{} differences:\n{}", bad.len(), bad.join("\n"));
    }
}
