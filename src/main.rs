#![allow(unused)]

extern crate clap;
extern crate rand;
extern crate lettre;
extern crate serde_json;

use clap::{Parser, Subcommand};
use std::{io, time::{Instant, Duration}, collections::HashMap};
use rand::Rng;
use lettre::transport::smtp::{authentication::Credentials, response::Response};
use lettre::{Message, SmtpTransport, Transport};
use std::fs::OpenOptions;
use serde_json::{Value, Map};
use lettre::message::Mailbox;
use std::fs;

// put this in a config file $
// deal better with units $
const LIMIT: u64 = 1 * 3600;
const FILENAME: &str = "keys.json";

/// Keyserver POC
#[derive(Parser, Debug)]
#[clap(author="moussa", version="0.0.1", about="i took some liberties", long_about="a keyserver poc that runs locally")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// authenticates then stores your key
    #[clap(arg_required_else_help = true)]
    Store {
        /// Name of entity
        #[clap(short, long)]
        name: String,
        /// Email address for authn $to verify
        #[clap(required = true, short, long)]
        email: String,
        /// Public key $to verify
        #[clap(required = true, short, long)]
        pubkey: String,
    },
    /// finds someone's key from email address
    Serve {
        /// Email address to match $to verify
        #[clap(required = true, short, long)]
        email: String,
    },
}

#[derive(Debug)]
struct Record {
    name: String,
    email: String,
    pubkey: String,
    created_on: Instant,
}

impl Record {
    fn new(name: &str, email: &str, pubkey: &str) -> Self {
        Record {
            name: name.to_string(),
            email: email.to_string(),
            pubkey: pubkey.to_string(),
            created_on: Instant::now()
        }
    }

    fn is_stale(&self) -> bool {
        self.created_on.elapsed() > Duration::from_secs(LIMIT)  // unit??
    }
}

fn send_code(addr: &str, code: &u8) -> Result<Response, lettre::transport::smtp::Error> {
    // $test this
    let email = Message::builder()
    .from("NoReply <noreply@gmail.com>".parse().unwrap())
    .to("Leila Moussa <l.moussa@aui.ma>".parse().unwrap())
    .subject("Confirmation")
    .body(code.to_string())
    .unwrap();

    let creds = Credentials::new("leila.farah.mouusa@gmail.com".to_string(), "babamohamed11".to_string());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    mailer.send(&email)
}

fn get_input() -> Option<usize> {
    // reconsider flattening this match and returning a result instead of option? $
    let mut input: String = String::from("");
    match io::stdin().read_line(&mut input) {
        Ok(val) => {
            Some(val)
        },
        Err(_) => {
            None
        },
    }
}

// fn check_and_delete(keys: &Map<String, Value>) {
//     for (k, v) in keys.iter_mut() {
//         if v.is_stale() {
//             println!("Deleting entry with key {}", k);
//             keys.remove(k);
//         }
//     }
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("duration {:?}", Duration::from_secs(LIMIT));
    // Open or create file
    //let mut file = OpenOptions::new().write(true).create(true).open(FILENAME).unwrap();
    let mut contents = fs::read_to_string(FILENAME)?;
    // parse as json
    let parsed: Value = serde_json::from_str(&contents)?;
    let mut keys: Map<String, Value> = parsed.as_object().unwrap().clone();

    println!("map {:?}", keys);
    return Ok(());
    //check_and_delete(&keys);
    // To run, `cargo run -- <FLAGS>`
    let args = Cli::parse();
    match args.command {
        Commands::Store { name, email, pubkey } => {
            let mut rng = rand::thread_rng();
            let code: u8 = rng.gen();
            match send_code(&email, &code) {
                Ok(_) => {
                    println!("Check your inbox.");
                },
                Err(err) => {
                    println!("Couldn't send email.");
                    return Ok(()); // look at all these returns $
                },
            }
            let input = get_input();
            if input.is_none() {
                println!("Bad input.");
                return Ok(());
            }
            let input: u8 = input.unwrap() as u8;
            println!("got input {}", input);
            if input != code {
                println!("Wrong code.");
                return Ok(());
            }
            let entry = Record::new(&name, &email, &pubkey);
            // $ how to use Value as Record
            //keys.insert(email, entry);
            // write to file immediately $
        },
        Commands::Serve { email } => {
            match keys.get(&email) {
                Some(record) => {
                    println!("{:?}", record);
                },
                None => {
                    println!("Not found.");
                },
            }
        },
    }
    Ok(())
}
