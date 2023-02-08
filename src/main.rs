use colored::*;
use notify_rust::Notification;
use soloud::*;
use std::env;
use std::io::Write;

use chrono::naive::NaiveTime;
use chrono::offset::Local;
use std::time;

use curl::easy::Easy;

const IOBROKER_IP: &str = "http://192.168.2.100:8087";
const PROGRESSBAR_LENGTH: usize = 28;

fn main() {
    let mut args: Vec<String> = env::args().collect();

    args.remove(0);

    if args.len() == 0 {
        help(String::from("Too few arguments"));
    }

    match parse_time_remaining(&mut args) {
        Ok(_) => print!("\nNotification Triggered\n"),
        Err(error) => help(String::from(error)),
    }
}

fn help(origin: String) {
    print!("\n");
    print!(
        "{} {}\n",
        " Spawned help dialog because".bright_red(),
        origin.bright_red()
    );
    print!("\n");
    print!(
        "{}\n",
        "===================== Remindy ====================".green()
    );
    print!("{}\n", " Helpcenter. How to use Remindy".bright_green());
    print!("\n");
    print!(
        "{}\n",
        " Just spawn a new notification like so:".bright_green()
    );
    print!("\n");
    print!(
        "{}\n",
        " remindy [NAME: string] [DURATION: number][MODIFIER: 'h' | 'm' | 's']".green()
    );
    print!(" remindy testmeeting 15m\n");
    print!("\n");
    print!(
        "{}\n",
        " This will spawn a new countdown which will notify you in 15 minutes.".bright_green()
    );
    print!("\n");
    print!(
        "{}\n",
        "======================== OR ======================".purple()
    );
    print!("\n");
    print!(
        "{}\n",
        " remindy [NAME: string] [TIME: [number][number]:[number][number]]".green()
    );
    print!(
        "{}\n",
        " optional parameter: '-d' to remind in X days".green()
    );
    print!(" remindy testmeeting 15:00 -d 1\n");
    print!("\n");
    print!(
        "{}\n",
        " This will spawn a new countdown which will notify you tomorrow at 15:00 O'Clock."
            .bright_green()
    );
    std::process::exit(0);
}

fn build_notification(message: String, timer_length_in_ms: &mut u64) {
    let initial_timer_length_in_ms = timer_length_in_ms.clone();
    let mut interval_timestamp = time::Instant::now();

    print!(
        "\n {}",
        "===================================================".blue()
    );

    // call iobroker
    let mut easy = Easy::new();
    easy.url(
        format!(
            "{}/set/0_userdata.0.endpoints.reminderMessage?value={}&prettyPrint",
            IOBROKER_IP, message
        )
        .as_str(),
    )
    .unwrap();
    easy.get(true).unwrap();
    let transfer = easy.transfer();
    transfer.perform().unwrap();

    let mut easy = Easy::new();
    easy.url(
        format!(
            "{}/set/0_userdata.0.endpoints.nextReminder?value={}&prettyPrint",
            IOBROKER_IP, timer_length_in_ms
        )
        .as_str(),
    )
    .unwrap();
    easy.get(true).unwrap();
    let transfer = easy.transfer();
    transfer.perform().unwrap();

    print!(
        "\n {} {} {} {}:{}:{}\n",
        "Reminder:".green(),
        message.bright_red(),
        "started.\n I will remind you in:".green(),
        format!("{:02}", *timer_length_in_ms / 3_600_000).bright_red(),
        format!("{:02}", (*timer_length_in_ms / 60_000) % 60).bright_red(),
        format!("{:02}", (*timer_length_in_ms / 1000) % 60).bright_red()
    );
    while *timer_length_in_ms > 1000 {
        if interval_timestamp.elapsed().as_secs() == 1 {
            interval_timestamp = time::Instant::now();
            let progress_bar_filler = vec![
                "=";
                ((initial_timer_length_in_ms - *timer_length_in_ms)
                    / (initial_timer_length_in_ms / PROGRESSBAR_LENGTH as u64))
                    as usize
            ]
            .join("");
            let progress_bar_space = vec![
                " ";
                PROGRESSBAR_LENGTH
                    - ((initial_timer_length_in_ms - *timer_length_in_ms)
                        / (initial_timer_length_in_ms / PROGRESSBAR_LENGTH as u64))
                        as usize
            ]
            .join("");

            print!(
                "\r {} {}:{}:{} [{}>{}]",
                "Time left:".bright_green(),
                format!("{:02}", *timer_length_in_ms / 3_600_000).bright_red(),
                format!("{:02}", (*timer_length_in_ms / 60_000) % 60).bright_red(),
                format!("{:02}", (*timer_length_in_ms / 1000) % 60).bright_red(),
                progress_bar_filler,
                progress_bar_space
            );
            *timer_length_in_ms -= 1000;
            std::io::stdout().flush().unwrap_or_default();
            std::thread::sleep(std::time::Duration::from_millis(
                (1000 - interval_timestamp.elapsed().as_millis()) as u64,
            ));
        }
    }

    print!(
        "\r {}\n",
        "====================Time is up!====================".bright_red()
    );
    std::io::stdout().flush().unwrap_or_default();
    Notification::new()
        .summary(message.as_str())
        .show()
        .unwrap();

    let sl = Soloud::default().unwrap();
    let mut wav = audio::Wav::default();
    wav.load_mem(include_bytes!("../audio.mp3")).unwrap();
    sl.play(&wav);
    while sl.voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn parse_time_remaining(options: &mut Vec<String>) -> Result<String, String> {
    if options.len() != 2 && (options.len() != 4) {
        return Err(format!(
            "Wrong parameters: {:?} expected testmeeting 15:00 -OR- testmeeting 15:00 -d 1",
            options
        ));
    }
    let mut timer_days_in_ms: u64 = 0;
    if options.len() == 4 {
        if options[2] == "-d" {
            let timer_days_as_bytes = options.pop().unwrap().into_bytes();
            for i in 0..(timer_days_as_bytes.len()) {
                timer_days_in_ms += (timer_days_as_bytes[i] as u64 - 48) * 86_400_000;
            }
            options.pop().unwrap();
        } else {
            return Err(format!("Wrong parameter: {}, expected '-d'", options[2]));
        }
    }

    let mut timer_input_string = options.remove(1);

    match timer_input_string.find(":") {
        Some(2) => {
            let mut timer_length_in_ms: u64 = 0;

            let timer_input_bytes = timer_input_string.into_bytes();

            timer_length_in_ms += (timer_input_bytes[0] as u64 - 48) * 36_000_000;
            timer_length_in_ms += (timer_input_bytes[1] as u64 - 48) * 3_600_000;
            timer_length_in_ms += (timer_input_bytes[3] as u64 - 48) * 600_000;
            timer_length_in_ms += (timer_input_bytes[4] as u64 - 48) * 60_000;

            let timestamp_target =
                NaiveTime::from_num_seconds_from_midnight_opt(timer_length_in_ms as u32 / 1000, 0)
                    .unwrap();
            let system_time_now = Local::now().time();
            let timestamps_difference =
                (timestamp_target - system_time_now).num_milliseconds() as u64;
            if timestamps_difference >= 86_400_000 {
                return Err("the desired time sould be in the future".to_string());
            }

            build_notification(
                options.pop().unwrap(),
                &mut (timestamps_difference + timer_days_in_ms),
            );
        }
        Some(_) => {
            return Err(format!(
                "Wrong format for {} expected hh:mm",
                timer_input_string
            ))
        }
        None => {
            let modifier = match timer_input_string.pop() {
                Some(e) => e,
                None => return Err("Modifier not found!".to_string()),
            };

            let mut timer_length_vec = timer_input_string.into_bytes();

            for e in timer_length_vec.iter_mut() {
                *e -= 48;
            }

            timer_length_vec.reverse();

            let mut weight = 1;
            let mut timer_length_in_ms: u64 = 0;

            for e in timer_length_vec.iter() {
                timer_length_in_ms += (*e as u64 * weight) as u64;
                weight *= 10;
            }

            timer_length_in_ms *= match modifier {
                's' => 1000,
                'm' => 60_000,
                'h' => 3_600_000,
                '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    return Err("no modifier found".to_string())
                }
                e => return Err(format!("modifier unknown: {}", e)),
            };

            build_notification(options.pop().unwrap(), &mut timer_length_in_ms);
        }
    }

    Ok("Success".to_string())
}
