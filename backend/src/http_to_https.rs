//! HTTP server which redirects to HTTPS.
//!
//! Example usage:
//!
//! ```rust
//! http_to_https::Config::new()
//!     .set_http_port(8080)
//!     .set_https_port(4443)
//!     .translate_urls(|_| "/".to_string())
//!     .serve()
//! ```

/// Configuration for the HTTP server which redirects to HTTPS
#[derive(Debug, Clone)]
pub struct Config<F, G> {
    http_port: u16,
    https_port: u16,
    url_translator: F,
    host_translator: G,
    hsts: bool,
}

impl Config<fn(&str) -> String, fn(&str) -> String> {
    /// Constructs the default configuration
    pub fn new() -> Self {
        fn default_url_translator(_: &str) -> String {
            "/".to_string()
        }
        fn default_host_translator(s: &str) -> String {
            s.to_string()
        }
        Config {
            http_port: 80,
            https_port: 443,
            url_translator: default_url_translator,
            host_translator: default_host_translator,
            hsts: false,
        }
    }
}

impl<F, G> Config<F, G> {
    /// Set the HTTP port to run the server on.
    /// The default is 80.
    pub fn set_http_port(mut self, new_port: u16) -> Self {
        self.http_port = new_port;
        self
    }

    /// Set the port to which the new HTTPS requests are forwarded.
    /// The default is 443.
    pub fn set_https_port(mut self, new_port: u16) -> Self {
        self.https_port = new_port;
        self
    }

    /// Set the function used to translate URLs.
    /// The default function always returns "/"
    pub fn translate_urls<H: Fn(&str) -> String>(self, func: H) -> Config<H, G> {
        Config {
            http_port: self.http_port,
            https_port: self.https_port,
            url_translator: func,
            host_translator: self.host_translator,
            hsts: self.hsts,
        }
    }

    /// Set the function used to translate the hostname.
    /// The default function returns the same hostname, unmodified
    pub fn translate_hosts<H: Fn(&str) -> String>(self, func: H) -> Config<F, H> {
        Config {
            http_port: self.http_port,
            https_port: self.https_port,
            url_translator: self.url_translator,
            host_translator: func,
            hsts: self.hsts,
        }
    }

    /// Whether to send an HSTS header disallowing http communication in the future.
    ///
    /// Default: `false`
    pub fn hsts(mut self, hsts: bool) -> Self {
        self.hsts = hsts;
        self
    }
}

impl<F, G> Config<F, G>
where
    F: Fn(&str) -> String,
    G: Fn(&str) -> String,
{
    pub fn serve(&self) {
        let server =
            tiny_http::Server::http(("0.0.0.0", self.http_port)).expect("Could not open server");
        for req in server.incoming_requests() {
            let orig_host = match req.headers().iter().find(|h| h.field.equiv("host")) {
                Some(h) => h.value.as_str().split(':').next().unwrap(),
                None => continue,
            };
            let host = (self.host_translator)(orig_host);
            let new_url = (self.url_translator)(req.url());
            let location = if self.https_port == 443 {
                format!("https://{}{}", host, new_url)
            } else {
                format!("https://{}:{}{}", host, self.https_port, new_url)
            };
            let mut response = tiny_http::Response::empty(tiny_http::StatusCode(301))
                .with_header(tiny_http::Header::from_bytes("Location", location).unwrap())
                .with_header(tiny_http::Header::from_bytes("Connection", "close").unwrap());
            if self.hsts {
                response.add_header(
                    tiny_http::Header::from_bytes("Strict-Transport-Security", "max-age=2592000")
                        .unwrap(),
                );
            }
            let _ = req.respond(response);
        }
    }
}
