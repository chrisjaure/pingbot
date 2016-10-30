extern crate slack_hook;
extern crate hyper;
extern crate docopt;
extern crate rustc_serialize;

use std::thread;
use std::time;

use slack_hook::{Slack, PayloadBuilder};
use hyper::client::{Client, Response};
use docopt::Docopt;

const USAGE: &'static str = "
pinger

Description:
    Ping a url every n minutes. If slack-url is provided, a message will be
    posted altering of a non-200 status.

Usage:
    pinger [options] <url> <minutes>

Options:
    -h --help             Show this screen.
    --slack-url=<url>     Slack webhook url.
    --bot-name=<name>     Name of bot. [default: pingbot]
    --bot-emoji=<emoji>   Emoji to use for bot. [default: warning]
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_slack_url: String,
    flag_bot_name: String,
    flag_bot_emoji: String,
    arg_url: String,
    arg_minutes: u64
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    let mut last_ping_successful: bool = true;

    loop {
        let until_secs = time::Duration::from_secs(args.arg_minutes * 60);
        let request = ping(&args.arg_url);
        match request {
            Ok(response) => match response.status {
                hyper::Ok => {
                    if !last_ping_successful {
                        send_alert(&args, "200 status returned! The site is back :)");
                    }
                    println!("Ping ok");
                    last_ping_successful = true;
                },
                _ => {
                    send_alert(&args, "Non-200 status code returned!");
                    last_ping_successful = false;
                }
            },
            _ => {
                send_alert(&args, "Not able to ping!");
                last_ping_successful = false;
            }
        }
        thread::sleep(until_secs);
    }
}

fn ping(url: &str) -> Result<Response, hyper::Error> {
    let client = Client::new();
    client.get(url).send()
}

fn send_alert(options: &Args, message: &str) {
    let &Args { ref flag_slack_url, ref flag_bot_name, ref flag_bot_emoji, .. } = options;
    if flag_slack_url.is_empty() {
        println!("{}", message); 
        return;
    }
    let slack = Slack::new(flag_slack_url.as_str()).unwrap();
    let p = PayloadBuilder::new()
      .text(message.to_string())
      .username(flag_bot_name.to_string())
      .icon_emoji(flag_bot_emoji.to_string())
      .build()
      .unwrap();

    let res = slack.send(&p);
    match res {
        Ok(()) => println!("Sent message to slack."),
        Err(x) => println!("Error sending message to slack: {:?}",x)
    }
}
