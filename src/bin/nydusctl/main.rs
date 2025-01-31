// Copyright 2020 Ant Group. All rights reserved.
//
// SPDX-License-Identifier: Apache-2.0

#[macro_use(crate_authors, crate_version)]
extern crate clap;
#[macro_use]
extern crate anyhow;

use std::collections::HashMap;

use anyhow::Result;
use clap::{App, Arg, SubCommand};

mod client;
mod commands;

use commands::{CommandBackend, CommandBlobcache, CommandDaemon, CommandFsStats};

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = App::new("A client to query and configure nydusd")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("sock")
                .long("sock")
                .help("Unix domain socket path")
                .takes_value(true)
                .required(true)
                .global(false),
        )
        .arg(
            Arg::with_name("raw")
                .long("raw")
                .help("Output plain json")
                .takes_value(false)
                .global(false),
        )
        .subcommand(SubCommand::with_name("info").help("Get nydusd working status"))
        .subcommand(
            SubCommand::with_name("set")
                .help("Configure nydusd")
                .arg(
                    Arg::with_name("KIND")
                        .help("what item to configure")
                        .required(true)
                        .possible_values(&["log_level"])
                        .takes_value(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("VALUE")
                        .help("what item to configure")
                        .required(true)
                        .takes_value(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("metrics")
                .help("Configure nydusd")
                .arg(
                    Arg::with_name("category")
                        .help("Show the category of metrics: blobcache, backend, fsstats")
                        .required(true)
                        .possible_values(&["blobcache", "backend", "fsstats"])
                        .takes_value(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("interval")
                        .long("interval")
                        .short("I")
                        .required(false)
                        .takes_value(true),
                ),
        )
        .get_matches();

    // Safe to unwrap because it is required by Clap
    let sock = cmd.value_of("sock").unwrap();
    let raw = cmd.is_present("raw");
    let client = client::NydusdClient::new(sock);

    if cmd.subcommand_matches("info").is_some() {
        let cmd = CommandDaemon {};
        cmd.execute(raw, &client, None).await?;
    }

    if let Some(matches) = cmd.subcommand_matches("set") {
        // Safe to unwrap since the below two arguments are required by clap.
        let kind = matches.value_of("KIND").unwrap().to_string();
        let value = matches.value_of("VALUE").unwrap().to_string();
        let mut items = HashMap::new();
        items.insert(kind, value);

        let cmd = CommandDaemon {};
        cmd.execute(raw, &client, Some(items)).await?;
    }

    if let Some(matches) = cmd.subcommand_matches("metrics") {
        // Safe to unwrap as it is required by clap
        let category = matches.value_of("category").unwrap();

        let mut context = HashMap::new();

        matches
            .value_of("interval")
            .map(|i| context.insert("interval".to_string(), i.to_string()));

        match category {
            "blobcache" => {
                let cmd = CommandBlobcache {};
                cmd.execute(raw, &client, None).await?
            }
            "backend" => {
                let cmd = CommandBackend {};
                cmd.execute(raw, &client, Some(context)).await?
            }
            "fsstats" => {
                let cmd = CommandFsStats {};
                cmd.execute(raw, &client, None).await?
            }
            _ => println!("Illegal category"),
        }
    }

    Ok(())
}
