use reqwest::{Client, RequestBuilder};
use serde::Serialize;

#[derive(Clone, Debug)]
pub enum Auth {
    Basic(String, String), // Username, Password
    Bearer(String),        // Token
    None,
}

#[derive(Clone, Debug)]
pub struct ApiFetch {
    client: Client,
    base_url: String,
    auth: Auth,
}

impl ApiFetch {
    pub fn new(base_url: impl Into<String>, auth: Auth) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            auth,
        }
    }

    pub fn with_client(client: Client, base_url: impl Into<String>, auth: Auth) -> Self {
        Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            auth,
        }
    }

    fn build_url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        format!("{}/{}", self.base_url, path)
    }

    fn apply_auth(&self, builder: RequestBuilder) -> RequestBuilder {
        match &self.auth {
            Auth::Basic(user, pass) => builder.basic_auth(user, Some(pass)),
            Auth::Bearer(token) => builder.bearer_auth(token),
            Auth::None => builder,
        }
    }

    pub async fn get(&self, path: &str) -> reqwest::Result<reqwest::Response> {
        let url = self.build_url(path);
        let builder = self.client.get(&url);
        self.apply_auth(builder).send().await
    }

    pub async fn post<T: Serialize + ?Sized>(&self, path: &str, body: &T) -> reqwest::Result<reqwest::Response> {
        let url = self.build_url(path);
        let builder = self.client.post(&url).json(body);
        self.apply_auth(builder).send().await
    }

    pub async fn post_raw(&self, path: &str, body: Vec<u8>, content_type: &str, content_disposition: &str) -> reqwest::Result<reqwest::Response> {
        let url = self.build_url(path);
        let builder = self.client.post(&url)
            .header(reqwest::header::CONTENT_TYPE, content_type)
            .header("Content-Disposition", content_disposition)
            .body(body);
        self.apply_auth(builder).send().await
    }

    pub async fn put<T: Serialize + ?Sized>(&self, path: &str, body: &T) -> reqwest::Result<reqwest::Response> {
        let url = self.build_url(path);
        let builder = self.client.put(&url).json(body);
        self.apply_auth(builder).send().await
    }

    pub async fn delete(&self, path: &str) -> reqwest::Result<reqwest::Response> {
        let url = self.build_url(path);
        let builder = self.client.delete(&url);
        self.apply_auth(builder).send().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url() {
        let api = ApiFetch::new("https://example.com/wp-json", Auth::None);
        assert_eq!(api.build_url("/wp/v2/posts"), "https://example.com/wp-json/wp/v2/posts");
        assert_eq!(api.build_url("wp/v2/posts"), "https://example.com/wp-json/wp/v2/posts");
        
        let api2 = ApiFetch::new("https://example.com/wp-json/", Auth::None);
        assert_eq!(api2.build_url("/wp/v2/posts"), "https://example.com/wp-json/wp/v2/posts");
    }

    #[test]
    fn test_apply_auth_bearer() {
        let api = ApiFetch::new("https://example.com", Auth::Bearer("token123".to_string()));
        let builder = api.client.get("https://example.com");
        let builder = api.apply_auth(builder);
        let req = builder.build().unwrap();
        
        assert_eq!(
            req.headers().get(reqwest::header::AUTHORIZATION).unwrap().to_str().unwrap(),
            "Bearer token123"
        );
    }
    
    #[test]
    fn test_apply_auth_basic() {
        let api = ApiFetch::new("https://example.com", Auth::Basic("user".to_string(), "pass".to_string()));
        let builder = api.client.get("https://example.com");
        let builder = api.apply_auth(builder);
        let req = builder.build().unwrap();
        
        assert_eq!(
            req.headers().get(reqwest::header::AUTHORIZATION).unwrap().to_str().unwrap(),
            "Basic dXNlcjpwYXNz"
        );
    }
}
