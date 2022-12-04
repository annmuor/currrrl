use std::process::exit;

use crate::client::App;

mod client;
mod utils;

async fn collect_options() -> Option<App> {
    let mut opts = getopts::Options::new();
    opts.optmulti(
        "H",
        "header",
        "Pass custom header(s) to server",
        "header/@file",
    );
    opts.optopt("d", "data", "HTTP POST data", "data");
    opts.optopt("X", "request", "Specify request method to use", "method");
    opts.optopt("o", "output", "Write to file instead of stdout", "file");
    opts.optopt("u", "user", "Server user and password", "user:password");
    opts.optopt(
        "A",
        "user-agent",
        "Send User-Agent <name> to server",
        "name",
    );
    opts.optopt(
        "T",
        "upload-tile",
        "Transfer local FILE to destination",
        "file",
    );
    opts.optflag(
        "O",
        "remote-name",
        "Write output to a file named as the remote file",
    );
    opts.optflag("L", "location", "Follow redirects");
    opts.optflag("v", "verbose", "Make the operation more talkative");
    opts.optflag("V", "version", "Show version number and quit");
    opts.optflag("s", "silent", "Silent mode");
    opts.optflag("k", "insecure", "Allow insecure server connections");
    opts.optflag(
        "i",
        "include",
        "Include protocol response headers in the output",
    );
    opts.optflag("", "recursive", "Download all found as wget do");
    opts.optflagopt("h", "help", "Get help for commands", "command");
    App::new(&opts).await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Some(mut app) = collect_options().await {
        let _ = app.run().await.map_err(|e| {
            app.error(format!("Warning: {}", e));
            exit(-1);
        });
    } else {
        exit(-1);
    }
    Ok(())
}
