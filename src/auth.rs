use crate::config;
use crate::error::{Error, Result};
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

const SSO_BASE: &str = "https://sso.garmin.com";
const CONNECT_BASE: &str = "https://connect.garmin.com";
const OAUTH_CONSUMER_URL: &str = "https://thegarth.s3.amazonaws.com/oauth_consumer.json";

const SSO_USER_AGENT: &str = "GCM-iOS-5.19.1.2";
const CONNECT_USER_AGENT: &str = "com.garmin.android.apps.connectmobile";

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
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(config::tokens_path(), data)?;
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
    let oauth2 = exchange_oauth1_for_oauth2(&oauth1, &consumer).await?;

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
    tokens.oauth2 = exchange_oauth1_for_oauth2(&tokens.oauth1, &tokens.consumer).await?;
    tokens.save()?;
    Ok(())
}

// --- SSO flow ---

async fn sso_login(email: &str, password: &str) -> Result<String> {
    let jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
    let client = reqwest::Client::builder()
        .cookie_provider(jar)
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(SSO_USER_AGENT)
        .build()?;

    let qs = "id=gauth-widget&embedWidget=true&gauthHost=https%3A%2F%2Fsso.garmin.com";
    let embed_url = format!("{SSO_BASE}/sso/embed?{qs}");

    // GET /sso/embed - sets cookies
    client.get(&embed_url).send().await?;

    // GET /sso/signin - get CSRF token
    let signin_url = format!("{SSO_BASE}/sso/signin?{qs}");
    let signin_page = client.get(&signin_url).send().await?.text().await?;

    let csrf = extract_csrf(&signin_page)?;

    // POST /sso/signin - submit credentials
    let form: &[(&str, &str)] = &[
        ("username", email),
        ("password", password),
        ("_csrf", &csrf),
        ("embed", "true"),
    ];

    let resp = client.post(&signin_url).form(form).send().await?;

    let body = resp.text().await?;

    if body.contains("verifyMFA") || body.contains("MFA") {
        return Err(Error::MfaRequired);
    }

    extract_ticket(&body)
}

fn extract_csrf(html: &str) -> Result<String> {
    let re = regex::Regex::new(r#"name="_csrf"\s+value="([^"]+)""#).unwrap();
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| Error::Auth("CSRF token not found in SSO page".into()))
}

fn extract_ticket(html: &str) -> Result<String> {
    let re = regex::Regex::new(r#"embed\?ticket=([^"]+)"#).unwrap();
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| Error::Auth("SSO ticket not found - check credentials".into()))
}

// --- OAuth flow ---

async fn fetch_consumer_credentials() -> Result<ConsumerCredentials> {
    let resp: serde_json::Value = reqwest::get(OAUTH_CONSUMER_URL).await?.json().await?;
    let key = resp["consumer_key"]
        .as_str()
        .ok_or_else(|| Error::Auth("Missing consumer key".into()))?
        .to_string();
    let secret = resp["consumer_secret"]
        .as_str()
        .ok_or_else(|| Error::Auth("Missing consumer secret".into()))?
        .to_string();
    Ok(ConsumerCredentials {
        consumer_key: key,
        consumer_secret: secret,
    })
}

async fn exchange_ticket_for_oauth1(
    ticket: &str,
    consumer: &ConsumerCredentials,
) -> Result<OAuth1Token> {
    let client = reqwest::Client::builder()
        .user_agent(CONNECT_USER_AGENT)
        .build()?;

    let base_path = format!("{CONNECT_BASE}/oauth-service/oauth/preauthorized");
    let login_url = format!("{SSO_BASE}/sso/embed");
    let encoded_ticket = urlencoding::encode(ticket);
    let url =
        format!("{base_path}?ticket={encoded_ticket}&login-url={login_url}&acceptTheTerms=true");

    let auth_header = oauth1_header(
        "GET",
        &base_path,
        &[
            ("ticket", ticket),
            ("login-url", &login_url),
            ("acceptTheTerms", "true"),
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

    let body = resp.text().await?;
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
) -> Result<OAuth2Token> {
    let client = reqwest::Client::builder()
        .user_agent(CONNECT_USER_AGENT)
        .build()?;

    let url = format!("{CONNECT_BASE}/oauth-service/oauth/exchange/user/2.0");

    let auth_header = oauth1_header(
        "POST",
        &url,
        &[],
        &consumer.consumer_key,
        &consumer.consumer_secret,
        Some(&oauth1.token),
        Some(&oauth1.token_secret),
    );

    let content_type = "application/x-www-form-urlencoded";
    let resp = client
        .post(&url)
        .header("Authorization", auth_header)
        .header("Content-Type", content_type)
        .send()
        .await?;

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

    let mut oauth_params: BTreeMap<String, String> = BTreeMap::new();
    oauth_params.insert("oauth_consumer_key".into(), consumer_key.into());
    oauth_params.insert("oauth_nonce".into(), nonce);
    oauth_params.insert("oauth_signature_method".into(), "HMAC-SHA1".into());
    oauth_params.insert("oauth_timestamp".into(), timestamp);
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

fn percent_encode(s: &str) -> String {
    percent_encoding::utf8_percent_encode(s, percent_encoding::NON_ALPHANUMERIC).to_string()
}
