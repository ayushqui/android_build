/*
 * Copyright (C) 2023 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! `aconfig` is a build time tool to manage build time configurations, such as feature flags.

use anyhow::Result;
use clap::{builder::ArgAction, builder::EnumValueParser, Arg, ArgMatches, Command};
use std::fs;

mod aconfig;
mod cache;
mod commands;
mod protos;

use crate::cache::Cache;
use commands::{Input, Source};

fn cli() -> Command {
    Command::new("aconfig")
        .subcommand_required(true)
        .subcommand(
            Command::new("create-cache")
                .arg(
                    Arg::new("build-id")
                        .long("build-id")
                        .value_parser(clap::value_parser!(u32))
                        .required(true),
                )
                .arg(Arg::new("aconfig").long("aconfig").action(ArgAction::Append))
                .arg(Arg::new("override").long("override").action(ArgAction::Append))
                .arg(Arg::new("cache").long("cache").required(true)),
        )
        .subcommand(
            Command::new("dump").arg(Arg::new("cache").long("cache").required(true)).arg(
                Arg::new("format")
                    .long("format")
                    .value_parser(EnumValueParser::<commands::Format>::new())
                    .default_value("text"),
            ),
        )
}

fn open_zero_or_more_files(matches: &ArgMatches, arg_name: &str) -> Result<Vec<Input>> {
    let mut opened_files = vec![];
    for path in matches.get_many::<String>(arg_name).unwrap_or_default() {
        let file = Box::new(fs::File::open(path)?);
        opened_files.push(Input { source: Source::File(path.to_string()), reader: file });
    }
    Ok(opened_files)
}

fn main() -> Result<()> {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("create-cache", sub_matches)) => {
            let build_id = *sub_matches.get_one::<u32>("build-id").unwrap();
            let aconfigs = open_zero_or_more_files(sub_matches, "aconfig")?;
            let overrides = open_zero_or_more_files(sub_matches, "override")?;
            let cache = commands::create_cache(build_id, aconfigs, overrides)?;
            let path = sub_matches.get_one::<String>("cache").unwrap();
            let file = fs::File::create(path)?;
            cache.write_to_writer(file)?;
        }
        Some(("dump", sub_matches)) => {
            let path = sub_matches.get_one::<String>("cache").unwrap();
            let file = fs::File::open(path)?;
            let cache = Cache::read_from_reader(file)?;
            let format = sub_matches.get_one("format").unwrap();
            commands::dump_cache(cache, *format)?;
        }
        _ => unreachable!(),
    }
    Ok(())
}
