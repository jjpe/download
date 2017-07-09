extern crate clap;
extern crate download;

use clap::{App, Arg, ArgMatches, SubCommand};
use download::*;
use std::env;
use std::fs::{DirEntry, File};
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::str;


const BIN_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str =  env!("CARGO_PKG_VERSION");
const AUTHORS: &str =  env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");



fn main() {
    let args = CliArgs::get();

    println!("args: {:#?}", args);
}



#[derive(Clone, Debug, PartialEq, Eq)]
struct CliArgs {
    from: Url,
    to: PathBuf,
    force: bool
}

impl CliArgs {
    pub fn get() -> Self {
        let matches: ArgMatches = Self::get_clap_matches();
        let url: &str = matches.value_of("url").unwrap_or_else(|| {
            println!("Error: Must provide an --url");
            process::exit(-1);
        });
        let output: &str = matches.value_of("output").unwrap_or_else(|| {
            println!("Error: Must provide an --output");
            process::exit(-1);
        });
        let force: bool = match matches.occurrences_of("force") {
            0 => false,
            _ => true,
        };

        Self {
            from: Url::parse(url).expect("Invalid url"),
            to: PathBuf::from(output),
            force: force
        }
    }

    fn get_clap_matches<'m>() -> ArgMatches<'m> {
        App::new(BIN_NAME)
            .version(VERSION)
            .about(DESCRIPTION)
            .author(AUTHORS)
            .arg(Arg::with_name("url")
                 .short("u")
                 .long("url")
                 .value_name("URL")
                 .help("The URL from where to download")
                 .takes_value(true))
            .arg(Arg::with_name("output")
                 .short("-o")
                 .long("output")
                 .value_name("FILE")
                 .help("The destination location to which to write the downloaded data.")
                 .takes_value(true))
            .arg(Arg::with_name("force")
                 .short("f")
                 .long("force")
                 .help("If set, overwrite FILE if it existed.
If not set, display a message if the file existed.
By default this flag is not set."))
            .get_matches()
    }
}
