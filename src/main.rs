use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime};
use clap::{Arg, ArgAction, Command};
use std::io::{stdin, stdout, Write};
use std::panic;

/** Extract hours, minutes, and seconds from String, return as vector of u32
 */
fn extract(input: &str) -> Vec<u32> {
    return input
        .split(":")
        .map(|x| x.parse::<u32>().unwrap())
        .collect();
}

/** Create DateTime object from vector of hours, minutes, and seconds [optional] in local timezone
 */
fn create_time(input: &str) -> DateTime<Local> {
    let now = Local::now();
    let offset = now.offset();
    let hm = extract(input);
    let dt: NaiveDateTime;
    match hm.len() {
        2 => {
            dt = NaiveDate::from_ymd(now.date().year(), now.date().month(), now.date().day())
                .and_hms(hm[0], hm[1], 0)
        }
        3 => {
            dt = NaiveDate::from_ymd(now.date().year(), now.date().month(), now.date().day())
                .and_hms(hm[0], hm[1], hm[2])
        }
        _ => panic!("Invalid format.  Stop!"),
    };
    let res = DateTime::<Local>::from_local(dt, *offset);
    return res;
}

/** Given two timestamps <HH:MM[:SS]>-<HH:MM[:SS]> extract the time between them and return as duration
 */
fn calculate_duration_from_string_ts(input: &String) -> Duration {
    let times_str: Vec<&str> = input.split("-").into_iter().collect();
    let start = create_time(times_str[0]);
    let end = create_time(times_str[1]);

    if end < start {
        return start - end;
    } else {
        return end - start;
    }
}

/** Sanitize stdin input, stripping trailing characters as needed
 */
fn sanitize_input(input: &mut String, c: char) {
    if c == input.chars().next_back().unwrap() {
        input.pop();
    }
}

/** Print duration struct in a human-readable way
 */
fn format_duration(input: &Duration) -> String {
    let res = format!(
        "{:02}:{:02}:{:02}",
        input.num_hours().abs(),
        (*input - Duration::hours(input.num_hours()))
            .num_minutes()
            .abs(),
        (*input - Duration::minutes(input.num_minutes()))
            .num_seconds()
            .abs()
    );
    return res;
}

fn main() {
    let m = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("Michael Lahnert <michael.lahnert@gmail.com>")
        .arg(
            Arg::new("starttime")
                .short('s')
                .help("Time when work started <HH:MM[:SS]>"),
        )
        .arg(
            Arg::new("breaks")
                .short('b')
                .num_args(1)
                .action(ArgAction::Append)
                .help("Break start and end <HH:MM[:SS]-HH:MM[:SS]>"),
        )
        .get_matches();

    let workday = Duration::hours(7) + Duration::minutes(48); // 7.8h
    let now: DateTime<Local> = Local::now();
    let break_short = Duration::minutes(30);
    let break_large = Duration::minutes(45);

    // Construct Start time.  From parameter or from commandline
    let start: DateTime<Local>;
    if let Some(start_s) = m.get_one::<String>("starttime") {
        start = create_time(start_s);
    } else {
        let mut user_input = String::new();
        print!("Enter start time [HH:MM]: ");
        let _ = stdout().flush();
        stdin()
            .read_line(&mut user_input)
            .expect("Bad string entered");
        sanitize_input(&mut user_input, '\n');
        sanitize_input(&mut user_input, '\r');
        start = create_time(&user_input);
    }

    let breaks_input = m.get_many::<String>("breaks");
    let mut breaks_s = Vec::new();
    match breaks_input {
        None => println!("No breaks defined, using default."),
        Some(x) => x.for_each(|s| breaks_s.push(s)),
    }

    let total_time = now - start;
    let mut break_time = Duration::seconds(0);
    let mut longest_break_time = Duration::seconds(0);
    if breaks_s.is_empty() {
        break_time = if total_time > (Duration::hours(9) + break_large) {
            break_large
        } else {
            break_short
        };
    } else {
        let breaks = breaks_s.iter();
        for break_ in breaks {
            let break_duration = calculate_duration_from_string_ts(break_);
            if break_duration > longest_break_time {
                longest_break_time = break_duration;
            }
            break_time = break_time + break_duration;
        }
    }

    let work_time = total_time - break_time;
    let done = work_time > workday;
    let remainder = if done {
        workday + break_short - total_time
    } else {
        total_time - (workday + break_short)
    };
    let text_rem = if done { "more" } else { "remaining" };
    let max_dur = (start + Duration::hours(10) + break_large) - now;

    println!(
        "[{}] start: {}; 7.8h: {}, 9h: {}, 10h: {}",
        now.format("%H:%M:%S"),
        start.time(),
        (start + workday + break_time).time(),
        (start + Duration::hours(9) + break_time).time(),
        (start + Duration::hours(10) + break_time).time()
    );
    println!(
        "           already done: {}; {} {}; no longer than {}",
        format_duration(&work_time),
        format_duration(&remainder),
        text_rem,
        format_duration(&max_dur)
    );
    println!(
        "           total break time: {}; longest break: {}",
        format_duration(&break_time),
        if longest_break_time == Duration::seconds(0) {
            format_duration(&break_time)
        } else {
            format_duration(&longest_break_time)
        }
    );
}
