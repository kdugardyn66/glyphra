//! Region settings: units, first day of the week, date, time and number formats, currency.
//!
//! A region (a country, or "system") supplies a complete set of defaults ([`country_defaults`]).
//! With "Change settings manually" on, the user's own set ([`RegionSettings::custom`]) is used
//! instead. Rendering lives in `gl_core::region`.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Temperature {
    #[default]
    Celsius,
    Fahrenheit,
    Kelvin,
}

impl Temperature {
    pub const ALL: [Temperature; 3] = [Temperature::Celsius, Temperature::Fahrenheit, Temperature::Kelvin];
    pub fn label(self) -> &'static str {
        match self {
            Temperature::Celsius => "Celsius (°C)",
            Temperature::Fahrenheit => "Fahrenheit (°F)",
            Temperature::Kelvin => "Kelvin (K)",
        }
    }
    pub fn symbol(self) -> &'static str {
        match self {
            Temperature::Celsius => "°C",
            Temperature::Fahrenheit => "°F",
            Temperature::Kelvin => "K",
        }
    }
    pub fn from_celsius(self, c: f64) -> f64 {
        match self {
            Temperature::Celsius => c,
            Temperature::Fahrenheit => c * 9.0 / 5.0 + 32.0,
            Temperature::Kelvin => c + 273.15,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Length {
    #[default]
    Centimeter,
    Inch,
    Point,
}

impl Length {
    pub const ALL: [Length; 3] = [Length::Centimeter, Length::Inch, Length::Point];
    pub fn label(self) -> &'static str {
        match self {
            Length::Centimeter => "Centimeter (cm)",
            Length::Inch => "Inch (in)",
            Length::Point => "Point (pt)",
        }
    }
    pub fn symbol(self) -> &'static str {
        match self {
            Length::Centimeter => "cm",
            Length::Inch => "in",
            Length::Point => "pt",
        }
    }
    pub fn from_cm(self, cm: f64) -> f64 {
        match self {
            Length::Centimeter => cm,
            Length::Inch => cm / 2.54,
            Length::Point => cm / 2.54 * 72.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Weekday {
    #[default]
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Weekday {
    pub const ALL: [Weekday; 7] =
        [Weekday::Monday, Weekday::Tuesday, Weekday::Wednesday, Weekday::Thursday, Weekday::Friday, Weekday::Saturday, Weekday::Sunday];
    /// Monday = 0 … Sunday = 6.
    pub fn index(self) -> usize {
        Weekday::ALL.iter().position(|d| *d == self).unwrap_or(0)
    }
}

/// How a currency amount is written, positive and negative (the symbol is `€` in the examples).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurrencyPattern {
    /// `1,00 €` / `-1,00 €`
    #[serde(alias = "minus_number_symbol")]
    NumberSpaceSymbol,
    /// `€1.00` / `-€1.00`
    #[serde(alias = "minus_symbol_tight")]
    #[default]
    SymbolNumber,
    /// `€ 1,00` / `-€ 1,00`
    #[serde(alias = "minus_symbol_number")]
    SymbolSpaceNumber,
    /// `€ 1,00` / `€ -1,00`
    #[serde(alias = "symbol_minus_number")]
    SymbolSpaceMinus,
    /// `CHF 1.00` / `CHF-1.00`
    SymbolSpaceMinusTight,
    /// `$1.000` / `$-1.000`
    SymbolMinusTight,
    /// `1,00€` / `-1,00€`
    NumberSymbol,
    /// Cape Verde: the symbol is the decimal mark, `1 000$00` / `-1 000$00`.
    Cifrao,
}

impl CurrencyPattern {
    pub const ALL: [CurrencyPattern; 8] = [
        CurrencyPattern::NumberSpaceSymbol,
        CurrencyPattern::SymbolNumber,
        CurrencyPattern::SymbolSpaceNumber,
        CurrencyPattern::SymbolSpaceMinus,
        CurrencyPattern::SymbolSpaceMinusTight,
        CurrencyPattern::SymbolMinusTight,
        CurrencyPattern::NumberSymbol,
        CurrencyPattern::Cifrao,
    ];
    /// The pattern as shown in the list: positive and negative, with `[number]` for the digits.
    pub fn label(self, symbol: &str) -> String {
        format!("{}   /   {}", self.apply(symbol, "[number]", false), self.apply(symbol, "[number]", true))
    }
    /// Writes an already formatted absolute amount.
    pub fn apply(self, symbol: &str, number: &str, negative: bool) -> String {
        match (self, negative) {
            (CurrencyPattern::NumberSpaceSymbol, false) => format!("{number} {symbol}"),
            (CurrencyPattern::NumberSpaceSymbol, true) => format!("-{number} {symbol}"),
            (CurrencyPattern::SymbolNumber, false) => format!("{symbol}{number}"),
            (CurrencyPattern::SymbolNumber, true) => format!("-{symbol}{number}"),
            (CurrencyPattern::SymbolSpaceNumber, false) => format!("{symbol} {number}"),
            (CurrencyPattern::SymbolSpaceNumber, true) => format!("-{symbol} {number}"),
            (CurrencyPattern::SymbolSpaceMinus, false) => format!("{symbol} {number}"),
            (CurrencyPattern::SymbolSpaceMinus, true) => format!("{symbol} -{number}"),
            (CurrencyPattern::SymbolSpaceMinusTight, false) => format!("{symbol} {number}"),
            (CurrencyPattern::SymbolSpaceMinusTight, true) => format!("{symbol}-{number}"),
            (CurrencyPattern::SymbolMinusTight, false) => format!("{symbol}{number}"),
            (CurrencyPattern::SymbolMinusTight, true) => format!("{symbol}-{number}"),
            (CurrencyPattern::NumberSymbol, false) => format!("{number}{symbol}"),
            (CurrencyPattern::NumberSymbol, true) => format!("-{number}{symbol}"),
            (CurrencyPattern::Cifrao, false) => format!("{number}{symbol}"),
            (CurrencyPattern::Cifrao, true) => format!("-{number}{symbol}"),
        }
    }
}

/// A complete set of region formats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RegionFormat {
    pub temperature: Temperature,
    pub length: Length,
    pub first_day: Weekday,
    pub short_date: String,
    pub long_date: String,
    pub short_time: String,
    pub long_time: String,
    pub decimal: String,
    pub thousands: String,
    pub currency_symbol: String,
    pub currency_code: String,
    pub currency_pattern: CurrencyPattern,
    /// Digits after the decimal mark in amounts (0 for yen, 3 for dinars).
    pub currency_decimals: u8,
    /// Groups of two digits after the first three (1,00,000), as in India.
    pub indian_grouping: bool,
}

impl Default for RegionFormat {
    fn default() -> Self {
        country_defaults("US")
    }
}

/// The region choice and the user's own formats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RegionSettings {
    /// `system` or a country code such as `BE`.
    pub region: String,
    /// Use `custom` instead of the region's own formats.
    pub manual: bool,
    pub custom: RegionFormat,
}

impl Default for RegionSettings {
    fn default() -> Self {
        RegionSettings { region: SYSTEM.into(), manual: false, custom: RegionFormat::default() }
    }
}

pub const SYSTEM: &str = "system";

impl RegionSettings {
    /// The country whose defaults apply: the chosen one, or the system's.
    pub fn country<'a>(&'a self, system_country: &'a str) -> &'a str {
        let c = if self.region == SYSTEM { system_country } else { self.region.as_str() };
        if has_country(c) { c } else { "US" }
    }
    /// The formats in force.
    pub fn effective(&self, system_country: &str) -> RegionFormat {
        if self.manual { self.custom.clone() } else { country_defaults(self.country(system_country)) }
    }
}

/// A country and its usual formats, read from `data/region.json` (generated from the World
/// Currencies workbook: ISO codes, symbols, number, currency, date and time formats).
#[derive(Debug, Clone, Deserialize)]
pub struct Country {
    pub code: String,
    pub name: String,
    currency: String,
    symbol: String,
    decimal: String,
    thousands: String,
    indian: bool,
    cur_decimals: u8,
    pattern: CurrencyPattern,
    short_date: String,
    long_date: String,
    short_time: String,
    long_time: String,
}

/// A currency: ISO code, English name, usual symbol and decimals.
#[derive(Debug, Clone, Deserialize)]
pub struct Currency {
    pub code: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Deserialize)]
struct Data {
    countries: Vec<Country>,
    currencies: Vec<Currency>,
}

fn data() -> &'static Data {
    static DATA: OnceLock<Data> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../data/region.json")).unwrap_or(Data { countries: Vec::new(), currencies: Vec::new() })
    })
}

/// All countries, by English name. The names are English.
pub fn countries() -> &'static [Country] {
    &data().countries
}

/// All currencies.
pub fn currencies() -> &'static [Currency] {
    &data().currencies
}

pub fn currency_info(code: &str) -> Option<&'static Currency> {
    currencies().iter().find(|c| c.code == code)
}

pub fn has_country(code: &str) -> bool {
    countries().iter().any(|c| c.code == code)
}

pub const SHORT_TIMES: [&str; 3] = ["h:mm a", "hh:mm a", "HH:mm"];
pub const LONG_TIMES: [&str; 4] = ["HH:mm:ss", "HH:mm:ss.sss", "h:mm:ss a", "hh:mm:ss a"];
pub const SHORT_DATES: [&str; 9] =
    ["dd/MM/yyyy", "dd/MM/yy", "dd.MM.yyyy", "dd.MM.yy", "dd-MM-yyyy", "MM/dd/yyyy", "MM/dd/yy", "yyyy-MM-dd", "yyyy/MM/dd"];
pub const LONG_DATES: [&str; 7] = [
    "d MMMM yyyy",
    "d. MMMM yyyy",
    "MMMM d, yyyy",
    "dddd, d MMMM yyyy",
    "dddd, MMMM d, yyyy",
    "dddd, d. MMMM yyyy",
    "d 'de' MMMM 'de' yyyy",
];
pub const DECIMALS: [&str; 2] = [".", ","];
pub const THOUSANDS: [&str; 4] = [",", ".", "'", " "];

// Not in the workbook: usual units and the first day of the week (after CLDR).
const FAHRENHEIT: &[&str] = &["US", "BS", "BZ", "KY", "PW", "LR", "AS", "GU", "PR", "VI", "UM", "MH", "FM"];
const INCH: &[&str] = &["US", "LR", "AS", "GU", "PR", "VI", "UM"];
const SUNDAY: &[&str] = &[
    "AG", "AS", "BD", "BR", "BS", "BT", "BW", "BZ", "CA", "CO", "DM", "DO", "ET", "GT", "GU", "HK", "HN", "ID", "IL", "IN", "JM", "JP",
    "KE", "KH", "KR", "LA", "MH", "MM", "MO", "MT", "MX", "MZ", "NI", "NP", "PA", "PE", "PH", "PK", "PR", "PT", "PY", "SA", "SG", "SV",
    "TH", "TT", "TW", "UM", "US", "VE", "VI", "WS", "YE", "ZA", "ZW",
];
const SATURDAY: &[&str] = &["AF", "BH", "DJ", "DZ", "EG", "IQ", "IR", "JO", "KW", "LY", "OM", "QA", "SD", "SY"];

/// The defaults of a country (the United States when the code is unknown).
pub fn country_defaults(code: &str) -> RegionFormat {
    let list = countries();
    let Some(c) = list.iter().find(|c| c.code == code).or_else(|| list.iter().find(|c| c.code == "US")) else {
        // Only without data: keeps the function total without a panic.
        return RegionFormat {
            temperature: Temperature::Celsius,
            length: Length::Centimeter,
            first_day: Weekday::Monday,
            short_date: "dd/MM/yyyy".into(),
            long_date: "d MMMM yyyy".into(),
            short_time: "HH:mm".into(),
            long_time: "HH:mm:ss".into(),
            decimal: ".".into(),
            thousands: ",".into(),
            currency_symbol: "€".into(),
            currency_code: "EUR".into(),
            currency_pattern: CurrencyPattern::SymbolNumber,
            currency_decimals: 2,
            indian_grouping: false,
        };
    };
    let code = c.code.as_str();
    RegionFormat {
        temperature: if FAHRENHEIT.contains(&code) { Temperature::Fahrenheit } else { Temperature::Celsius },
        length: if INCH.contains(&code) { Length::Inch } else { Length::Centimeter },
        first_day: if SUNDAY.contains(&code) {
            Weekday::Sunday
        } else if SATURDAY.contains(&code) {
            Weekday::Saturday
        } else {
            Weekday::Monday
        },
        short_date: c.short_date.clone(),
        long_date: c.long_date.clone(),
        short_time: c.short_time.clone(),
        long_time: c.long_time.clone(),
        decimal: c.decimal.clone(),
        thousands: c.thousands.clone(),
        currency_symbol: c.symbol.clone(),
        currency_code: c.currency.clone(),
        currency_pattern: c.pattern,
        currency_decimals: c.cur_decimals,
        indian_grouping: c.indian,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_is_consistent() {
        assert!(countries().len() > 240, "countries missing");
        let mut codes: Vec<_> = countries().iter().map(|c| c.code.as_str()).collect();
        let n = codes.len();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), n, "duplicate country code");
        for c in countries() {
            assert!(currency_info(&c.currency).is_some(), "{} has an unknown currency", c.code);
            assert_ne!(c.decimal, c.thousands, "{}: same decimal and thousands symbol", c.code);
            assert!(DECIMALS.contains(&c.decimal.as_str()) && THOUSANDS.contains(&c.thousands.as_str()), "{}: symbol not offered", c.code);
            assert!(!c.symbol.is_empty() && c.cur_decimals <= 3, "{}", c.code);
            assert!(
                c.short_date.contains("MM") && c.short_date.contains("dd") && c.short_date.contains('y'),
                "{}: {}",
                c.code,
                c.short_date
            );
            assert!(c.long_date.contains('d') && c.long_date.contains('y'), "{}: {}", c.code, c.long_date);
            assert!(c.short_time.contains('m') && c.long_time.contains("ss"), "{}: {}", c.code, c.short_time);
        }
        for c in currencies() {
            assert!(c.decimals <= 3 && !c.symbol.is_empty(), "{}", c.code);
        }
    }

    #[test]
    fn defaults_follow_the_workbook() {
        let be = country_defaults("BE");
        assert_eq!((be.decimal.as_str(), be.thousands.as_str(), be.currency_code.as_str()), (",", ".", "EUR"));
        assert_eq!(be.currency_pattern, CurrencyPattern::SymbolSpaceMinus);
        assert_eq!((be.short_date.as_str(), be.short_time.as_str()), ("dd/MM/yyyy", "HH:mm"));
        let ch = country_defaults("CH");
        assert_eq!((ch.thousands.as_str(), ch.currency_pattern), ("'", CurrencyPattern::SymbolSpaceMinusTight));
        assert!(country_defaults("IN").indian_grouping);
        assert_eq!(country_defaults("JP").currency_decimals, 0);
        assert_eq!(country_defaults("KW").currency_decimals, 3);
        assert_eq!(country_defaults("CV").currency_pattern, CurrencyPattern::Cifrao);
        assert_eq!(country_defaults("US").temperature, Temperature::Fahrenheit);
        assert_eq!(country_defaults("??").currency_code, "USD");
        let mut r = RegionSettings::default();
        assert_eq!(r.effective("CH").currency_code, "CHF");
        r.region = "BE".into();
        assert_eq!(r.effective("CH").currency_code, "EUR");
        r.manual = true;
        r.custom.currency_code = "SEK".into();
        assert_eq!(r.effective("CH").currency_code, "SEK");
    }

    #[test]
    fn currency_patterns() {
        assert_eq!(CurrencyPattern::SymbolSpaceNumber.apply("€", "5,00", true), "-€ 5,00");
        assert_eq!(CurrencyPattern::SymbolSpaceNumber.apply("€", "5,00", false), "€ 5,00");
        assert_eq!(CurrencyPattern::SymbolSpaceMinus.apply("€", "5,00", true), "€ -5,00");
        assert_eq!(CurrencyPattern::SymbolSpaceMinusTight.apply("CHF", "5.00", true), "CHF-5.00");
        assert_eq!(CurrencyPattern::SymbolMinusTight.apply("$", "5", true), "$-5");
        assert_eq!(CurrencyPattern::NumberSpaceSymbol.apply("€", "5,00", true), "-5,00 €");
        assert_eq!(CurrencyPattern::SymbolNumber.label("$"), "$[number]   /   -$[number]");
    }

    #[test]
    fn old_pattern_names_still_load() {
        let p: CurrencyPattern = serde_json::from_str("\"minus_number_symbol\"").unwrap();
        assert_eq!(p, CurrencyPattern::NumberSpaceSymbol);
    }

    #[test]
    fn units() {
        assert!((Temperature::Fahrenheit.from_celsius(100.0) - 212.0).abs() < 1e-9);
        assert!((Length::Point.from_cm(2.54) - 72.0).abs() < 1e-9);
    }
}
