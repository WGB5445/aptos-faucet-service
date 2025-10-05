use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct GoogleProfile {
    pub email: String,
    pub subject: String,
    pub domain: Option<String>,
    pub name: Option<String>,
}

#[derive(Clone)]
pub struct GoogleVerifier {
    client: Client,
    client_id: String,
}

impl GoogleVerifier {
    pub fn new(client_id: &str) -> Result<Self> {
        let client = Client::builder().build()?;
        Ok(Self {
            client,
            client_id: client_id.to_string(),
        })
    }

    pub async fn verify(&self, id_token: &str) -> Result<GoogleProfile> {
        let response = self
            .client
            .get("https://oauth2.googleapis.com/tokeninfo")
            .query(&[("id_token", id_token)])
            .send()
            .await
            .context("failed to call Google tokeninfo endpoint")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "google token verification failed: status {}",
                response.status()
            );
        }

        let payload: TokenInfo = response
            .json()
            .await
            .context("failed to decode google tokeninfo response")?;

        if payload.aud != self.client_id {
            anyhow::bail!("google token targeted different client id");
        }

        let verified = matches!(payload.email_verified.as_deref(), Some("true"));
        if !verified {
            anyhow::bail!("google account email not verified");
        }

        let email = payload
            .email
            .context("google tokeninfo response missing email")?;
        let subject = payload
            .sub
            .context("google tokeninfo response missing subject")?;
        let domain = payload
            .hd
            .or_else(|| email.split('@').nth(1).map(|s| s.to_string()))
            .map(|d| d.to_ascii_lowercase());

        Ok(GoogleProfile {
            email,
            subject,
            domain,
            name: payload.name,
        })
    }
}

#[derive(Debug, Deserialize)]
struct TokenInfo {
    aud: String,
    email: Option<String>,
    email_verified: Option<String>,
    sub: Option<String>,
    hd: Option<String>,
    name: Option<String>,
}
