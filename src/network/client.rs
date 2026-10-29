use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Client;
use std::time::Duration;

pub struct StealthHttpClient {
    client: Client,
    user_agent: String,
}

impl StealthHttpClient {
    pub fn new(custom_ua: Option<&str>) -> Result<Self, reqwest::Error> {
        Self::new_for_platform(custom_ua, std::env::consts::OS)
    }

    /// Monta o cliente furtivo com client hints coerentes com o SO real do host.
    /// `platform` segue `std::env::consts::OS` ("windows", "linux", "macos").
    pub fn new_for_platform(
        custom_ua: Option<&str>,
        platform: &str,
    ) -> Result<Self, reqwest::Error> {
        let ua = custom_ua.unwrap_or(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
        );

        // UA inválido é descartado: o reqwest aplica o User-Agent padrão em vez de falhar de forma silenciosa.
        let ua_value = HeaderValue::from_str(ua).unwrap_or_else(|_| {
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
            )
        });

        // Client hints coerentes com o SO real: um VPS Linux enviando "Windows" é fingerprint de bot.
        let ch_platform = match platform {
            "windows" => "\"Windows\"",
            "macos" => "\"macOS\"",
            _ => "\"Linux\"",
        };

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, ua_value);
        headers.insert(
            "sec-ch-ua",
            HeaderValue::from_static(
                "\"Chromium\";v=\"128\", \"Not;A=Brand\";v=\"24\", \"Google Chrome\";v=\"128\"",
            ),
        );
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        headers.insert("sec-ch-ua-platform", HeaderValue::from_static(ch_platform));

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(15))
            .build()?;

        Ok(Self {
            client,
            user_agent: ua.to_string(),
        })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }
}
