use crate::commands::output::{HumanReadable, LABEL_WIDTH};
use crate::garmin::types::helpers::{deser_f_to_c, deser_mph_to_kmh, deser_nested_desc, deser_nested_name};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ActivityWeather {
    /// API returns Fahrenheit under the bare key `temp`; unit suffix makes the
    /// post-conversion value unambiguous.
    #[serde(rename(deserialize = "temp"), default, deserialize_with = "deser_f_to_c")]
    pub temperature_celsius: Option<f64>,
    /// API `apparentTemp` (F); renamed to idiomatic "feels like" with unit.
    #[serde(rename(deserialize = "apparentTemp"), default, deserialize_with = "deser_f_to_c")]
    pub feels_like_celsius: Option<f64>,
    /// API `dewPoint` (F); unit suffix matches post-conversion value.
    #[serde(rename(deserialize = "dewPoint"), default, deserialize_with = "deser_f_to_c")]
    pub dew_point_celsius: Option<f64>,
    pub relative_humidity: Option<f64>,
    #[serde(rename(deserialize = "windSpeed"), default, deserialize_with = "deser_mph_to_kmh")]
    pub wind_speed_kmh: Option<f64>,
    #[serde(rename(deserialize = "windGust"), default, deserialize_with = "deser_mph_to_kmh")]
    pub wind_gust_kmh: Option<f64>,
    #[serde(rename(deserialize = "windDirection"))]
    pub wind_direction_degrees: Option<i64>,
    pub wind_direction_compass_point: Option<String>,
    /// API key is `weatherTypeDTO`; strip DTO suffix and flatten nested `desc`.
    #[serde(
        rename(deserialize = "weatherTypeDTO"),
        default,
        deserialize_with = "deser_nested_desc"
    )]
    pub weather_description: Option<String>,
    /// API key is `weatherStationDTO`; strip DTO suffix and flatten nested `name`.
    #[serde(
        rename(deserialize = "weatherStationDTO"),
        default,
        deserialize_with = "deser_nested_name"
    )]
    pub station_name: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    /// API `issueDate` is actually a full ISO datetime; rename to match.
    #[serde(rename(deserialize = "issueDate"))]
    pub timestamp: Option<String>,
}

impl HumanReadable for ActivityWeather {
    fn print_human(&self) {
        println!("{}", "Weather".bold());
        println!("{}", "\u{2500}".repeat(40).dimmed());
        if let Some(temp) = self.temperature_celsius {
            let feels = self
                .feels_like_celsius
                .map(|f| format!(" (feels like {f:.0}\u{b0}C)"))
                .unwrap_or_default();
            println!("  {:<LABEL_WIDTH$}{:.0}\u{b0}C{feels}", "Temperature:", temp);
        }
        if let Some(hum) = self.relative_humidity {
            println!("  {:<LABEL_WIDTH$}{:.0}%", "Humidity:", hum);
        }
        if let Some(wind) = self.wind_speed_kmh {
            let dir = self.wind_direction_compass_point.as_deref().unwrap_or("");
            println!("  {:<LABEL_WIDTH$}{:.0} km/h {dir}", "Wind:", wind);
        }
        if let Some(ref desc) = self.weather_description {
            println!("  {:<LABEL_WIDTH$}{desc}", "Conditions:");
        }
        if let Some(ref station) = self.station_name {
            println!("  {:<LABEL_WIDTH$}{station}", "Station:");
        }
    }
}
