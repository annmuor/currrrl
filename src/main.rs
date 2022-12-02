use std::collections::HashMap;
use std::env;
use std::process::exit;

use getopts::HasArg::{No, Yes};
use getopts::Occur::Multi;
use getopts::Options;

#[derive(Debug)]
#[non_exhaustive]
enum LogStatus {
    FULL,
    NORMAL,
    SILENT,
}

#[derive(Debug)]
struct AppConfig {
    log: LogStatus,
    method: Option<String>,
    headers: Option<Vec<String>>,
    output_file: Option<String>,
    ua: Option<String>,
    data: Option<Vec<u8>>,
    upload_file: Option<String>,
    include: bool,
    auth: Option<String>,
    urls: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            log: LogStatus::NORMAL,
            method: None,
            headers: None,
            output_file: None,
            ua: None,
            data: None,
            upload_file: None,
            include: false,
            auth: None,
            urls: vec![],
        }
    }
}


impl AppConfig {
    fn new(options: &getopts::Options) -> Option<Self> {
        let args = env::args().collect::<Vec<String>>();
        let config = options.parse(&args[1..]).ok().or_else(|| {
            eprintln!("{}", options.short_usage(&args[0]));
            None
        })?;
        if config.opt_present("V") {
            eprintln!("Version: {}", env!("CARGO_PKG_VERSION"));
            None?;
        }
        if config.opt_present("h") {
            eprintln!("{}", options.usage("currrrll is the best CURL alternative"));
            None?;
        }
        if config.free.len() == 0 {
            eprintln!("no URL specified!");
            None?;
        }
        let mut app_config = AppConfig::default();
        if config.opt_present("s") {
            app_config.log = LogStatus::SILENT;
        } else if config.opt_present("v") {
            app_config.log = LogStatus::FULL;
        }
        if config.opt_present("i") {
            app_config.include = true;
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
            if x.starts_with("@") {

            } else {
                app_config.data = Some(x.into());
            }
        }
        let headers = config.opt_strs("H");
        if headers.len() > 0 {
            app_config.headers = Some(headers);
        }
        app_config.urls = config.free;
        Some(app_config)
    }
    async fn run(&self) {
        todo!()
    }
}

fn collect_options() -> Option<AppConfig> {
    let mut opts = getopts::Options::new();
    opts.optmulti("H", "header", "Pass custom header(s) to server", "header/@file");
    opts.optopt("d", "data", "HTTP POST data", "data");
    opts.optopt("X", "request", "Specify request method to use", "method");
    opts.optopt("o", "output", "Write to file instead of stdout", "file");
    opts.optopt("u", "user", "Server user and password", "user:password");
    opts.optopt("A", "user-agent", "Send User-Agent <name> to server", "name");
    opts.optopt("T", "upload-tile", "Transfer local FILE to destination", "file");
    opts.optflag("v", "verbose", "Make the operation more talkative");
    opts.optflag("V", "version", "Show version number and quit");
    opts.optflag("s", "silent", "Silent mode");
    opts.optflag("i", "include", "Include protocol response headers in the output");
    opts.optflagopt("h", "help", "Get help for commands", "command");
    AppConfig::new(&opts)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Some(app) = collect_options() {
        tokio::spawn(async move { app.run().await });
    } else {
        exit(-1);
    }
    Ok(())
}
