use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use faucet_core::models::{Channel, Role};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // user_id
    pub handle: String,     // user handle (email)
    pub channel: String,    // channel type
    pub domain: Option<String>, // user domain
    pub role: String,       // user role
    pub exp: i64,          // expiration time
    pub iat: i64,          // issued at
}

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtService {
    pub fn new(secret: &str) -> Result<Self> {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        
        Ok(Self {
            encoding_key,
            decoding_key,
            validation,
        })
    }

    pub fn generate_token(
        &self,
        user_id: Uuid,
        handle: &str,
        channel: &Channel,
        domain: Option<&str>,
        role: &Role,
        expiry_hours: i64,
    ) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(expiry_hours);
        
        let claims = Claims {
            sub: user_id.to_string(),
            handle: handle.to_string(),
            channel: channel.as_str().to_string(),
            domain: domain.map(|s| s.to_string()),
            role: role.as_str().to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .context("failed to encode JWT token")
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .context("failed to decode JWT token")?;
        
        // 检查token是否过期
        let now = Utc::now().timestamp();
        if token_data.claims.exp < now {
            anyhow::bail!("token has expired");
        }
        
        Ok(token_data.claims)
    }

    pub fn is_token_expired(&self, token: &str) -> bool {
        match self.verify_token(token) {
            Ok(_) => false,
            Err(_) => true,
        }
    }
}

// Role的as_str方法已经在core模块中定义，这里不需要重复定义
// 我们只需要在main.rs中处理FromStr转换

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use faucet_core::models::{Channel, Role};

    #[test]
    fn test_jwt_token_generation_and_verification() {
        let jwt_service = JwtService::new("test-secret").unwrap();
        let user_id = Uuid::new_v4();
        let handle = "test@example.com";
        let channel = Channel::Web;
        let domain = Some("example.com");
        let role = Role::User;
        let expiry_hours = 24;

        // 生成token
        let token = jwt_service.generate_token(
            user_id,
            handle,
            &channel,
            domain.as_deref(),
            &role,
            expiry_hours,
        ).unwrap();

        // 验证token
        let claims = jwt_service.verify_token(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.handle, handle);
        assert_eq!(claims.channel, "web");
        assert_eq!(claims.domain, domain.map(|s| s.to_string()));
        assert_eq!(claims.role, "user");
    }

    #[test]
    fn test_jwt_token_expiry() {
        let jwt_service = JwtService::new("test-secret").unwrap();
        let user_id = Uuid::new_v4();
        let handle = "test@example.com";
        let channel = Channel::Web;
        let role = Role::User;

        // 生成一个立即过期的token
        let token = jwt_service.generate_token(
            user_id,
            handle,
            &channel,
            None,
            &role,
            0, // 0小时，立即过期
        ).unwrap();

        // 验证token应该失败（已过期）
        let result = jwt_service.verify_token(&token);
        assert!(result.is_err());
    }
}
