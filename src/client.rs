use std::error::Error;

use reqwest::header::{HeaderMap, HeaderValue};

use super::executor::RawExecResponse;
use super::ExecResponse;
use super::ExecResult;
use super::Executor;
use super::Runtime;

/// A client used to send requests to Piston.
#[derive(Debug, Clone)]
pub struct Client {
    /// The base url for Piston.
    url: String,
    /// The reqwest client to use.
    client: reqwest::Client,
    /// The headers to send with each request.
    headers: HeaderMap,
}

impl Default for Client {
    /// Creates a new client. Alias for [`Client::new`].
    ///
    /// # Returns
    /// - [`Client`] - The new Client.
    ///
    /// # Example
    /// ```
    /// let client = piston_rs::Client::default();
    ///
    /// assert!(client.get_headers().contains_key("Accept"));
    /// assert!(client.get_headers().contains_key("User-Agent"));
    /// assert!(!client.get_headers().contains_key("Authorization"));
    /// assert_eq!(client.get_url(), "https://emkc.org/api/v2/piston".to_string());
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    /// Creates a new client.
    ///
    /// # Returns
    /// - [`Client`] - The new Client.
    ///
    /// # Example
    /// ```
    /// let client = piston_rs::Client::new();
    ///
    /// assert!(client.get_headers().contains_key("Accept"));
    /// assert!(client.get_headers().contains_key("User-Agent"));
    /// assert!(!client.get_headers().contains_key("Authorization"));
    /// ```
    pub fn new() -> Self {
        Self {
            url: "https://emkc.org/api/v2/piston".to_string(),
            client: reqwest::Client::new(),
            headers: Self::generate_headers(None),
        }
    }

    /// Creates a new Client with a url that runs the piston code execution engine.
    ///
    /// This makes it possible to interact with a self-hosted instance of piston.
    ///
    /// # Arguments
    /// - `url` - The url to use as the underlying piston backend.
    ///
    /// # Returns
    /// - [`Client`] - The new Client.
    ///
    /// # Example
    /// ```
    /// let client = piston_rs::Client::with_url("http://localhost:3000");
    /// assert_eq!(client.get_url(), "http://localhost:3000");
    /// ```
    pub fn with_url(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: reqwest::Client::new(),
            headers: Self::generate_headers(None),
        }
    }

    /// Creates a new client, with an api key.
    ///
    /// # Arguments
    /// - `key` - The api key to use.
    ///
    /// # Returns
    /// - [`Client`] - The new Client.
    ///
    /// # Example
    /// ```
    /// let client = piston_rs::Client::with_key("123abc");
    ///
    /// assert!(client.get_headers().contains_key("Authorization"));
    /// assert_eq!(client.get_headers().get("Authorization").unwrap(), "123abc");
    /// ```
    pub fn with_key(key: &str) -> Self {
        Self {
            url: "https://emkc.org/api/v2/piston".to_string(),
            client: reqwest::Client::new(),
            headers: Self::generate_headers(Some(key)),
        }
    }

    /// Creates a new Client using a url and an api key.
    ///
    /// # Arguments
    /// - `url` - The url to use as the underlying piston backend.
    /// - `key` - The api key to use.
    ///
    /// # Returns
    /// - [`Client`] - The new Client.
    ///
    /// # Example
    /// ```
    /// let client = piston_rs::Client::with_url_and_key("http://localhost:3000", "123abc");
    /// assert_eq!(client.get_url(), "http://localhost:3000");
    /// assert!(client.get_headers().contains_key("Authorization"));
    /// assert_eq!(client.get_headers().get("Authorization").unwrap(), "123abc");
    /// ```
    pub fn with_url_and_key(url: &str, key: &str) -> Self {
        Self {
            url: url.to_string(),
            client: reqwest::Client::new(),
            headers: Self::generate_headers(Some(key)),
        }
    }

    /// The base url for the Piston V2 API that is being used by this client.
    ///
    /// # Returns
    ///
    /// - [`String`] - The requested url.
    ///
    /// # Example
    /// ```
    /// let client = piston_rs::Client::new();
    ///
    /// assert_eq!(client.get_url(), "https://emkc.org/api/v2/piston".to_string());
    /// ```
    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    /// The headers being used by this client.
    ///
    /// # Returns
    ///
    /// - [`HeaderMap`] - A map of Header key, value pairs.
    ///
    /// # Example
    /// ```
    /// let client = piston_rs::Client::new();
    /// let headers = client.get_headers();
    ///
    /// assert_eq!(headers.get("Accept").unwrap(), "application/json");
    /// ```
    pub fn get_headers(&self) -> HeaderMap {
        self.headers.clone()
    }

    /// Generates the headers the client should use.
    ///
    /// # Returns
    ///
    /// - [`HeaderMap`] - A map of Header key, value pairs.
    ///
    /// # Example
    /// ```ignore # Fails to compile (private function)
    /// let headers = piston_rs::Client::generate_headers(None);
    ///
    /// assert!(!headers.contains_key("Authorization"));
    /// assert_eq!(headers.get("Accept").unwrap(), "application/json");
    /// assert_eq!(headers.get("User-Agent").unwrap(), "piston-rs");
    ///
    /// let headers = piston_rs::Client::generate_headers(Some("123abc"));
    ///
    /// assert_eq!(headers.get("Authorization").unwrap(), "123abc");
    /// assert_eq!(headers.get("Accept").unwrap(), "application/json");
    /// assert_eq!(headers.get("User-Agent").unwrap(), "piston-rs");
    /// ```
    fn generate_headers(key: Option<&str>) -> HeaderMap {
        let mut headers = HeaderMap::with_capacity(3);
        headers.insert("Accept", HeaderValue::from_str("application/json").unwrap());
        headers.insert("User-Agent", HeaderValue::from_str("piston-rs").unwrap());

        if let Some(k) = key {
            headers.insert("Authorization", HeaderValue::from_str(k).unwrap());
        };

        headers
    }

    /// Fetches the runtimes from Piston. **This is an http request**.
    ///
    /// # Returns
    /// - [`Result<Vec<Runtime>, Box<dyn Error>>`] - The available
    /// runtimes or the error, if any.
    ///
    /// # Example
    /// ```no_run
    /// # #[tokio::test]
    /// # async fn test_fetch_runtimes() {
    /// let client = piston_rs::Client::new();
    ///
    /// if let Ok(runtimes) = client.fetch_runtimes().await {
    ///     assert!(!runtimes.is_empty());
    /// } else {
    ///     // There was an error contacting Piston.
    /// }
    /// # }
    /// ```
    pub async fn fetch_runtimes(&self) -> Result<Vec<Runtime>, Box<dyn Error>> {
        let endpoint = format!("{}/runtimes", self.url);
        let runtimes = self
            .client
            .get(endpoint)
            .headers(self.headers.clone())
            .send()
            .await?
            .json::<Vec<Runtime>>()
            .await?;

        Ok(runtimes)
    }

    /// Executes code using a given executor. **This is an http
    /// request**.
    ///
    /// # Arguments
    /// - `executor` - The executor to use.
    ///
    /// # Returns
    /// - [`Result<ExecutorResponse, Box<dyn Error>>`] - The response
    /// from Piston or the error, if any.
    ///
    /// # Example
    /// ```no_run
    /// # #[tokio::test]
    /// # async fn test_execute() {
    /// let client = piston_rs::Client::new();
    /// let executor = piston_rs::Executor::new()
    ///     .set_language("rust")
    ///     .set_version("1.50.0")
    ///     .add_file(piston_rs::File::default().set_content(
    ///         "fn main() { println!(\"42\"); }",
    ///     ));
    ///
    /// if let Ok(response) = client.execute(&executor).await {
    ///     assert!(response.compile.is_some());
    ///     assert!(response.run.is_ok());
    ///     assert!(response.is_ok());
    /// } else {
    ///     // There was an error contacting Piston.
    /// }
    /// # }
    /// ```
    pub async fn execute(&self, executor: &Executor) -> Result<ExecResponse, Box<dyn Error>> {
        let endpoint = format!("{}/execute", self.url);

        match self
            .client
            .post(endpoint)
            .headers(self.headers.clone())
            .json::<Executor>(executor)
            .send()
            .await
        {
            Ok(data) => {
                let status = data.status();

                match status {
                    reqwest::StatusCode::OK => {
                        let response = data.json::<RawExecResponse>().await?;

                        Ok(ExecResponse {
                            language: response.language,
                            version: response.version,
                            run: response.run,
                            compile: response.compile,
                            status: status.as_u16(),
                        })
                    }
                    _ => {
                        let text = format!("{}: {}", data.status(), data.text().await?);

                        let exec_result = ExecResult {
                            stdout: String::new(),
                            stderr: text.clone(),
                            output: text,
                            code: 1,
                            signal: None,
                        };

                        let exec_response = ExecResponse {
                            language: executor.language.clone(),
                            version: executor.version.clone(),
                            run: exec_result,
                            compile: None,
                            status: status.as_u16(),
                        };

                        Ok(exec_response)
                    }
                }
            }
            Err(e) => Err(Box::new(e)),
        }
    }
}

#[cfg(test)]
mod test_client_private {
    use super::Client;

    #[test]
    fn test_gen_headers_no_key() {
        let headers = Client::generate_headers(None);

        assert!(!headers.contains_key("Authorization"));
        assert_eq!(headers.get("Accept").unwrap(), "application/json");
        assert_eq!(headers.get("User-Agent").unwrap(), "piston-rs");
    }

    #[test]
    fn test_gen_headers_with_key() {
        let headers = Client::generate_headers(Some("123abc"));

        assert_eq!(headers.get("Authorization").unwrap(), "123abc");
        assert_eq!(headers.get("Accept").unwrap(), "application/json");
        assert_eq!(headers.get("User-Agent").unwrap(), "piston-rs");
    }
}
