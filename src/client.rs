use crate::utils::{read_file_async, read_file_lines_sync};
use hyper::body::{Bytes, HttpBody};
use hyper::{Body, Client, Response};
use std::env;
use std::process::exit;
use tokio::fs::File;
use tokio::io::{stdout, AsyncReadExt, AsyncWriteExt};
use tokio::task::JoinHandle;

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum LogStatus {
    Full,
    Normal,
    Silent,
}

#[derive(Debug)]
pub(crate) struct App {
    log: LogStatus,
    method: Option<String>,
    headers: Option<Vec<String>>,
    output_file: Option<String>,
    ua: Option<String>,
    data: Option<Vec<u8>>,
    upload_file: Option<String>,
    include_headers: bool,
    auth: Option<String>,
    urls: Vec<String>,
    recursive: bool,
    remote_name: bool,
    follow_redirects: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            log: LogStatus::Normal,
            method: None,
            headers: None,
            output_file: None,
            ua: None,
            data: None,
            upload_file: None,
            include_headers: false,
            auth: None,
            urls: vec![],
            recursive: false,
            remote_name: false,
            follow_redirects: false,
        }
    }
}

impl App {
    pub(crate) async fn new(options: &getopts::Options) -> Option<Self> {
        let args = env::args().collect::<Vec<String>>();
        let mut app_config = App::default();
        let config = options.parse(&args[1..]).ok().or_else(|| {
            app_config.error(options.short_usage(&args[0]));
            None
        })?;
        if config.opt_present("V") {
            app_config.error(format!("currrrl version: {}", env!("CARGO_PKG_VERSION")));
            None?;
        }
        if config.opt_present("h") {
            app_config.error(options.usage("currrrl is the best CURL alternative"));
            None?;
        }
        if config.free.is_empty() {
            app_config.error_str("currrrl: no URL specified!");
            None?;
        }

        if config.opt_present("s") {
            app_config.log = LogStatus::Silent;
        } else if config.opt_present("v") {
            app_config.log = LogStatus::Full;
        }
        if config.opt_present("i") {
            app_config.include_headers = true;
        }
        if config.opt_present("recursive") {
            app_config.recursive = true;
            app_config.error_str("Warning: not implemented yet");
            None?;
        }
        if config.opt_present("L") {
            app_config.follow_redirects = true;
        }
        if config.opt_present("O") {
            app_config.remote_name = true;
            app_config.error_str("Warning: not implemented yet");
            None?;
        }
        if let Some(x) = config.opt_str("X") {
            app_config.method = Some(x);
        }
        if let Some(x) = config.opt_str("o") {
            app_config.output_file = Some(x)
        }
        if let Some(x) = config.opt_str("u") {
            app_config.auth = Some(x)
        }
        if let Some(x) = config.opt_str("A") {
            app_config.ua = Some(x)
        }
        if let Some(x) = config.opt_str("T") {
            app_config.upload_file = Some(x)
        }
        if let Some(x) = config.opt_str("d") {
            if x.starts_with('@') {
                app_config.data = read_file_async(&x)
                    .await
                    .map_err(|e| {
                        app_config.error(format!("Warning: can't read data from file: {}", e));
                        exit(-1);
                    })
                    .ok();
            } else {
                app_config.data = Some(x.into());
            }
        }
        let headers = config.opt_strs("H");
        if !headers.is_empty() {
            app_config.headers = Some(
                headers
                    .into_iter()
                    .map(|f| {
                        if f.starts_with('@') {
                            // TODO: make this async? async closures are unstable?
                            read_file_lines_sync(&f).unwrap_or_else(|_| vec![])
                        } else {
                            // a bit costly? Yes? No?
                            vec![f]
                        }
                    })
                    .filter(|x| !x.is_empty())
                    .flatten()
                    .collect(),
            );
        }
        app_config.urls = config.free;
        // let's set some defaults
        if app_config.method.is_none() {
            if app_config.upload_file.is_some() {
                app_config.method = Some("PUT".to_string());
            } else if app_config.data.is_some() {
                app_config.method = Some("POST".to_string());
            } else {
                app_config.method = Some("GET".to_string());
            }
        }
        if app_config.ua.is_none() {
            app_config.ua = Some(format!("currrrl/{}", env!("CARGO_PKG_VERSION")));
        }
        if app_config.data.is_some() && app_config.upload_file.is_some() {
            app_config.error_str(
                "Warning: You can only select one HTTP request method! You asked for both PUT",
            );
            app_config.error_str("Warning: (-T, --upload-file) and POST (-d, --data).");
            None?;
        }
        Some(app_config)
    }
    pub(crate) fn error(&self, log: String) {
        self.error_str(&log);
    }
    fn error_str(&self, log: &str) {
        if self.log != LogStatus::Silent {
            eprintln!("{}", log);
        }
    }
    pub(crate) async fn run(&mut self) -> anyhow::Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(1usize);
        let out: JoinHandle<()>;
        if let Some(file) = self.output_file.as_ref() {
            let mut fd = File::create(file).await?;
            out = tokio::spawn(async move {
                while let Some(dat) = rx.recv().await {
                    let _ = fd.write(&dat).await;
                }
            });
        } else {
            out = tokio::spawn(async move {
                while let Some(dat) = rx.recv().await {
                    let _ = stdout().write(&dat).await;
                }
            });
        }
        while let Some(mut url) = self.urls.pop() {
            let method = self.method.as_ref().unwrap();
            if !url.contains("://") {
                url = format!("http://{}", url);
            }
            let uri = hyper::Uri::try_from(url)?;
            let mut builder = hyper::Request::builder().uri(&uri).method(method.as_str());
            if let Some(headers) = &self.headers {
                for h in headers {
                    if let Some(key) = h.find(':') {
                        let value = h[key + 1..].trim();
                        let key = &h[..key];
                        builder = builder.header(key, value)
                    }
                }
            }
            if let Some(ua) = self.ua.as_ref() {
                builder = builder.header("User-Agent", ua);
            }
            if let Some(auth) = self.auth.as_ref() {
                let encoded = base64::encode(auth);
                builder = builder.header("Authorization", format!("Basic {}", encoded));
            }
            let client =
                Client::builder().build::<_, hyper::Body>(hyper_tls::HttpsConnector::new());
            let mut result: Response<Body>;
            if let Some(upload_file) = self.upload_file.as_ref() {
                let mut fd = File::open(upload_file).await?;
                let (mut sender, body) = Body::channel();
                tokio::spawn(async move {
                    // Reuse this buffer
                    let mut buf = [0_u8; 1024 * 16];
                    loop {
                        let read_count = fd.read(&mut buf).await.unwrap();
                        if read_count == 0 {
                            break;
                        }
                        sender
                            .send_data(Bytes::copy_from_slice(&buf))
                            .await
                            .unwrap();
                    }
                });
                let request = builder.body(body)?;
                result = client.request(request).await?;
            } else if let Some(data) = self.data.as_ref() {
                let request = builder.body(Body::from(data.clone()))?;
                result = client.request(request).await?;
            } else {
                let request = builder.body(Body::empty())?;
                result = client.request(request).await?;
            }
            // let's do something with the result
            if self.include_headers {
                let status_line = format!("{:?} {}\n", result.version(), result.status());
                tx.send(status_line.into_bytes()).await?;
                for (key, value) in result.headers() {
                    let header_line = format!("{}: {:?}\n", key, value);
                    tx.send(header_line.into_bytes()).await?;
                }
                tx.send("\n".to_string().into_bytes()).await?;
            }
            if result.status().is_redirection() && self.follow_redirects {
                if let Some(x) = result.headers().get("location") {
                    let mut s = x.to_str()?.to_string();
                    if s.starts_with('/') {
                        s = format!(
                            "{}://{}{}",
                            uri.scheme_str().unwrap(),
                            uri.host().unwrap(),
                            s
                        );
                    }
                    self.urls.push(s);
                    continue; // skip next stuff - don't print body eh
                }
            }
            let body = result.body_mut();
            while let Some(data) = body.data().await {
                tx.send(data?.to_vec()).await?;
            }
        }
        drop(tx); // close channel
        tokio::try_join!(out)?;
        Ok(())
    }
}
