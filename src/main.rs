use anyhow::Result;
use clap::Parser;
use dashmap::DashMap;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// A high-performance log file analyzer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The log files to analyze
    #[arg(required = true)]
    files: Vec<PathBuf>,
    
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Search for specific IP address or pattern
    #[arg(short, long)]
    search: Option<String>,
    
    /// Number of threads to use (default: all available)
    #[arg(short, long)]
    threads: Option<usize>,
    
    /// Export results to file (JSON or CSV based on extension)
    #[arg(short, long)]
    export: Option<PathBuf>,
    
    /// Show all results (not just top 20)
    #[arg(short, long)]
    all: bool,
    
    /// Extract and show additional log fields
    #[arg(long)]
    extract_fields: bool,
    
    /// Filter by HTTP status code
    #[arg(long)]
    status: Option<u16>,
    
    /// Filter by HTTP method
    #[arg(long)]
    method: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogEntry {
    ip: String,
    timestamp: Option<String>,
    method: Option<String>,
    path: Option<String>,
    status: Option<u16>,
    size: Option<u64>,
    user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnalysisResult {
    total_lines: u64,
    total_files: usize,
    ip_counts: Vec<(String, u64)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    log_entries: Option<Vec<LogEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_codes: Option<HashMap<u16, u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    methods: Option<HashMap<String, u64>>,
}

// Pre-compiled regex patterns
static IP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?P<ip>\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap()
});

static LOG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(?P<ip>\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})\s+\S+\s+\S+\s+\[(?P<timestamp>[^\]]+)\]\s+"(?P<method>\w+)\s+(?P<path>[^\s]+)\s+[^"]+"\s+(?P<status>\d{3})\s+(?P<size>\d+|-)"#).unwrap()
});

static USER_AGENT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#""([^"]+)"\s*$"#).unwrap()
});

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Set thread pool size if specified
    if let Some(threads) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()?;
    }
    
    let ip_counts = Arc::new(DashMap::with_capacity(10_000));
    let total_lines = Arc::new(AtomicU64::new(0));
    let status_counts = if args.extract_fields {
        Some(Arc::new(DashMap::new()))
    } else {
        None
    };
    let method_counts = if args.extract_fields {
        Some(Arc::new(DashMap::new()))
    } else {
        None
    };
    let log_entries = if args.extract_fields {
        Some(Arc::new(DashMap::new()))
    } else {
        None
    };

    if args.verbose {
        println!("Configuration:");
        println!("  Files: {:?}", args.files);
        println!("  Threads: {}", args.threads.unwrap_or_else(|| rayon::current_num_threads()));
        println!("  Extract fields: {}", args.extract_fields);
        if let Some(ref search) = args.search {
            println!("  Searching for: {}", search);
        }
        println!();
    }

    println!("Analyzing {} files...", args.files.len());

    let start = std::time::Instant::now();
    process_files(
        &args,
        ip_counts.clone(),
        total_lines.clone(),
        status_counts.clone(),
        method_counts.clone(),
        log_entries.clone(),
    )?;
    let duration = start.elapsed();

    // Prepare results
    let mut ip_results: Vec<_> = ip_counts.iter().map(|e| (e.key().clone(), *e.value())).collect();
    ip_results.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Filter by search if specified
    if let Some(ref search) = args.search {
        ip_results.retain(|(ip, _)| ip.contains(search));
    }

    let result = AnalysisResult {
        total_lines: total_lines.load(Ordering::Relaxed),
        total_files: args.files.len(),
        ip_counts: ip_results.clone(),
        log_entries: log_entries.map(|le| {
            le.iter().map(|e| e.value().clone()).collect()
        }),
        status_codes: status_counts.map(|sc| {
            sc.iter().map(|e| (*e.key(), *e.value())).collect()
        }),
        methods: method_counts.map(|mc| {
            mc.iter().map(|e| (e.key().clone(), *e.value())).collect()
        }),
    };

    // Export if requested
    if let Some(ref export_path) = args.export {
        export_results(&result, export_path)?;
        println!("Results exported to: {:?}", export_path);
    }

    // Display results
    println!("\nAnalysis complete in {:.2?}", duration);
    println!("Total lines processed: {}", result.total_lines);
    
    if args.search.is_some() {
        println!("\n--- Search Results ---");
        println!("Found {} matching IP addresses", ip_results.len());
    }
    
    println!("\n--- IP Address Counts ---");
    let display_limit = if args.all { ip_results.len() } else { 20.min(ip_results.len()) };
    
    for (ip, count) in ip_results.iter().take(display_limit) {
        println!("{}: {}", ip, count);
    }
    
    if !args.all && ip_results.len() > 20 {
        println!("... and {} more IP addresses (use --all to see all)", ip_results.len() - 20);
    }
    
    // Display additional stats if extract_fields is enabled
    if args.extract_fields {
        if let Some(ref status_codes) = result.status_codes {
            println!("\n--- HTTP Status Codes ---");
            let mut status_vec: Vec<_> = status_codes.iter().collect();
            status_vec.sort_by_key(|&(k, _)| k);
            for (status, count) in status_vec {
                println!("{}: {}", status, count);
            }
        }
        
        if let Some(ref methods) = result.methods {
            println!("\n--- HTTP Methods ---");
            let mut method_vec: Vec<_> = methods.iter().collect();
            method_vec.sort_by(|a, b| b.1.cmp(a.1));
            for (method, count) in method_vec {
                println!("{}: {}", method, count);
            }
        }
    }

    Ok(())
}

fn process_files(
    args: &Args,
    ip_counts: Arc<DashMap<String, u64>>,
    total_lines: Arc<AtomicU64>,
    status_counts: Option<Arc<DashMap<u16, u64>>>,
    method_counts: Option<Arc<DashMap<String, u64>>>,
    log_entries: Option<Arc<DashMap<String, LogEntry>>>,
) -> Result<()> {
    args.files.par_iter().for_each(|file_path| {
        if let Err(e) = process_file(
            file_path,
            args,
            ip_counts.clone(),
            total_lines.clone(),
            status_counts.clone(),
            method_counts.clone(),
            log_entries.clone(),
        ) {
            eprintln!("Error processing file {:?}: {}", file_path, e);
        }
    });
    Ok(())
}

fn process_file(
    file_path: &PathBuf,
    args: &Args,
    ip_counts: Arc<DashMap<String, u64>>,
    total_lines: Arc<AtomicU64>,
    status_counts: Option<Arc<DashMap<u16, u64>>>,
    method_counts: Option<Arc<DashMap<String, u64>>>,
    log_entries: Option<Arc<DashMap<String, LogEntry>>>,
) -> Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::with_capacity(64 * 1024, file);
    
    let mut local_ip_counts = HashMap::with_capacity(1000);
    let mut local_status_counts = HashMap::new();
    let mut local_method_counts = HashMap::new();
    let mut lines_processed = 0u64;
    
    if args.verbose {
        println!("Processing file: {:?}", file_path);
    }
    
    for line in reader.lines() {
        let line = line?;
        lines_processed += 1;
        
        // Fast path: check if line starts with a digit
        if let Some(first_char) = line.chars().next() {
            if !first_char.is_ascii_digit() {
                continue;
            }
        }
        
        // Try to parse full log entry if extract_fields is enabled
        if args.extract_fields {
            if let Some(caps) = LOG_REGEX.captures(&line) {
                let ip = caps.name("ip").unwrap().as_str().to_string();
                let status = caps.name("status")
                    .and_then(|s| s.as_str().parse::<u16>().ok());
                let method = caps.name("method")
                    .map(|m| m.as_str().to_string());
                
                // Apply filters
                if let Some(filter_status) = args.status {
                    if status != Some(filter_status) {
                        continue;
                    }
                }
                
                if let Some(ref filter_method) = args.method {
                    if method.as_ref().map(|m| m != filter_method).unwrap_or(true) {
                        continue;
                    }
                }
                
                // Count IP
                *local_ip_counts.entry(ip.clone()).or_insert(0u64) += 1;
                
                // Count status and method
                if let Some(status_val) = status {
                    *local_status_counts.entry(status_val).or_insert(0u64) += 1;
                }
                if let Some(method_val) = method.clone() {
                    *local_method_counts.entry(method_val).or_insert(0u64) += 1;
                }
                
                // Store full entry if needed
                if log_entries.is_some() {
                    let entry = LogEntry {
                        ip: ip.clone(),
                        timestamp: caps.name("timestamp").map(|t| t.as_str().to_string()),
                        method,
                        path: caps.name("path").map(|p| p.as_str().to_string()),
                        status,
                        size: caps.name("size")
                            .and_then(|s| s.as_str().parse::<u64>().ok()),
                        user_agent: USER_AGENT_REGEX.captures(&line)
                            .and_then(|ua_caps| ua_caps.get(1))
                            .map(|ua| ua.as_str().to_string()),
                    };
                    if let Some(ref le) = log_entries {
                        le.insert(format!("{}-{}", ip, lines_processed), entry);
                    }
                }
            }
        } else {
            // Simple IP extraction
            if let Some(caps) = IP_REGEX.captures(&line) {
                if let Some(ip_match) = caps.name("ip") {
                    let ip = ip_match.as_str();
                    *local_ip_counts.entry(ip.to_string()).or_insert(0u64) += 1;
                }
            }
        }
        
        // Batch update every 10,000 lines
        if lines_processed % 10_000 == 0 {
            update_global_counts(&local_ip_counts, &ip_counts);
            if let Some(ref sc) = status_counts {
                update_global_counts_u16(&local_status_counts, sc);
            }
            if let Some(ref mc) = method_counts {
                update_global_counts(&local_method_counts, mc);
            }
            local_ip_counts.clear();
            local_status_counts.clear();
            local_method_counts.clear();
        }
    }
    
    // Final update
    update_global_counts(&local_ip_counts, &ip_counts);
    if let Some(ref sc) = status_counts {
        update_global_counts_u16(&local_status_counts, sc);
    }
    if let Some(ref mc) = method_counts {
        update_global_counts(&local_method_counts, mc);
    }
    
    total_lines.fetch_add(lines_processed, Ordering::Relaxed);
    
    if args.verbose {
        println!("Processed {} lines from {:?}", lines_processed, file_path);
    }
    
    Ok(())
}

fn update_global_counts(local: &HashMap<String, u64>, global: &Arc<DashMap<String, u64>>) {
    for (key, count) in local {
        global.entry(key.clone())
            .and_modify(|e| *e += count)
            .or_insert(*count);
    }
}

fn update_global_counts_u16(local: &HashMap<u16, u64>, global: &Arc<DashMap<u16, u64>>) {
    for (key, count) in local {
        global.entry(*key)
            .and_modify(|e| *e += count)
            .or_insert(*count);
    }
}

fn export_results(result: &AnalysisResult, path: &PathBuf) -> Result<()> {
    let mut file = File::create(path)?;
    
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => {
            let json = serde_json::to_string_pretty(result)?;
            writeln!(file, "{}", json)?;
        }
        Some("csv") => {
            // Write CSV header
            writeln!(file, "IP,Count")?;
            for (ip, count) in &result.ip_counts {
                writeln!(file, "{},{}", ip, count)?;
            }
        }
        _ => {
            // Default to text format
            writeln!(file, "Log Analysis Results")?;
            writeln!(file, "====================")?;
            writeln!(file, "Total lines: {}", result.total_lines)?;
            writeln!(file, "Total files: {}", result.total_files)?;
            writeln!(file)?;
            writeln!(file, "IP Address Counts:")?;
            for (ip, count) in &result.ip_counts {
                writeln!(file, "{}: {}", ip, count)?;
            }
            
            if let Some(ref status_codes) = result.status_codes {
                writeln!(file)?;
                writeln!(file, "HTTP Status Codes:")?;
                let mut status_vec: Vec<_> = status_codes.iter().collect();
                status_vec.sort_by_key(|&(k, _)| k);
                for (status, count) in status_vec {
                    writeln!(file, "{}: {}", status, count)?;
                }
            }
            
            if let Some(ref methods) = result.methods {
                writeln!(file)?;
                writeln!(file, "HTTP Methods:")?;
                for (method, count) in methods {
                    writeln!(file, "{}: {}", method, count)?;
                }
            }
        }
    }
    
    Ok(())
}
