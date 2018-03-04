extern crate service;
#[macro_use]
extern crate slog;
extern crate slog_json;

use slog::{Drain, FnValue};
use std::sync::Mutex;
use service::Config;
use std::process;

fn main() {
    let log = slog::Logger::root(
        Mutex::new(slog_json::Json::default(std::io::stderr())).map(slog::Fuse),
        o!(
            "program" => env!("CARGO_PKG_NAME"),
            "version" => env!("CARGO_PKG_VERSION"),
            "source" => FnValue(move |info| {
               format!("{}:{} {}",
                       info.file(),
                       info.line(),
                       info.module(),
                       )
           }),
        ),
    );

    let config = Config::new().unwrap_or_else(|err| {
        crit!(log, "Problem parsing arguments: {}", err);
        process::exit(1);
    });

    service::run(log, config);
}
