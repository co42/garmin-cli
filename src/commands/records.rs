use super::output::Output;
use crate::error::Result;
use crate::garmin::{GarminClient, PersonalRecord};
use std::collections::HashMap;

pub async fn run(output: &Output) -> Result<()> {
    let client = GarminClient::new(super::helpers::require_auth()?)?;
    list(&client, output).await
}

async fn list(client: &GarminClient, output: &Output) -> Result<()> {
    let (entries, types) = tokio::try_join!(client.personal_records(), client.personal_record_types(),)?;

    let type_map: HashMap<i64, _> = types.into_iter().map(|t| (t.id, t)).collect();

    let records: Vec<PersonalRecord> = entries
        .iter()
        .map(|e| PersonalRecord::from_entry(e, &type_map))
        .collect();

    output.print_list(&records, "Personal Records");
    Ok(())
}
