use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::json;
use once_cell::sync::Lazy;
use redis::aio::ConnectionManager;

// Shared test context
struct TestContext {
    client: reqwest::Client,
    base_url: String,
}

static REDIS_CLIENT: Lazy<redis::Client> = Lazy::new(|| {
    redis::Client::open("redis://127.0.0.1:6380/").unwrap()
});

impl TestContext {
    fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .cookie_store(true)
                .build()
                .unwrap(),
            base_url: "http://127.0.0.1:3000".to_string(),
        }
    }

    fn get_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

async fn get_redis_conn() -> ConnectionManager {
    REDIS_CLIENT.get_connection_manager().await.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    async fn setup() {
        let mut con = get_redis_conn().await;
        let _: () = redis::cmd("DEL").arg("rate_limit:register:127.0.0.1").query_async(&mut con).await.unwrap();
    }

    #[tokio::test]
    async fn test_user_registration_and_login() {
        setup().await;
        let context = TestContext::new();
        let timestamp = TestContext::get_timestamp();
        let username = format!("testuser_{}", timestamp);

        // Step 1: User Registration
        let reg_response = context.client.post(format!("{}/api/auth/register", context.base_url))
            .json(&json!({
                "name": "Test User",
                "username": username,
                "password": "SecurePass123!@#"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(reg_response.status().as_u16(), 201, "Registration failed");
        let reg_body: Value = reg_response.json().await.unwrap();
        assert_eq!(reg_body["message"], "Registration successful. Welcome!");

        // Step 3: User Login
        let login_response = context.client.post(format!("{}/api/auth/login", context.base_url))
            .json(&json!({
                "username": username,
                "password": "SecurePass123!@#"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(login_response.status().as_u16(), 200, "Login failed");

        let cookies = login_response.cookies().collect::<Vec<_>>();
        let csrf_cookie = cookies.iter().find(|c| c.name() == "csrf_token").expect("CSRF token not found in login response");
        let csrf_token = csrf_cookie.value().to_string();

        // Step 4: Query User Storage Info
        let storage_response = context.client.get(format!("{}/api/files/storage/info", context.base_url))
            .header("X-CSRF-Token", csrf_token)
            .send()
            .await
            .unwrap();

        assert_eq!(storage_response.status().as_u16(), 200, "Failed to get storage info");

        let storage_body: Value = storage_response.json().await.unwrap();
        assert_eq!(storage_body["storage_used_bytes"], 0);
        assert_eq!(storage_body["storage_quota_bytes"], 1073741824i64);
    }
}
