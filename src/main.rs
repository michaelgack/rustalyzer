use anyhow::Result;
use clap::Parser;
use dashmap::DashMap;
use rayon::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;

/// A high-performance log file analyzer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The log files to analyze
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let ip_counts = Arc::new(DashMap::new());

    println!("Analyzing {} files...", args.files.len());

    process_files(&args.files, ip_counts.clone())?;

    println!("Analysis complete.");
    println!("--- IP Address Counts ---");
    for item in ip_counts.iter() {
        println!("{}: {}", item.key(), item.value());
    }

    Ok(())
}

fn process_files(files: &[PathBuf], ip_counts: Arc<DashMap<String, i32>>) -> Result<()> {
    files.par_iter().for_each(|file_path| {
        if let Err(e) = process_file(file_path, ip_counts.clone()) {
            eprintln!("Error processing file {:?}: {}", file_path, e);
        }
    });
    Ok(())
}

fn process_file(file_path: &PathBuf, ip_counts: Arc<DashMap<String, i32>>) -> Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let re = Regex::new(r"^(?P<ip>\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})")?;

    for line in reader.lines() {
        let line = line?;
        if let Some(caps) = re.captures(&line) {
            let ip = caps.name("ip").unwrap().as_str().to_string();
            *ip_counts.entry(ip).or_insert(0) += 1;
        }
    }

    Ok(())
}
