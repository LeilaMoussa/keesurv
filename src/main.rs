#![allow(unused)]

extern crate clap;
extern crate rand;
extern crate lettre;

use clap::{Parser, Subcommand};
use std::{io, time::{Instant, Duration}, collections::HashMap};
use rand::Rng;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

// put this in a config file
// deal better with units
const LIMIT: u64 = 1 * 3600;

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
        /// Email address for authn $to verify$
        #[clap(required = true, short, long)]
        email: String,
        /// Public key $to verify$
        #[clap(required = true, short, long)]
        pubkey: String,
    },
    /// finds someone's key from email address
    Serve {
        /// Email address to match $to verify$
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

fn send_code(addr: &str, code: &u8) -> Result<(), ()> {
    let email = Message::builder()
    .from("NoReply <noreply@gmail.com>".parse().unwrap())
    .to(addr)
    .subject("Confirmation")
    .body(code)
    .unwrap();

    // think about these
    let creds = Credentials::new("smtp_username".to_string(), "smtp_password".to_string());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    mailer.send(&email)
}

fn get_input() -> Option<u8> {
    // reconsider flattening this match and returning a result instead of option?
    let input: String;
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            // parse as u8
        },
        Err(_) => {
            None
        },
    }
}

fn check_and_delete(keys: &HashMap<String, Record>) {
    for (k, v) in keys.iter_mut() {
        if v.is_stale() {
            println!("Deleting entry with key {}", k);
            keys.remove(k);
        }
    }
}

fn main() {
    // Find file
    let mut keys: HashMap<String, Record> = HashMap::new();
    // fill hashmap

    check_and_delete(&keys);
    // To run, `cargo run -- <FLAGS>`
    let args = Cli::parse();
    match args.command {
        Commands::Store { name, email, pubkey } => {
            // generate random code
            let mut rng = rand::thread_rng();
            let code: u8 = rng.gen();
            // send to email
            match send_code(&email, &code) {
                Ok(_) => {
                    println!("Check your inbox.");
                },
                Err(err) => {
                    println!("Couldn't send email.");
                    return;
                },
            }
            // prompt for input
            let input = get_input();
            if input.is_none() {
                println!("Bad input.");
                return;
            }
            let input: u8 = input.unwrap();
            // compare, if success, make struct
            if input != code {
                println!("Wrong code.");
                return;
            }
            let entry = Record::new(&name, &email, &pubkey);
            // add to hashmap
            keys.insert(email, entry);
            // write to file
        },
        Commands::Serve { email } => {
            // lookup, print
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
}
