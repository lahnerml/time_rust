use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime};
use std::io::{stdin, stdout, Write};

fn extract(input: &String) -> Vec<u32> {
    return input
        .split(":")
        .map(|x| x.parse::<u32>().unwrap())
        .collect();
}

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
    let workday = Duration::hours(7) + Duration::minutes(48); // 7.8h
    let now: DateTime<Local> = Local::now();
    let offset = now.offset();
    let break_short = Duration::minutes(30);
    let break_large = Duration::minutes(45);

    // Construct Start time.  From parameter or from commandline
    let args: Vec<String> = std::env::args().collect();
    let start_t: Vec<u32>;
    if args.len() == 2 {
        start_t = extract(&args[1]);
    } else {
        let mut user_input = String::new();
        print!("Enter start time [HH:MM]: ");
        let _ = stdout().flush();
        stdin()
            .read_line(&mut user_input)
            .expect("Bad string entered");
        if let Some('\n') = user_input.chars().next_back() {
            user_input.pop();
        }
        if let Some('\r') = user_input.chars().next_back() {
            user_input.pop();
        }
        start_t = extract(&user_input);
    }

    let start_dt: NaiveDateTime =
        NaiveDate::from_ymd(now.date().year(), now.date().month(), now.date().day())
            .and_hms(start_t[0], start_t[1], 0);
    let start = DateTime::<Local>::from_local(start_dt, *offset);
    let total_work_time = now - start;
    let break_time = if total_work_time > (Duration::hours(9) + break_large) {
        break_large
    } else {
        break_short
    };
    let work_time = total_work_time - break_time;
    let done = work_time > workday;
    let remainder = if done {
        workday + break_short - total_work_time
    } else {
        total_work_time - (workday + break_short)
    };
    let text_rem = if done { "more" } else { "remaining" };
    let max_dur = (start + Duration::hours(10) + break_large) - now;

    println!(
        "[{}] start: {}; 7.8h: {}, 9h: {}, 10h: {}",
        now.format("%H:%M:%S"),
        start.time(),
        (start + workday + break_short).time(),
        (start + Duration::hours(9) + break_large).time(),
        (start + Duration::hours(10) + break_large).time()
    );
    println!(
        "           already done: {}; {} {}; no longer than {}",
        format_duration(&work_time),
        format_duration(&remainder),
        text_rem,
        format_duration(&max_dur)
    );
}
