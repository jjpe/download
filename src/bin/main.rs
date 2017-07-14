extern crate clap;
extern crate download;

use clap::{App, Arg, ArgMatches};
use download::*;
use std::path::PathBuf;
use std::process;
use std::str;

const BIN_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str =  env!("CARGO_PKG_VERSION");
const AUTHORS: &str =  env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");


enum ReturnCode {
    Ok = 0,
    InvalidDestination = 1,
    FailedToDownload = 2,
    InvalidUrl = 3,
}

fn main() {
    let args = CliArgs::get();
    match args.to.to_str() {
        None => {
            println!("ERROR: Specify a valid destination using --to <FILE>");
            process::exit(ReturnCode::InvalidDestination as i32);
        },
        Some(file_name) if file_name.is_empty() => {
            println!("ERROR: Specify a valid destination using --to <FILE>");
            process::exit(ReturnCode::InvalidDestination as i32);
        },
        Some(_) => {/* continue below */},
    }

    let result: DlResult<DlStatus> = download::Dl::new()
        .overwrite(args.overwrite)
        .verbose(args.verbose)
        .download(&args.from, &args.to);
    match result {
        Ok(DlStatus::Downloaded{ location, num_bytes }) => {
            println!("Downloaded {}  ({} bytes)",
                     location.display(),
                     num_bytes);
        },
        Ok(DlStatus::Replaced{ location, num_bytes }) => {
            println!("Downloaded {} (replacement size: {} bytes)",
                     location.display(),
                     num_bytes);
        },
        Ok(DlStatus::FileExists(location)) => {
            println!("File exists: {}", location.display());
            process::exit(ReturnCode::Ok as i32);
        },
        Err(dl_err) => {
            println!("Failed to download {}:  {:#?}", args.to.display(), dl_err);
            process::exit(ReturnCode::FailedToDownload as i32);
        }
    }
}



#[derive(Clone, Debug, PartialEq, Eq)]
struct CliArgs {
    from: Url,
    to: PathBuf,
    overwrite: bool,
    verbose: bool,
}

impl CliArgs {
    pub fn get() -> Self {
        let matches: ArgMatches = Self::get_clap_matches();
        let from: &str = matches.value_of("from").unwrap_or_else(|| {
            println!("Error: Must provide an URL using --from");
            process::exit(ReturnCode::InvalidUrl as i32);
        });
        let to: &str = matches.value_of("to").unwrap_or_else(|| {
            println!("Error: Must provide a destination file using --to");
            process::exit(ReturnCode::InvalidDestination as i32);
        });
        let overwrite: bool = match matches.occurrences_of("overwrite") {
            0 => false,
            _ => true,
        };
        let verbose: bool = match matches.occurrences_of("verbose") {
            0 => false,
            _ => true,
        };

        Self {
            from: Url::parse(from).expect("Invalid url"),
            to: PathBuf::from(to),
            overwrite: overwrite,
            verbose: verbose,
        }
    }

    fn get_clap_matches<'m>() -> ArgMatches<'m> {
        App::new(BIN_NAME)
            .version(VERSION)
            .about(DESCRIPTION)
            .author(AUTHORS)
            .arg(Arg::with_name("from")
                 .long("from")
                 .value_name("URL")
                 .help("The URL from where to download")
                 .takes_value(true))
            .arg(Arg::with_name("to")
                 .long("to")
                 .value_name("FILE")
                 .help("The file system destination for the downloaded data.")
                 .takes_value(true))
            .arg(Arg::with_name("overwrite")
                 .long("overwrite")
                 .help("If set, overwrite FILE if it existed, no questions asked.
If not set and the FILE existed, display a message but don't overwrite.
By default this flag is not set."))
            .arg(Arg::with_name("verbose")
                 .long("verbose")
                 .help("Use verbose output if set, which includes transfer data.
By default this flag is not set."))
            .get_matches()
    }
}
