use crate::config;
use crate::error::{Error, Result};
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

const SSO_BASE: &str = "https://sso.garmin.com";
const CONNECT_API: &str = "https://connectapi.garmin.com";
const OAUTH_CONSUMER_URL: &str = "https://thegarth.s3.amazonaws.com/oauth_consumer.json";
const SERVICE_URL: &str = "https://mobile.integration.garmin.com/gcm/android";
const CLIENT_ID: &str = "GCM_ANDROID_DARK";

const CONNECT_USER_AGENT: &str = "com.garmin.android.apps.connectmobile";

/// Browser-like headers for SSO — Cloudflare blocks mismatched UAs.
fn sso_headers() -> reqwest::header::HeaderMap {
    use reqwest::header::{
        ACCEPT, ACCEPT_LANGUAGE, HeaderMap, HeaderName, HeaderValue, USER_AGENT,
    };
    let mut h = HeaderMap::new();
    h.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (iPhone; CPU iPhone OS 18_7 like Mac OS X) \
             AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148",
        ),
    );
    h.insert(
        ACCEPT,
        HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
    );
    h.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
    h.insert(
        HeaderName::from_static("sec-fetch-mode"),
        HeaderValue::from_static("navigate"),
    );
    h.insert(
        HeaderName::from_static("sec-fetch-dest"),
        HeaderValue::from_static("document"),
    );
    h
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tokens {
    pub consumer: ConsumerCredentials,
    pub oauth1: OAuth1Token,
    pub oauth2: OAuth2Token,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerCredentials {
    pub consumer_key: String,
    pub consumer_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth1Token {
    pub token: String,
    pub token_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Token {
    pub access_token: String,
    pub token_type: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

impl OAuth2Token {
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        now >= self.expires_at - 60
    }
}

impl Tokens {
    pub fn load() -> Result<Self> {
        let path = config::tokens_path();
        if !path.exists() {
            return Err(Error::NotAuthenticated);
        }
        let data = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn save(&self) -> Result<()> {
        let dir = config::config_dir();
        std::fs::create_dir_all(&dir)?;
        let path = config::tokens_path();
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, data)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    pub fn delete() -> Result<()> {
        let path = config::tokens_path();
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}

/// Full SSO login flow: username/password -> SSO ticket -> OAuth1 -> OAuth2
pub async fn login(email: &str, password: &str) -> Result<Tokens> {
    let ticket = sso_login(email, password).await?;
    let consumer = fetch_consumer_credentials().await?;
    let oauth1 = exchange_ticket_for_oauth1(&ticket, &consumer).await?;
    let oauth2 = exchange_oauth1_for_oauth2(&oauth1, &consumer, true).await?;

    let tokens = Tokens {
        consumer,
        oauth1,
        oauth2,
    };
    tokens.save()?;
    Ok(tokens)
}

/// Refresh OAuth2 token using stored OAuth1 credentials
pub async fn refresh(tokens: &mut Tokens) -> Result<()> {
    tokens.oauth2 = exchange_oauth1_for_oauth2(&tokens.oauth1, &tokens.consumer, false).await?;
    tokens.save()?;
    Ok(())
}

// --- SSO flow (mobile API, March 2026) ---

async fn sso_login(email: &str, password: &str) -> Result<String> {
    let jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
    let client = reqwest::Client::builder()
        .cookie_provider(jar)
        .default_headers(sso_headers())
        .build()?;

    let login_params = [
        ("clientId", CLIENT_ID),
        ("locale", "en-US"),
        ("service", SERVICE_URL),
    ];

    // Step 1: GET /mobile/sso/en/sign-in — sets cookies
    client
        .get(format!("{SSO_BASE}/mobile/sso/en/sign-in"))
        .query(&[("clientId", CLIENT_ID)])
        .header("Sec-Fetch-Site", "none")
        .send()
        .await?;

    // Step 2: POST /mobile/api/login — submit credentials as JSON
    let resp = client
        .post(format!("{SSO_BASE}/mobile/api/login"))
        .query(&login_params)
        .json(&serde_json::json!({
            "username": email,
            "password": password,
            "rememberMe": false,
            "captchaToken": "",
        }))
        .send()
        .await?;

    let status = resp.status();
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|_| Error::Auth(format!("SSO returned non-JSON response (HTTP {status})")))?;

    let resp_type = body
        .pointer("/responseStatus/type")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match resp_type {
        "SUCCESSFUL" => {
            let ticket = body["serviceTicketId"]
                .as_str()
                .ok_or_else(|| Error::Auth("No ticket in SSO response".into()))?
                .to_string();
            complete_sso(&client).await;
            Ok(ticket)
        }
        "MFA_REQUIRED" => {
            let mfa_method = body
                .pointer("/customerMfaInfo/mfaLastMethodUsed")
                .and_then(|v| v.as_str())
                .unwrap_or("email")
                .to_string();
            let ticket = handle_mfa(&client, &login_params, &mfa_method).await?;
            complete_sso(&client).await;
            Ok(ticket)
        }
        _ => {
            let message = body
                .pointer("/responseStatus/message")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let detail = if message.is_empty() {
                format!("HTTP {status}, type={resp_type}")
            } else {
                format!("{resp_type}: {message}")
            };
            Err(Error::Auth(format!("SSO login failed ({detail})")))
        }
    }
}

/// Best-effort GET to set Cloudflare LB cookie for backend pinning.
async fn complete_sso(client: &reqwest::Client) {
    let _ = client
        .get(format!("{SSO_BASE}/portal/sso/embed"))
        .header("Sec-Fetch-Site", "same-origin")
        .send()
        .await;
}

async fn handle_mfa(
    client: &reqwest::Client,
    login_params: &[(&str, &str)],
    mfa_method: &str,
) -> Result<String> {
    eprint!("MFA code: ");
    let mut mfa_code = String::new();
    std::io::stdin().read_line(&mut mfa_code)?;
    let mfa_code = mfa_code.trim();

    let resp = client
        .post(format!("{SSO_BASE}/mobile/api/mfa/verifyCode"))
        .query(login_params)
        .json(&serde_json::json!({
            "mfaMethod": mfa_method,
            "mfaVerificationCode": mfa_code,
            "rememberMyBrowser": false,
            "reconsentList": [],
            "mfaSetup": false,
        }))
        .send()
        .await?;

    let status = resp.status();
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|_| Error::Auth(format!("MFA returned non-JSON response (HTTP {status})")))?;

    let resp_type = body
        .pointer("/responseStatus/type")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if resp_type != "SUCCESSFUL" {
        let message = body
            .pointer("/responseStatus/message")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        return Err(Error::Auth(format!(
            "MFA verification failed: {resp_type} {message}"
        )));
    }

    body["serviceTicketId"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| Error::Auth("No ticket in MFA response".into()))
}

// --- OAuth flow ---

async fn fetch_consumer_credentials() -> Result<ConsumerCredentials> {
    // 1. Env var override
    if let (Ok(key), Ok(secret)) = (
        std::env::var("GARMIN_CONSUMER_KEY"),
        std::env::var("GARMIN_CONSUMER_SECRET"),
    ) {
        return Ok(ConsumerCredentials {
            consumer_key: key,
            consumer_secret: secret,
        });
    }

    // 2. Local cache
    let cache_path = config::consumer_path();
    if cache_path.exists() {
        let data = std::fs::read_to_string(&cache_path)?;
        if let Ok(creds) = serde_json::from_str::<ConsumerCredentials>(&data) {
            return Ok(creds);
        }
    }

    // 3. Fetch from S3 and cache
    let resp: serde_json::Value = reqwest::get(OAUTH_CONSUMER_URL).await?.json().await?;
    let key = resp["consumer_key"]
        .as_str()
        .ok_or_else(|| Error::Auth("Missing consumer key".into()))?
        .to_string();
    let secret = resp["consumer_secret"]
        .as_str()
        .ok_or_else(|| Error::Auth("Missing consumer secret".into()))?
        .to_string();
    let creds = ConsumerCredentials {
        consumer_key: key,
        consumer_secret: secret,
    };

    // Cache for next time
    let dir = config::config_dir();
    std::fs::create_dir_all(&dir)?;
    let data = serde_json::to_string_pretty(&creds)?;
    std::fs::write(&cache_path, data)?;

    Ok(creds)
}

async fn exchange_ticket_for_oauth1(
    ticket: &str,
    consumer: &ConsumerCredentials,
) -> Result<OAuth1Token> {
    let client = reqwest::Client::builder()
        .user_agent(CONNECT_USER_AGENT)
        .build()?;

    let base_path = format!("{CONNECT_API}/oauth-service/oauth/preauthorized");
    let encoded_ticket = urlencoding::encode(ticket);
    let url = format!(
        "{base_path}?ticket={encoded_ticket}&login-url={SERVICE_URL}&accepts-mfa-tokens=true"
    );

    let auth_header = oauth1_header(
        "GET",
        &base_path,
        &[
            ("ticket", ticket),
            ("login-url", SERVICE_URL),
            ("accepts-mfa-tokens", "true"),
        ],
        &consumer.consumer_key,
        &consumer.consumer_secret,
        None,
        None,
    );

    let resp = client
        .get(&url)
        .header("Authorization", auth_header)
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;

    if !status.is_success() {
        return Err(Error::Auth(format!(
            "OAuth1 preauthorized failed ({status}): {}",
            &body[..body.len().min(200)]
        )));
    }

    let params: Vec<(String, String)> = url::form_urlencoded::parse(body.as_bytes())
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let token = params
        .iter()
        .find(|(k, _)| k == "oauth_token")
        .map(|(_, v)| v.clone())
        .ok_or_else(|| Error::Auth("OAuth1 token not in response".into()))?;
    let token_secret = params
        .iter()
        .find(|(k, _)| k == "oauth_token_secret")
        .map(|(_, v)| v.clone())
        .ok_or_else(|| Error::Auth("OAuth1 token secret not in response".into()))?;

    Ok(OAuth1Token {
        token,
        token_secret,
    })
}

async fn exchange_oauth1_for_oauth2(
    oauth1: &OAuth1Token,
    consumer: &ConsumerCredentials,
    is_login: bool,
) -> Result<OAuth2Token> {
    let client = reqwest::Client::builder()
        .user_agent(CONNECT_USER_AGENT)
        .build()?;

    let url = format!("{CONNECT_API}/oauth-service/oauth/exchange/user/2.0");

    let body_params: Vec<(&str, &str)> = if is_login {
        vec![("audience", "GARMIN_CONNECT_MOBILE_ANDROID_DI")]
    } else {
        vec![]
    };

    let auth_header = oauth1_header(
        "POST",
        &url,
        &body_params,
        &consumer.consumer_key,
        &consumer.consumer_secret,
        Some(&oauth1.token),
        Some(&oauth1.token_secret),
    );

    let content_type = "application/x-www-form-urlencoded";
    let mut req = client
        .post(&url)
        .header("Authorization", auth_header)
        .header("Content-Type", content_type);

    if is_login {
        req = req.form(&[("audience", "GARMIN_CONNECT_MOBILE_ANDROID_DI")]);
    }

    let resp = req.send().await?;

    let status = resp.status();
    let body = resp.text().await?;

    if !status.is_success() {
        return Err(Error::Auth(format!(
            "OAuth2 exchange failed ({status}): {body}"
        )));
    }

    #[derive(Deserialize)]
    struct OAuth2Response {
        access_token: String,
        token_type: String,
        refresh_token: String,
        expires_in: i64,
    }

    let resp: OAuth2Response = serde_json::from_str(&body)?;
    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + resp.expires_in;

    Ok(OAuth2Token {
        access_token: resp.access_token,
        token_type: resp.token_type,
        refresh_token: resp.refresh_token,
        expires_at,
    })
}

// --- OAuth1 signing (RFC 5849) ---

fn oauth1_header(
    method: &str,
    base_url: &str,
    params: &[(&str, &str)],
    consumer_key: &str,
    consumer_secret: &str,
    token: Option<&str>,
    token_secret: Option<&str>,
) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    let nonce: String = {
        let bytes: [u8; 16] = rand::rng().random();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect()
    };

    build_oauth1_header(
        method,
        base_url,
        params,
        consumer_key,
        consumer_secret,
        token,
        token_secret,
        &timestamp,
        &nonce,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_oauth1_header(
    method: &str,
    base_url: &str,
    params: &[(&str, &str)],
    consumer_key: &str,
    consumer_secret: &str,
    token: Option<&str>,
    token_secret: Option<&str>,
    timestamp: &str,
    nonce: &str,
) -> String {
    let mut oauth_params: BTreeMap<String, String> = BTreeMap::new();
    oauth_params.insert("oauth_consumer_key".into(), consumer_key.into());
    oauth_params.insert("oauth_nonce".into(), nonce.to_string());
    oauth_params.insert("oauth_signature_method".into(), "HMAC-SHA1".into());
    oauth_params.insert("oauth_timestamp".into(), timestamp.to_string());
    oauth_params.insert("oauth_version".into(), "1.0".into());
    if let Some(t) = token {
        oauth_params.insert("oauth_token".into(), t.into());
    }

    let mut all_params = oauth_params.clone();
    for (k, v) in params {
        all_params.insert(k.to_string(), v.to_string());
    }

    let param_string: String = all_params
        .iter()
        .map(|(k, v)| format!("{k}={v}", k = percent_encode(k), v = percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let base_string = format!(
        "{method}&{url}&{params}",
        method = method.to_uppercase(),
        url = percent_encode(base_url),
        params = percent_encode(&param_string)
    );

    let signing_key = format!(
        "{cs}&{ts}",
        cs = percent_encode(consumer_secret),
        ts = percent_encode(token_secret.unwrap_or(""))
    );

    let mut mac = Hmac::<Sha1>::new_from_slice(signing_key.as_bytes()).unwrap();
    hmac::Mac::update(&mut mac, base_string.as_bytes());
    let signature = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        mac.finalize().into_bytes(),
    );

    oauth_params.insert("oauth_signature".into(), signature);

    let header = oauth_params
        .iter()
        .map(|(k, v)| {
            let encoded = percent_encode(v);
            format!("{k}=\"{encoded}\"")
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!("OAuth {header}")
}

/// RFC 5849 percent encoding: encode everything except unreserved chars (A-Z a-z 0-9 - . _ ~)
const OAUTH_ENCODE_SET: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

fn percent_encode(s: &str) -> String {
    percent_encoding::utf8_percent_encode(s, OAUTH_ENCODE_SET).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth1_signature() {
        let header = build_oauth1_header(
            "GET",
            "https://connectapi.garmin.com/oauth-service/oauth/preauthorized",
            &[
                ("ticket", "ST-123456-test"),
                ("login-url", "https://sso.garmin.com/sso/embed"),
                ("accepts-mfa-tokens", "true"),
            ],
            "fc3e99d2-118c-44b8-8ae3-03370dde24c0",
            "E08WAR897WEy2knn7aFBrvegVAf0AFdWBBF",
            None,
            None,
            "1234567890",
            "testnonce123",
        );
        // Python reference: G5Ts96qQYZNkQv7x7j4gyEFS/oo=
        assert!(
            header.contains("G5Ts96qQYZNkQv7x7j4gyEFS%2Foo%3D"),
            "Signature mismatch in header: {header}"
        );
    }
}
