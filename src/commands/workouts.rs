use crate::client::GarminClient;
use crate::error::Result;
use crate::output::Output;

pub async fn list(client: &GarminClient, _output: &Output, limit: u32, start: u32) -> Result<()> {
    let path = format!("/workout-service/workouts?start={start}&limit={limit}");
    let v: serde_json::Value = client.get_json(&path).await?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

pub async fn get(client: &GarminClient, _output: &Output, id: u64) -> Result<()> {
    let path = format!("/workout-service/workout/{id}");
    let v: serde_json::Value = client.get_json(&path).await?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

pub async fn create(client: &GarminClient, _output: &Output, file: &str) -> Result<()> {
    let data = std::fs::read_to_string(file)?;
    let body: serde_json::Value = serde_json::from_str(&data)?;
    let result: serde_json::Value = client.post_json("/workout-service/workout", &body).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

pub async fn schedule(client: &GarminClient, output: &Output, id: u64, date: &str) -> Result<()> {
    let body = serde_json::json!({
        "date": date,
    });
    let path = format!("/workout-service/schedule/{id}");
    client.post(&path, &body).await?;
    if !output.is_json() {
        eprintln!("Scheduled workout {id} on {date}");
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "workoutId": id,
                "date": date,
                "scheduled": true,
            }))?
        );
    }
    Ok(())
}

pub async fn delete(client: &GarminClient, output: &Output, id: u64) -> Result<()> {
    let path = format!("/workout-service/workout/{id}");
    client.delete(&path).await?;
    if !output.is_json() {
        eprintln!("Deleted workout {id}");
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "workoutId": id,
                "deleted": true,
            }))?
        );
    }
    Ok(())
}
