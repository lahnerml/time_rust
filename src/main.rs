use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime};
use clap::{Arg, ArgAction, Command};
use std::{cmp::max, panic};

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
    let tz = now.timezone();
    let hm = extract(input);
    let dt: NaiveDateTime;
    match hm.len() {
        2 => {
            dt = NaiveDate::from_ymd_opt(
                now.date_naive().year(),
                now.date_naive().month(),
                now.date_naive().day(),
            )
            .unwrap()
            .and_hms_opt(hm[0], hm[1], 0)
            .unwrap()
        }
        3 => {
            dt = NaiveDate::from_ymd_opt(
                now.date_naive().year(),
                now.date_naive().month(),
                now.date_naive().day(),
            )
            .unwrap()
            .and_hms_opt(hm[0], hm[1], hm[2])
            .unwrap()
        }
        _ => panic!("Invalid format.  Stop!"),
    };
    let res = dt.and_local_timezone(tz).single().unwrap();
    return res;
}

fn create_duration(input: &str) -> Duration {
    let times_str: Vec<i64> = input
        .split(":")
        .map(|x| x.parse::<i64>().unwrap())
        .collect();
    let res: Duration;
    match times_str.len() {
        2 => {
            res = Duration::hours(times_str[0]) + Duration::minutes(times_str[1]);
        }
        3 => {
            res = Duration::hours(times_str[0])
                + Duration::minutes(times_str[1])
                + Duration::seconds(times_str[2]);
        }
        _ => panic!("Invalid format.  Stop!"),
    }
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

fn format_duration_hours(input: &Duration) -> String {
    let res = format!(
        "{:.2}",
        input.num_hours().abs() as f64
            + (*input - Duration::hours(input.num_hours()))
                .num_minutes()
                .abs() as f64
                / 60.
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
                .required(true)
                .help("Time when work started <HH:MM[:SS]>"),
        )
        .arg(
            Arg::new("endtime")
                .short('e')
                .help("Time when work ended <HH:MM[:SS]>"),
        )
        .arg(
            Arg::new("daily-goal")
                .short('d')
                .help("Daily work goal <HH:MM[:SS]>"),
        )
        .arg(
            Arg::new("weekly-goal")
                .short('w')
                .default_value("39:00")
                .help("Weekly work goal <HH:MM[:SS]>"),
        )
        .arg(
            Arg::new("breaks")
                .short('b')
                .num_args(1)
                .action(ArgAction::Append)
                .help("Break start and end <HH:MM[:SS]-HH:MM[:SS]>"),
        )
        .get_matches();

    let now: DateTime<Local> = Local::now();
    let break_short = Duration::minutes(30);
    let break_large = Duration::minutes(45);

    // Build start and end time from commandline
    let start: DateTime<Local>;
    if let Some(start_s) = m.get_one::<String>("starttime") {
        start = create_time(start_s);
    } else {
        panic!("Start time not defined");
    }

    let mut end = DateTime::<Local>::default();
    if let Some(end_s) = m.get_one::<String>("endtime") {
        end = create_time(end_s);
    }

    // Build daily worktime goal
    let workday: Duration;
    if let Some(workday_s) = m.get_one::<String>("daily-goal") {
        workday = create_duration(workday_s);
    } else if let Some(workweek_s) = m.get_one::<String>("weekly-goal") {
        workday = create_duration(workweek_s) / 5;
    } else {
        panic!("Working-hour goal undefined")
    }

    // Build breaks
    let breaks_input = m.get_many::<String>("breaks");
    let mut breaks_s = Vec::new();
    match breaks_input {
        None => println!("No breaks defined, using default."),
        Some(x) => x.for_each(|s| breaks_s.push(s)),
    }

    let total_time: Duration;
    if end != DateTime::<Local>::default() {
        total_time = end - start;
    } else {
        total_time = now - start;
    }
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
        workday + break_time - total_time
    } else {
        total_time - (workday + break_time)
    };
    let text_rem = if done { "more" } else { "remaining" };
    let max_dur = (start + Duration::hours(10) + max(break_large, break_time)) - now;

    let mut end_time_str: String = "".to_owned();
    if end != DateTime::<Local>::default() {
        end_time_str.push_str("end: ");
        end_time_str.push_str(&end.time().to_string());
        end_time_str.push_str("; ");
    }

    println!(
        "[{}] start: {}; {}{}h: {}, 9h: {}, 10h: {}",
        now.format("%H:%M:%S"),
        start.time(),
        end_time_str,
        format_duration_hours(&workday),
        (start + workday + break_time).time(),
        (start + Duration::hours(9) + max(break_large, break_time)).time(),
        (start + Duration::hours(10) + max(break_large, break_time)).time()
    );
    println!(
        "           already done: {} [{}]; {} [{}] {}; no longer than {}",
        format_duration(&work_time),
        format_duration_hours(&(work_time)),
        format_duration(&remainder),
        format_duration_hours(&remainder),
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
    if end != DateTime::<Local>::default() {
        println!(
            "           total hours worked: {}",
            format_duration_hours(&(total_time - break_time))
        );
    }
}
