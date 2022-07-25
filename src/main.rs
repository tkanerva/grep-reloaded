use chrono::prelude::DateTime;
use chrono::Utc;
use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
use std::process;
use std::time::{Duration, UNIX_EPOCH};

#[derive(Parser)]
struct Cli {
    /// The pattern to look for (it is a regex)
    pattern: String,

    /// output only the count of matching lines
    #[clap(short, long, action = clap::ArgAction::Count)]
    count: u8,

    /// quiet output
    #[clap(short, long, action = clap::ArgAction::Count)]
    quiet: u8,

    /// case insensitive search
    #[clap(short, long, action = clap::ArgAction::Count)]
    insensitive: u8,

    /// parse unix timestamps (rows that start w/integers)
    #[clap(short, long, action = clap::ArgAction::Count)]
    unix_timestamp: u8,

    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum GrrMode {
    Count,
    Quiet,
    Insensitive,
    Unixtimestamp,
}

fn parse_unix_ts(timestamp: u64) -> String {
    // check whether the value fits into 32 bits. If not, divide by 1000. (millisecond)
    let tmp = match timestamp {
        0..=4294967295 => timestamp,
        4294967296.. => timestamp / 1000,
    };
    let d = UNIX_EPOCH + Duration::from_secs(tmp);
    let datetime = DateTime::<Utc>::from(d);
    let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string();

    timestamp_str
}

fn output_line_parse_unix_timestamp(line: &String, _mydict: &mut HashMap<String, u32>) {
    let words: Vec<&str> = line.split_whitespace().collect();
    let first0 = words.get(0).unwrap();
    let slice = &words[1..];

    // remove all non-numbers from the first word
    let first = first0.replace(":", "");

    // try to see if we can convert first to integer.
    match u64::from_str_radix(&first, 10) {
        Ok(number) => {
            println!("{} | {}", parse_unix_ts(number), slice.join(" "));
        }
        Err(_err) => {
            println!("{}", &words[0..].join(" "));
        }
    }
}

fn output_line(line: &String, _mydict: &mut HashMap<String, u32>) {
    let words: Vec<&str> = line.split_whitespace().collect();
    println!("{}", &words[0..].join(" "));
}

fn output_line_counter(line: &String, mydict: &mut HashMap<String, u32>) {
    let _words: Vec<&str> = line.split_whitespace().collect();
    let counter = &mydict.get_mut(&"count".to_string()); // acquire a reference
    let newval = match counter {
        Some(number) => **number + 1,
        None => 0,
    };
    let _ = &mydict.insert("count".to_string(), newval);
}

fn output_none(_line: &String, _mydict: &mut HashMap<String, u32>) {
    //println!("DEBUG: no output.");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // when in quiet mode, return OK only in case nothing was found.
    let args = Cli::parse();

    let mut modes_enabled = Vec::new();
    let mut mydict = HashMap::<String, u32>::new();
    mydict.insert("count".to_string(), 0);

    match args.count {
        0 => {}
        _ => {
            modes_enabled.push(GrrMode::Count);
        }
    }
    match args.quiet {
        0 => {}
        _ => modes_enabled.push(GrrMode::Quiet),
    };
    match args.insensitive {
        0 => {}
        _ => modes_enabled.push(GrrMode::Insensitive),
    };
    match args.unix_timestamp {
        0 => {}
        _ => modes_enabled.push(GrrMode::Unixtimestamp),
    };

    //println!("modes enabled: {:?}", modes_enabled);

    let mut output_fun: fn(&String, &mut HashMap<String, u32>) = output_line;

    if modes_enabled.contains(&GrrMode::Unixtimestamp) {
        output_fun = output_line_parse_unix_timestamp;
    }
    if modes_enabled.contains(&GrrMode::Count) {
        output_fun = output_line_counter;
    }

    // finally, look for Modes that will suppress normal line output
    if modes_enabled.contains(&GrrMode::Quiet) {
        output_fun = output_line_counter;
    }
    // TODO: create a output_fun "builder" that would create necessary function by currying

    let result = std::fs::read_to_string(&args.path); // read the whole file. TODO: switch to BufReader

    // modify the regex if doing insensitive matching
    let re = if modes_enabled.contains(&GrrMode::Insensitive) {
        let modified = r"(?i)".to_string() + &args.pattern.to_string();
        Regex::new(&modified).unwrap()
    } else {
        Regex::new(&args.pattern).unwrap()
    };

    let content = match result {
        Ok(stuff) => stuff,
        Err(error) => {
            return Err(error.into());
        }
    };

    for line in content.lines() {
        let m = re.is_match(&line.to_string());
        if m == true {
            {
                output_fun(&line.to_string(), &mut mydict); // pass ownership
            }
        }
    }

    let mut flagi = false;

    // for modes_enabled, drop those that do not affect the endgame.
    let index1 = modes_enabled
        .iter()
        .position(|&x| x == GrrMode::Insensitive); // look up the index
    match index1 {
        Some(idx) => {
            modes_enabled.remove(idx);
        }
        None => {}
    }

    for finish_mode in modes_enabled {
        let finish_fun = match finish_mode {
            GrrMode::Count => finish_count_mode,
            GrrMode::Quiet => finish_mode_returncode,
            GrrMode::Insensitive => finish_mode_noop,
            GrrMode::Unixtimestamp => finish_mode_noop,
        };
        //println!("debug: going to invoke {:?} mode finish", finish_mode);
        flagi = finish_fun(&mydict);
    }
    match flagi {
        true => Ok(()),
        false => {
            process::exit(1);
        }
    }
}

fn finish_mode_noop(_inventory: &HashMap<String, u32>) -> bool {
    // no operation
    true
}
fn finish_mode_returncode(inventory: &HashMap<String, u32>) -> bool {
    let count = &inventory.get("count").unwrap();
    match count {
        0 => false,
        _ => true,
    }
}

fn finish_count_mode(inventory: &HashMap<String, u32>) -> bool {
    //println!("DEBUG: inventory = {:?}", &inventory);
    let count = &inventory.get("count");
    println!("{}", count.unwrap());
    true
}
