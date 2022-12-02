use std::collections::HashMap;
use std::env;
use std::process::exit;

struct HttpRequest {
    method: String,
    protocol: String,
    address: String,
    headers: HashMap<String, Vec<String>>,
    body: String,
}
/**
Usage: curl [options...] <url>
 -d, --data <data>          HTTP POST data
 -f, --fail                 Fail fast with no output on HTTP errors
 -h, --help <category>      Get help for commands
 -i, --include              Include protocol response headers in the output
 -o, --output <file>        Write to file instead of stdout
 -O, --remote-name          Write output to a file named as the remote file
 -s, --silent               Silent mode
 -T, --upload-file <file>   Transfer local FILE to destination
 -u, --user <user:password> Server user and password
 -A, --user-agent <name>    Send User-Agent <name> to server
 -v, --verbose              Make the operation more talkative
 -V, --version              Show version number and quit
**/
fn collect_options() {
    let mut opts = getopts::Options::new();
    opts.optopt("d", "data", "HTTP POST data", "data");
    opts.optopt("X", "request", "Specify request method to use", "method");
    opts.optmulti( "H", "header", "Pass custom header(s) to server", "header/@file");
    opts.optopt("o", "--output", "Write to file instead of stdout", "file");
    opts.optflagopt("h", "help", "Get help for commands", "command");
    opts.optflag("v", "verbose", "Make the operation more talkative");
    opts.optflag("V", "version", "Show version number and quit");
    opts.optflag("s", "silent", "Silent mode");
    opts.optflag("i", "include", "Include protocol response headers in the output");
    opts.optopt("u", "user", "Server user and password", "user:password");
    opts.optopt("A", "user-agent", "Send User-Agent <name> to server", "name");

    if let Ok(result) = opts.parse(&env::args().collect::<Vec<String>>()[1..]) {
        if result.opt_present("V") {
            println!("Program name: currrrl, version: {}", env!("CARGO_PKG_VERSION"));
            exit(0);
        }
        if result.opt_present("h") {
            println!("{}", opts.usage("env!("CARGO_PKG_NAME")"));
            exit(0);
        }

    } else {
        eprintln!("Error parsing options");
        exit(-1);
    }


}
fn main() {
    collect_options()
}
