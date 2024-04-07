#![allow(unused_imports, dead_code)]
use std::{io::Cursor, num::ParseIntError, path::Path, time::Duration};

use clap::Parser;
use futures::future::{select, Either};
use humansize::{make_format, BINARY};
use temp_dir::TempDir;
use tracing::*;

use gagbot_rs::{
    configure_tracing, db::{
        background_jobs::spawn_db_background_jobs_task, close_database, open_database, queries::message_log::{self, compress, verify_compressed_chunks, LogType}, spawn_db_task, vacuum_database, DbCommand 
    }, load_dotenv, Error
};
use zstd::encode_all;

// 0 means use the default (3)
const COMPRESSION_LEVEL: i32 = 0;

fn frequency_seconds_valid_range(s: &str) -> Result<u64, String> {
    let v = s.parse().map_err(|e: ParseIntError| e.to_string())?;

    if v < 60 {
        Err(format!(
            "Running more often than once per minute ({} seconds) is not recommended",
            v
        ))?;
    }
    Ok(v)
}

#[derive(Debug, Parser)]
#[clap(name = "gagbot.rs")]
struct Cli {
    #[clap(long, env, default_value = "gagbot.sqlite")]
    sqlite_connection_string: String,
    #[clap(long, env, default_value = "64")]
    database_command_channel_bound: usize,
    #[clap(long, env, default_value = "3600", value_parser = frequency_seconds_valid_range)]
    background_task_frequency_seconds: u64,
}

// This simulates a single core vm: #[tokio::main(flavor = "multi_thread", worker_threads = 1)]
#[tokio::main]
async fn main() -> Result<(), Error> {
    load_dotenv()?;
    configure_tracing();

    let args = Cli::parse();
    debug!("Parsed args: {:#?}", args);

    let mut sqlite_con = open_database(&args.sqlite_connection_string, true, false)?;
    
    loop {
        let cs = message_log::get_compression_state(&sqlite_con)?;
        info!("{}", cs.uncompressed_messages);
    }

    close_database(sqlite_con)?;

    Ok(())
}