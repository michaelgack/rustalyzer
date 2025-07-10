use anyhow::Result;
use clap::Parser;
use dashmap::DashMap;
use rayon::prelude::*;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use once_cell::sync::Lazy;

/// A high-performance log file analyzer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The log files to analyze
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

// Pre-compiled regex for better performance
static IP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?P<ip>\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap()
});

fn main() -> Result<()> {
    let args = Args::parse();
    let ip_counts = Arc::new(DashMap::with_capacity(10_000)); // Pre-allocate capacity
    let total_lines = Arc::new(AtomicU64::new(0));

    println!("Analyzing {} files...", args.files.len());

    let start = std::time::Instant::now();
    process_files(&args.files, ip_counts.clone(), total_lines.clone())?;
    let duration = start.elapsed();

    println!("\nAnalysis complete in {:.2?}", duration);
    println!("Total lines processed: {}", total_lines.load(Ordering::Relaxed));
    println!("\n--- IP Address Counts ---");
    
    // Sort by count (descending) for better readability
    let mut entries: Vec<_> = ip_counts.iter().map(|e| (e.key().clone(), *e.value())).collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    
    for (ip, count) in entries.iter().take(20) { // Show top 20 IPs
        println!("{}: {}", ip, count);
    }
    
    if entries.len() > 20 {
        println!("... and {} more IP addresses", entries.len() - 20);
    }

    Ok(())
}

fn process_files(files: &[PathBuf], ip_counts: Arc<DashMap<String, u64>>, total_lines: Arc<AtomicU64>) -> Result<()> {
    files.par_iter().for_each(|file_path| {
        if let Err(e) = process_file(file_path, ip_counts.clone(), total_lines.clone()) {
            eprintln!("Error processing file {:?}: {}", file_path, e);
        }
    });
    Ok(())
}

fn process_file(file_path: &PathBuf, ip_counts: Arc<DashMap<String, u64>>, total_lines: Arc<AtomicU64>) -> Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::with_capacity(64 * 1024, file); // 64KB buffer
    
    let mut local_counts = std::collections::HashMap::with_capacity(1000);
    let mut lines_processed = 0u64;
    
    for line in reader.lines() {
        let line = line?;
        lines_processed += 1;
        
        // Fast path: check if line starts with a digit before regex
        if let Some(first_char) = line.chars().next() {
            if !first_char.is_ascii_digit() {
                continue;
            }
        }
        
        if let Some(caps) = IP_REGEX.captures(&line) {
            if let Some(ip_match) = caps.name("ip") {
                let ip = ip_match.as_str();
                *local_counts.entry(ip.to_string()).or_insert(0u64) += 1;
            }
        }
        
        // Batch update every 10,000 lines to reduce contention
        if lines_processed % 10_000 == 0 {
            update_global_counts(&local_counts, &ip_counts);
            local_counts.clear();
        }
    }
    
    // Final update
    update_global_counts(&local_counts, &ip_counts);
    total_lines.fetch_add(lines_processed, Ordering::Relaxed);
    
    Ok(())
}

fn update_global_counts(local: &std::collections::HashMap<String, u64>, global: &Arc<DashMap<String, u64>>) {
    for (ip, count) in local {
        global.entry(ip.clone())
            .and_modify(|e| *e += count)
            .or_insert(*count);
    }
}
