use crate::client::GarminClient;
use crate::error::Result;
use crate::output::{HumanReadable, Output};
use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Profile {
    pub display_name: String,
    pub user_name: Option<String>,
    pub email: Option<String>,
    pub locale: Option<String>,
    pub measurement_system: Option<String>,
}

impl HumanReadable for Profile {
    fn print_human(&self) {
        println!("{} {}", "Name:".bold(), self.display_name);
        if let Some(ref u) = self.user_name {
            println!("{} {}", "Username:".bold(), u);
        }
        if let Some(ref e) = self.email {
            println!("{} {}", "Email:".bold(), e);
        }
        if let Some(ref l) = self.locale {
            println!("{} {}", "Locale:".bold(), l);
        }
        if let Some(ref m) = self.measurement_system {
            println!("{} {}", "Units:".bold(), m);
        }
    }
}

pub async fn show(client: &GarminClient, output: &Output) -> Result<()> {
    let v: serde_json::Value = client.get_json("/userprofile-service/usersummary").await?;
    let profile = Profile {
        display_name: v["displayName"].as_str().unwrap_or("").into(),
        user_name: v["userName"].as_str().map(Into::into),
        email: v["primaryEmail"].as_str().map(Into::into),
        locale: v["locale"].as_str().map(Into::into),
        measurement_system: v["measurementSystem"].as_str().map(Into::into),
    };
    output.print(&profile);
    Ok(())
}

pub async fn settings(client: &GarminClient, output: &Output) -> Result<()> {
    let v: serde_json::Value = client
        .get_json("/userprofile-service/userprofile/user-settings")
        .await?;
    let _ = output; // settings are always JSON - complex nested structure
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}
