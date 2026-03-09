use crate::client::GarminClient;
use crate::error::Result;
use crate::output::Output;

pub async fn list(client: &GarminClient, _output: &Output) -> Result<()> {
    let pk = client.profile_pk().await?;
    let path = format!("/gear-service/gear/filterGear?userProfilePk={pk}");
    let v: serde_json::Value = client.get_json(&path).await?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

pub async fn stats(client: &GarminClient, _output: &Output, uuid: &str) -> Result<()> {
    let path = format!("/gear-service/gear/stats/{uuid}");
    let v: serde_json::Value = client.get_json(&path).await?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

pub async fn link(
    client: &GarminClient,
    output: &Output,
    uuid: &str,
    activity_id: u64,
) -> Result<()> {
    let path = format!("/gear-service/gear/link/{uuid}/activity/{activity_id}");
    client.request(reqwest::Method::PUT, &path, None).await?;
    if !output.is_json() {
        eprintln!("Linked gear {uuid} to activity {activity_id}");
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "gearUUID": uuid,
                "activityId": activity_id,
                "linked": true,
            }))?
        );
    }
    Ok(())
}
