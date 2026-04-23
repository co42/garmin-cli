use super::auth::{self, Tokens};
use super::types::*;
use crate::error::{Error, Result};
use chrono::NaiveDate;
use reqwest::Method;
use reqwest::header::AUTHORIZATION;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, OnceCell};

fn ymd(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

pub(crate) const CONNECT_API: &str = "https://connectapi.garmin.com";
const CLIENT_USER_AGENT: &str = "com.garmin.android.apps.connectmobile";

pub struct GarminClient {
    http: reqwest::Client,
    tokens: Mutex<Tokens>,
    profile: OnceCell<CachedProfile>,
}

/// Minimal profile data we need to build per-user API paths.
struct CachedProfile {
    display_name: String,
    profile_pk: Option<u64>,
}

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

impl GarminClient {
    pub fn new(tokens: Tokens) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(CLIENT_USER_AGENT)
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()?;

        Ok(Self {
            http,
            tokens: Mutex::new(tokens),
            profile: OnceCell::new(),
        })
    }

    async fn access_token(&self) -> Result<String> {
        let mut tokens = self.tokens.lock().await;
        if tokens.oauth2.is_expired() {
            auth::refresh(&mut tokens).await?;
        }
        Ok(tokens.oauth2.access_token.clone())
    }

    fn build_url(path: &str) -> String {
        // Accept an absolute URL only if it's https (prevents accidental plain
        // http). Otherwise treat `path` as relative to the Garmin host — every
        // internal caller passes a `/`-prefixed path.
        if path.starts_with("https://") {
            path.to_string()
        } else {
            format!("{CONNECT_API}{path}")
        }
    }

    /// Core request helper: emits tracing events and returns the raw body.
    async fn send(&self, method: Method, path: &str, body: Option<&Value>) -> Result<(reqwest::StatusCode, String)> {
        let token = self.access_token().await?;
        let url = Self::build_url(path);

        tracing::debug!(
            target: "garmin::api",
            method = %method,
            url = %url,
            body = body.map(|v| v.to_string()).as_deref().unwrap_or(""),
            "request"
        );

        let start = Instant::now();

        let mut req = self
            .http
            .request(method.clone(), &url)
            .header(AUTHORIZATION, format!("Bearer {token}"));
        if let Some(b) = body {
            req = req.json(b);
        }

        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        let elapsed_ms = start.elapsed().as_millis();

        tracing::debug!(
            target: "garmin::api",
            method = %method,
            url = %url,
            status = %status,
            elapsed_ms = %elapsed_ms,
            body_bytes = text.len(),
            body = %body_snippet(&text),
            "response"
        );

        if !status.is_success() {
            return Err(Error::Http {
                status: status.as_u16(),
                body: text,
            });
        }

        Ok((status, text))
    }

    /// GET + deserialize response into `T`.
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let (status, text) = self.send(Method::GET, path, None).await?;
        decode(status, &text)
    }

    /// GET + deserialize into `T`, mapping a 404 response to `Ok(None)`. Other
    /// errors still propagate.
    async fn get_opt<T: DeserializeOwned>(&self, path: &str) -> Result<Option<T>> {
        match self.get::<T>(path).await {
            Ok(v) => Ok(Some(v)),
            Err(Error::Http { status: 404, .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// POST JSON body + deserialize response into `T`.
    async fn post<T: DeserializeOwned>(&self, path: &str, body: &Value) -> Result<T> {
        let (status, text) = self.send(Method::POST, path, Some(body)).await?;
        decode(status, &text)
    }

    /// Send a request we don't decode. Used for write endpoints with no
    /// response body.
    async fn void(&self, method: Method, path: &str, body: Option<&Value>) -> Result<()> {
        self.send(method, path, body).await?;
        Ok(())
    }

    async fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        let token = self.access_token().await?;
        let url = Self::build_url(path);

        tracing::debug!(target: "garmin::api", method = %Method::GET, url = %url, "request");
        let start = Instant::now();

        let resp = self
            .http
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            tracing::debug!(
                target: "garmin::api",
                method = %Method::GET,
                url = %url,
                status = %status,
                elapsed_ms = %start.elapsed().as_millis(),
                body_bytes = body.len(),
                body = %body_snippet(&body),
                "response"
            );
            return Err(Error::Http {
                status: status.as_u16(),
                body,
            });
        }
        let bytes = resp.bytes().await?.to_vec();
        tracing::debug!(
            target: "garmin::api",
            method = %Method::GET,
            url = %url,
            status = %status,
            elapsed_ms = %start.elapsed().as_millis(),
            body_bytes = bytes.len(),
            "response (binary)"
        );
        Ok(bytes)
    }

    async fn upload_multipart<T: DeserializeOwned>(
        &self,
        path: &str,
        file_bytes: Vec<u8>,
        filename: &str,
    ) -> Result<T> {
        let token = self.access_token().await?;
        let url = Self::build_url(path);

        tracing::debug!(
            target: "garmin::api",
            method = %Method::POST,
            url = %url,
            filename = filename,
            body_bytes = file_bytes.len(),
            "request (multipart)"
        );
        let start = Instant::now();

        let part = reqwest::multipart::Part::bytes(file_bytes).file_name(filename.to_string());
        let form = reqwest::multipart::Form::new().part("file", part);

        let resp = self
            .http
            .post(&url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .multipart(form)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();

        tracing::debug!(
            target: "garmin::api",
            method = %Method::POST,
            url = %url,
            status = %status,
            elapsed_ms = %start.elapsed().as_millis(),
            body_bytes = text.len(),
            body = %body_snippet(&text),
            "response"
        );

        if !status.is_success() {
            return Err(Error::Http {
                status: status.as_u16(),
                body: text,
            });
        }
        decode(status, &text)
    }

    /// Raw passthrough for the `garmin api` escape hatch. Returns `Value`.
    /// An empty 2xx body maps to JSON `null` — same sentinel used by
    /// [`decode`], so callers expecting `Vec<T>` may see `null` instead of
    /// `[]` if the endpoint returns no content.
    pub async fn raw_request(&self, method: Method, path: &str, body: Option<&Value>) -> Result<Value> {
        let (_status, text) = self.send(method, path, body).await?;
        if text.is_empty() {
            return Ok(Value::Null);
        }
        Ok(serde_json::from_str(&text)?)
    }

    // ── Profile cache ────────────────────────────────────────────────

    /// Fetch and cache `/socialProfile` the first time it's needed. Subsequent
    /// callers share the `OnceCell`, so a burst of parallel requests only
    /// triggers one API call.
    async fn cached_profile(&self) -> Result<&CachedProfile> {
        self.profile
            .get_or_try_init(|| async {
                let p = self.social_profile().await?;
                if p.display_name.is_empty() {
                    return Err(Error::Other(anyhow::anyhow!(
                        "/socialProfile returned empty displayName"
                    )));
                }
                Ok(CachedProfile {
                    display_name: p.display_name,
                    profile_pk: p.user_profile_pk,
                })
            })
            .await
    }

    pub async fn display_name(&self) -> Result<String> {
        Ok(self.cached_profile().await?.display_name.clone())
    }

    pub async fn profile_pk(&self) -> Result<u64> {
        self.cached_profile()
            .await?
            .profile_pk
            .ok_or_else(|| Error::Other(anyhow::anyhow!("userProfilePK not found in profile")))
    }

    // ── Profile endpoints ────────────────────────────────────────────

    pub async fn social_profile(&self) -> Result<SocialProfile> {
        self.get("/userprofile-service/socialProfile").await
    }

    pub async fn user_settings(&self) -> Result<UserSettings> {
        self.get("/userprofile-service/userprofile/user-settings").await
    }

    pub async fn hr_zones(&self) -> Result<Vec<HrZoneEntry>> {
        self.get("/biometric-service/heartRateZones").await
    }

    pub async fn update_user_settings(&self, body: &Value) -> Result<()> {
        self.void(
            Method::PUT,
            "/userprofile-service/userprofile/user-settings",
            Some(body),
        )
        .await
    }

    pub async fn update_hr_zones(&self, body: &Value) -> Result<()> {
        self.void(Method::PUT, "/biometric-service/heartRateZones", Some(body))
            .await
    }

    // ── Summary ──────────────────────────────────────────────────────

    pub async fn daily_summary(&self, date: &str) -> Result<DailySummary> {
        let name = self.display_name().await?;
        let path = format!("/usersummary-service/usersummary/daily/{name}?calendarDate={date}");
        self.get(&path).await
    }

    // ── Health ───────────────────────────────────────────────────────

    pub async fn daily_sleep(&self, date: &str) -> Result<SleepSummary> {
        let name = self.display_name().await?;
        let path = format!("/wellness-service/wellness/dailySleepData/{name}?date={date}");
        self.get(&path).await
    }

    pub async fn sleep_scores(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<SleepScore>> {
        let (start, end) = (ymd(start), ymd(end));
        let path = format!("/wellness-service/stats/daily/sleep/score/{start}/{end}");
        self.get(&path).await
    }

    /// Shared source for stress + body battery commands.
    pub async fn daily_stress(&self, date: &str) -> Result<DailyStressResponse> {
        let path = format!("/wellness-service/wellness/dailyStress/{date}");
        self.get(&path).await
    }

    pub async fn daily_heart_rate(&self, date: &str) -> Result<HeartRateDay> {
        let name = self.display_name().await?;
        let path = format!("/wellness-service/wellness/dailyHeartRate/{name}?date={date}");
        self.get(&path).await
    }

    pub async fn daily_hrv(&self, date: &str) -> Result<HrvSummary> {
        let path = format!("/hrv-service/hrv/{date}");
        self.get(&path).await
    }

    pub async fn daily_steps(&self, date: &str) -> Result<Vec<Steps>> {
        let path = format!("/usersummary-service/stats/steps/daily/{date}/{date}");
        self.get(&path).await
    }

    pub async fn weight_range(&self, start: NaiveDate, end: NaiveDate) -> Result<WeightRange> {
        let (start, end) = (ymd(start), ymd(end));
        let path = format!("/weight-service/weight/dateRange?startDate={start}&endDate={end}");
        self.get(&path).await
    }

    pub async fn daily_hydration(&self, date: &str) -> Result<Hydration> {
        let path = format!("/usersummary-service/usersummary/hydration/daily/{date}");
        self.get(&path).await
    }

    pub async fn daily_spo2(&self, date: &str) -> Result<SpO2> {
        let path = format!("/wellness-service/wellness/dailySpo2/{date}");
        self.get(&path).await
    }

    pub async fn daily_respiration(&self, date: &str) -> Result<Respiration> {
        let path = format!("/wellness-service/wellness/daily/respiration/{date}");
        self.get(&path).await
    }

    pub async fn daily_intensity_minutes(&self, date: &str) -> Result<Vec<IntensityMinutes>> {
        let path = format!("/usersummary-service/stats/im/daily/{date}/{date}");
        self.get(&path).await
    }

    // ── Training ─────────────────────────────────────────────────────

    pub async fn training_status(&self, date: &str) -> Result<TrainingStatus> {
        let path = format!("/metrics-service/metrics/trainingstatus/aggregated/{date}");
        self.get(&path).await
    }

    pub async fn training_readiness(&self, date: &str) -> Result<TrainingReadinessResponse> {
        let path = format!("/metrics-service/metrics/trainingreadiness/{date}");
        self.get(&path).await
    }

    pub async fn vo2max_daily(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<TrainingScoreRaw>> {
        let (start, end) = (ymd(start), ymd(end));
        let path = format!("/metrics-service/metrics/maxmet/daily/{start}/{end}");
        self.get(&path).await
    }

    pub async fn race_predictions(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<RacePredictionsRaw>> {
        let name = self.display_name().await?;
        let (start, end) = (ymd(start), ymd(end));
        let path = format!(
            "/metrics-service/metrics/racepredictions/daily/{name}?fromCalendarDate={start}&toCalendarDate={end}"
        );
        self.get(&path).await
    }

    pub async fn endurance_score(&self, date: &str) -> Result<EnduranceScoreRaw> {
        let path = format!("/metrics-service/metrics/endurancescore?calendarDate={date}");
        self.get(&path).await
    }

    pub async fn hill_score(&self, date: &str) -> Result<HillScore> {
        let path = format!("/metrics-service/metrics/hillscore?calendarDate={date}");
        self.get(&path).await
    }

    pub async fn fitness_age(&self, date: &str) -> Result<FitnessAgeRaw> {
        let path = format!("/fitnessage-service/fitnessage/{date}");
        self.get(&path).await
    }

    pub async fn lactate_threshold_hr(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<BiometricDataPoint>> {
        let (start, end) = (ymd(start), ymd(end));
        let path = format!(
            "/biometric-service/stats/lactateThresholdHeartRate/range/{start}/{end}?aggregation=daily&aggregationStrategy=LATEST&sport=RUNNING"
        );
        self.get(&path).await
    }

    pub async fn lactate_threshold_speed(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<BiometricDataPoint>> {
        let (start, end) = (ymd(start), ymd(end));
        let path = format!(
            "/biometric-service/stats/lactateThresholdSpeed/range/{start}/{end}?aggregation=daily&aggregationStrategy=LATEST&sport=RUNNING"
        );
        self.get(&path).await
    }

    // ── Activities ───────────────────────────────────────────────────

    pub async fn list_activities(
        &self,
        limit: u32,
        start: u32,
        activity_type: Option<&str>,
        after: Option<&str>,
        before: Option<&str>,
    ) -> Result<Vec<ActivitySummary>> {
        let mut path = format!("/activitylist-service/activities/search/activities?limit={limit}&start={start}");
        if let Some(t) = activity_type {
            path.push_str(&format!("&activityType={}", urlencoding::encode(t)));
        }
        // Server-side date filter. `startDate`/`endDate` are inclusive (YYYY-MM-DD).
        if let Some(d) = after {
            path.push_str(&format!("&startDate={}", urlencoding::encode(d)));
        }
        if let Some(d) = before {
            path.push_str(&format!("&endDate={}", urlencoding::encode(d)));
        }
        self.get(&path).await
    }

    /// Merged summary (list endpoint) + detail (summaryDTO) view.
    /// Fetches both endpoints in parallel and combines the results.
    pub async fn activity(&self, id: u64) -> Result<Option<Activity>> {
        let (summary, detail) = tokio::try_join!(self.activity_summary(id), self.activity_detail(id))?;
        Ok(summary
            .zip(detail)
            .map(|(summary, detail)| Activity { summary, detail }))
    }

    async fn activity_summary(&self, id: u64) -> Result<Option<ActivitySummary>> {
        let path = format!("/activitylist-service/activities/search/activities?limit=1&start=0&activityId={id}");
        let v: Vec<ActivitySummary> = self.get(&path).await?;
        Ok(v.into_iter().next())
    }

    /// Extracts `summaryDTO` from `/activity-service/activity/{id}`; fields
    /// not on the list endpoint (stamina, RPE, normalized power, ...).
    async fn activity_detail(&self, id: u64) -> Result<Option<ActivityDetail>> {
        #[derive(serde::Deserialize)]
        struct Envelope {
            // API uses all-caps `summaryDTO`; rename_all would produce `summaryDto`.
            #[serde(rename = "summaryDTO")]
            summary_dto: ActivityDetail,
        }
        let path = format!("/activity-service/activity/{id}");
        Ok(self.get_opt::<Envelope>(&path).await?.map(|e| e.summary_dto))
    }

    pub async fn activity_details(&self, id: u64) -> Result<Value> {
        // Raw passthrough — the `details` endpoint returns time-series data.
        let path = format!("/activity-service/activity/{id}/details");
        self.get(&path).await
    }

    pub async fn activity_splits(&self, id: u64) -> Result<Vec<ActivitySplit>> {
        let path = format!("/activity-service/activity/{id}/splits");
        let resp: SplitsResponse = self.get(&path).await?;
        let mut laps = resp.laps;
        for (i, lap) in laps.iter_mut().enumerate() {
            lap.split = (i + 1) as i64;
        }
        Ok(laps)
    }

    pub async fn activity_hr_zones(&self, id: u64) -> Result<Vec<HrZone>> {
        let path = format!("/activity-service/activity/{id}/hrTimeInZones");
        self.get(&path).await
    }

    pub async fn activity_weather(&self, id: u64) -> Result<ActivityWeather> {
        let path = format!("/activity-service/activity/{id}/weather");
        self.get(&path).await
    }

    pub async fn activity_laps(&self, id: u64) -> Result<Vec<ActivityLap>> {
        let path = format!("/activity-service/activity/{id}/laps");
        let resp: LapsResponse = self.get(&path).await?;
        let mut laps = resp.laps;
        for (i, lap) in laps.iter_mut().enumerate() {
            lap.lap_number = (i + 1) as i64;
        }
        Ok(laps)
    }

    pub async fn activity_exercises(&self, id: u64) -> Result<Value> {
        // Variable shape; raw passthrough.
        let path = format!("/activity-service/activity/{id}/exerciseSets");
        self.get(&path).await
    }

    /// Power zones API returns `[{ "zones": [...] }]`. Flatten to the inner list.
    pub async fn activity_power_zones(&self, id: u64) -> Result<Vec<PowerZone>> {
        let path = format!("/activity-service/activity/{id}/powerTimeInZones");
        let groups: Vec<PowerZoneGroup> = self.get(&path).await?;
        Ok(groups.into_iter().flat_map(|g| g.zones).collect())
    }

    pub async fn download_activity(&self, id: u64, format: &str) -> Result<Vec<u8>> {
        let path = match format {
            "gpx" => format!("/download-service/export/gpx/activity/{id}"),
            "tcx" => format!("/download-service/export/tcx/activity/{id}"),
            _ => format!("/download-service/files/activity/{id}"),
        };
        self.get_bytes(&path).await
    }

    pub async fn upload_activity(&self, file_bytes: Vec<u8>, filename: &str, ext: &str) -> Result<Value> {
        let path = format!("/upload-service/upload/.{ext}");
        self.upload_multipart(&path, file_bytes, filename).await
    }

    // ── Workouts ─────────────────────────────────────────────────────

    pub async fn list_workouts(&self, limit: u32, start: u32) -> Result<Vec<WorkoutSummary>> {
        let path = format!("/workout-service/workouts?start={start}&limit={limit}");
        self.get(&path).await
    }

    pub async fn workout(&self, id: u64) -> Result<Workout> {
        let path = format!("/workout-service/workout/{id}");
        self.get(&path).await
    }

    pub async fn create_workout(&self, body: &Value) -> Result<Value> {
        self.post("/workout-service/workout", body).await
    }

    pub async fn schedule_workout(&self, id: u64, date: &str) -> Result<()> {
        let body = serde_json::json!({ "date": date });
        let path = format!("/workout-service/schedule/{id}");
        self.void(Method::POST, &path, Some(&body)).await
    }

    pub async fn update_workout(&self, id: u64, body: &Value) -> Result<()> {
        let path = format!("/workout-service/workout/{id}");
        self.void(Method::PUT, &path, Some(body)).await
    }

    pub async fn delete_workout(&self, id: u64) -> Result<()> {
        let path = format!("/workout-service/workout/{id}");
        self.void(Method::DELETE, &path, None).await
    }

    // ── Coach ────────────────────────────────────────────────────────

    pub async fn list_coach_workouts(&self) -> Result<Vec<CoachWorkout>> {
        self.get("/workout-service/fbt-adaptive").await
    }

    pub async fn coach_workout(&self, uuid: &str) -> Result<CoachWorkout> {
        let path = format!("/workout-service/fbt-adaptive/{uuid}");
        self.get(&path).await
    }

    /// Adaptive plan detail. Returns task list, phases, and supplemental
    /// sports in addition to the fields the non-adaptive endpoint exposes.
    /// Falls back to the non-adaptive endpoint on 404 so non-adaptive plans
    /// still render (without tasks/phases).
    pub async fn training_plan(&self, plan_id: u64) -> Result<CoachPlan> {
        let adaptive = format!("/trainingplan-service/trainingplan/fbt-adaptive/{plan_id}");
        match self.get::<CoachPlan>(&adaptive).await {
            Ok(plan) => Ok(plan),
            Err(Error::Http { status: 404, .. }) => {
                let fallback = format!("/trainingplan-service/trainingplan/{plan_id}");
                self.get(&fallback).await
            }
            Err(e) => Err(e),
        }
    }

    pub async fn list_training_plans(&self) -> Result<Vec<TrainingPlanSummary>> {
        let res: TrainingPlanListResponse = self.get("/trainingplan-service/trainingplan/plans?limit=50").await?;
        Ok(res.training_plan_list)
    }

    pub async fn plan_events(&self, plan_id: u64) -> Result<Vec<TargetEvent>> {
        let path = format!("/calendar-service/events?trainingPlanId={plan_id}");
        self.get(&path).await
    }

    pub async fn calendar_event(&self, event_id: u64) -> Result<TargetEvent> {
        let path = format!("/calendar-service/event/{event_id}");
        self.get(&path).await
    }

    pub async fn event_projections(
        &self,
        event_id: u64,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<EventProjection>> {
        let (start, end) = (ymd(start), ymd(end));
        let path = format!("/metrics-service/metrics/eventracetimeprojections/{event_id}/{start}/{end}");
        self.get(&path).await
    }

    // ── Courses ──────────────────────────────────────────────────────

    pub async fn list_courses(&self) -> Result<Vec<Course>> {
        self.get("/course-service/course").await
    }

    pub async fn course(&self, id: u64) -> Result<Course> {
        let path = format!("/course-service/course/{id}");
        self.get(&path).await
    }

    // ── Badges ───────────────────────────────────────────────────────

    pub async fn earned_badges(&self) -> Result<Vec<Badge>> {
        self.get("/badge-service/badge/earned").await
    }

    // ── Calendar ─────────────────────────────────────────────────────

    /// Pass a 1-12 month; the API expects 0-indexed, we convert.
    pub async fn calendar_month(&self, year: u32, month: u32) -> Result<Vec<CalendarItem>> {
        let api_month = month - 1;
        let path = format!("/calendar-service/year/{year}/month/{api_month}");
        let resp: CalendarMonth = self.get(&path).await?;
        Ok(resp.into_items())
    }

    pub async fn delete_calendar_entry(&self, id: u64) -> Result<()> {
        let path = format!("/workout-service/schedule/{id}");
        self.void(Method::DELETE, &path, None).await
    }

    // ── Devices ──────────────────────────────────────────────────────

    pub async fn list_devices(&self) -> Result<Vec<Device>> {
        self.get("/device-service/deviceregistration/devices").await
    }

    pub async fn device(&self, id: u64) -> Result<Device> {
        let path = format!("/device-service/deviceregistration/devices/{id}");
        self.get(&path).await
    }

    // ── Gear ─────────────────────────────────────────────────────────

    pub async fn list_gear(&self) -> Result<Vec<GearItem>> {
        let pk = self.profile_pk().await?;
        let path = format!("/gear-service/gear/filterGear?userProfilePk={pk}");
        self.get(&path).await
    }

    pub async fn gear_stats(&self, uuid: &str) -> Result<GearStats> {
        let path = format!("/gear-service/gear/stats/{uuid}");
        let mut stats: GearStats = self.get(&path).await?;
        stats.uuid = uuid.to_string();
        Ok(stats)
    }

    pub async fn link_gear(&self, uuid: &str, activity_id: u64) -> Result<()> {
        let path = format!("/gear-service/gear/link/{uuid}/activity/{activity_id}");
        self.void(Method::PUT, &path, None).await
    }

    // ── Personal records ─────────────────────────────────────────────

    pub async fn personal_records(&self) -> Result<Vec<PersonalRecordEntry>> {
        let name = self.display_name().await?;
        let path = format!("/personalrecord-service/personalrecord/prs/{name}");
        self.get(&path).await
    }

    pub async fn personal_record_types(&self) -> Result<Vec<PersonalRecordType>> {
        let name = self.display_name().await?;
        let path = format!("/personalrecord-service/personalrecordtype/prtypes/{name}");
        self.get(&path).await
    }
}

/// Deserialize a 2xx response body into `T`.
///
/// Empty bodies are decoded as JSON `null`; this succeeds for `Option<T>` or
/// types with a null default, and errors otherwise. Callers expecting a
/// `Vec<T>` response may therefore see a decode error instead of an empty
/// list if the endpoint unexpectedly returns no content.
fn decode<T: DeserializeOwned>(status: reqwest::StatusCode, text: &str) -> Result<T> {
    if text.is_empty() {
        return serde_json::from_str("null")
            .map_err(|e| Error::Other(anyhow::anyhow!("empty response for {status}, cannot deserialize: {e}")));
    }
    serde_json::from_str(text).map_err(|e| {
        Error::Other(anyhow::anyhow!(
            "error decoding response (status {status}): {e}\nBody: {}",
            &text[..text.len().min(500)]
        ))
    })
}

fn body_snippet(body: &str) -> String {
    const MAX: usize = 500;
    if body.len() <= MAX {
        body.replace('\n', " ")
    } else {
        let mut s = body[..MAX].replace('\n', " ");
        s.push_str(&format!("… [{} bytes truncated]", body.len() - MAX));
        s
    }
}
