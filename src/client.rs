use crate::auth::{self, Tokens};
use crate::error::{Error, Result};
use reqwest::header::AUTHORIZATION;
use serde::de::DeserializeOwned;
use std::sync::Mutex;

const CONNECT_API: &str = "https://connectapi.garmin.com";
const CLIENT_USER_AGENT: &str = "com.garmin.android.apps.connectmobile";

pub struct GarminClient {
    http: reqwest::Client,
    tokens: Mutex<Tokens>,
    display_name: Mutex<Option<String>>,
    profile_pk: Mutex<Option<u64>>,
}

impl GarminClient {
    pub fn new(tokens: Tokens) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(CLIENT_USER_AGENT)
            .build()?;

        Ok(Self {
            http,
            tokens: Mutex::new(tokens),
            display_name: Mutex::new(None),
            profile_pk: Mutex::new(None),
        })
    }

    /// Get a valid access token, refreshing if expired
    async fn access_token(&self) -> Result<String> {
        let mut tokens = self.tokens.lock().unwrap().clone();
        if tokens.oauth2.is_expired() {
            auth::refresh(&mut tokens).await?;
            *self.tokens.lock().unwrap() = tokens.clone();
        }
        Ok(tokens.oauth2.access_token)
    }

    /// GET JSON from Garmin Connect API
    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let token = self.access_token().await?;
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{CONNECT_API}{path}")
        };

        let resp = self
            .http
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Api(format!("{status}: {body}")));
        }

        Ok(resp.json().await?)
    }

    /// GET raw JSON Value
    pub async fn get_value(&self, path: &str) -> Result<serde_json::Value> {
        self.get_json(path).await
    }

    /// POST JSON to Garmin Connect API
    pub async fn post_json<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<T> {
        let token = self.access_token().await?;
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{CONNECT_API}{path}")
        };

        let resp = self
            .http
            .post(&url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .json(body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Api(format!("{status}: {body}")));
        }

        Ok(resp.json().await?)
    }

    /// POST with no response body expected
    pub async fn post(&self, path: &str, body: &serde_json::Value) -> Result<()> {
        let token = self.access_token().await?;
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{CONNECT_API}{path}")
        };

        let resp = self
            .http
            .post(&url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .json(body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Api(format!("{status}: {body}")));
        }

        Ok(())
    }

    /// GET raw bytes (for file downloads)
    pub async fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        let token = self.access_token().await?;
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{CONNECT_API}{path}")
        };

        let resp = self
            .http
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Api(format!("{status}: {body}")));
        }

        Ok(resp.bytes().await?.to_vec())
    }

    /// PUT with multipart (for file upload)
    pub async fn put_file(
        &self,
        path: &str,
        file_bytes: Vec<u8>,
        filename: &str,
    ) -> Result<serde_json::Value> {
        let token = self.access_token().await?;
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{CONNECT_API}{path}")
        };

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
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Api(format!("{status}: {body}")));
        }

        Ok(resp.json().await?)
    }

    /// Generic request with method
    pub async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let token = self.access_token().await?;
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{CONNECT_API}{path}")
        };

        let mut req = self
            .http
            .request(method, &url)
            .header(AUTHORIZATION, format!("Bearer {token}"));

        if let Some(body) = body {
            req = req.json(body);
        }

        let resp = req.send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Api(format!("{status}: {body}")));
        }

        Ok(resp.json().await?)
    }

    /// DELETE request with no response body expected
    pub async fn delete(&self, path: &str) -> Result<()> {
        let token = self.access_token().await?;
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{CONNECT_API}{path}")
        };

        let resp = self
            .http
            .delete(&url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Api(format!("{status}: {body}")));
        }

        Ok(())
    }

    /// Fetch and cache social profile fields
    async fn ensure_profile(&self) -> Result<()> {
        if self.display_name.lock().unwrap().is_some() {
            return Ok(());
        }

        let profile: serde_json::Value =
            self.get_json("/userprofile-service/socialProfile").await?;
        let name = profile["displayName"]
            .as_str()
            .ok_or_else(|| Error::Api("displayName not found in profile".into()))?
            .to_string();
        let pk = profile["userProfilePK"]
            .as_u64()
            .or(profile["profileId"].as_u64());

        *self.display_name.lock().unwrap() = Some(name);
        if let Some(pk) = pk {
            *self.profile_pk.lock().unwrap() = Some(pk);
        }
        Ok(())
    }

    /// Get display name, fetching from profile if not cached
    pub async fn display_name(&self) -> Result<String> {
        self.ensure_profile().await?;
        Ok(self.display_name.lock().unwrap().clone().unwrap())
    }

    /// Get user profile PK, fetching from profile if not cached
    pub async fn profile_pk(&self) -> Result<u64> {
        self.ensure_profile().await?;
        self.profile_pk
            .lock()
            .unwrap()
            .ok_or_else(|| Error::Api("userProfilePK not found in profile".into()))
    }
}
