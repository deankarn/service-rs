extern crate cadence;
extern crate chan_signal;
#[macro_use]
extern crate slog;
extern crate slog_json;
extern crate spmc;

use chan_signal::Signal;
use slog::{Drain, FnValue, Logger};
use cadence::prelude::*;
use cadence::StatsdClient;
use std::sync::Mutex;
use spmc::{Receiver, Sender, TryRecvError};
use std::thread;
use spmc::channel;
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::process;
#[macro_use]
extern crate clap;
use clap::{App, Arg};

pub struct Config {
    pub datadog_addr: String,
}

impl Config {
    pub fn new() -> Result<Config, &'static str> {
        const DATADOG_ADDR: &str = "datadog_addr";

        let args = App::new("My Service")
            .version(crate_version!())
            .author("Dean Karn <dean.karn@gmail.com>")
            .about("This is a test service")
            .arg(
                Arg::with_name(DATADOG_ADDR)
                    .short("d")
                    .long("datadog-addr")
                    .env("DATADOG_ADDR")
                    .default_value("127.0.0.1:8125")
                    .help("datadog address")
                    .required(true)
                    .takes_value(true),
            )
            .get_matches();

        Ok(Config {
            datadog_addr: args.value_of(DATADOG_ADDR).unwrap().to_string(), // can just unwrap as it's required and has default value
        })
    }
}

pub fn run(log: Logger, config: Config) {
    let metrics =
        StatsdClient::from_udp_host(env!("CARGO_PKG_NAME"), config.datadog_addr.clone()).unwrap();

    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    let (sender, receiver): (Sender<bool>, Receiver<bool>) = channel();

    info!(log, "Value for dd addr: {}", config.datadog_addr);
    let _ = metrics
        .incr_with_tags("test")
        .with_tag("addr", &config.datadog_addr)
        .send();

    let threads = (0..10)
        .map(|_| {
            let receiver = receiver.clone();
            let log = log.clone();
            let metrics = metrics.clone();
            thread::spawn(move || loop {
                if receiver.try_recv().err() == Some(TryRecvError::Disconnected) {
                    break;
                }
                let now = Instant::now();
                info!(log, "running");
                sleep(Duration::new(1, 0));
                info!(log, "finished");
                let _ = metrics.time_duration("loop.latency", now.elapsed());
            })
        })
        .collect::<Vec<_>>();

    // Blocks until this process is sent an INT or TERM signal.
    // Since the channel is never closed, we can unwrap the received value.
    signal.recv().unwrap();
    info!(log, "shutdown signal received");
    drop(sender);

    // TODO: join thread(s) here, they will also be listening to the cloned receiver
    // and will finish once current work gracefully finishes
    for h in threads {
        let _ = h.join();
    }
}
