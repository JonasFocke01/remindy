use notify_rust::Notification;
use std::env;
use colored::*;
use soloud::*;

use std::time;

fn main() {
    let mut args: Vec<String> = env::args().collect();

    args.remove(0);

    if args.len() == 0 {
        help(String::from("Too few arguments"));
    }

    match build_notification(&mut args) {
            Ok(_) => print!("\nNotification Triggered\n"),
            Err(error) => help(String::from(error))
        }
    

    
}

fn help(origin: String) {
    print!("\n");
    print!("{} {}\n"," Spawned help dialog because".bright_red(), origin.bright_red());
    print!("\n");
    print!("{}\n", "===================== Remindy ====================".green());
    print!("{}\n" , " Helpcenter. How to use Remindy".bright_green());
    print!("\n");
    print!("{}\n", " Just spawn a new notification like so:".bright_green());
    print!("\n");
    print!("{}\n", " remindy [NAME: string] [DURATION: number][MODIFIER: 'h' | 'm' | 's']".green());
    print!(" remindy testmeeting 15m\n");
    print!("\n");
    print!("{}\n", " This will spawn a new countdown which will notify you in 15 minutes.".bright_green());
    std::process::exit(0);
}

fn build_notification(options: &mut Vec<String>) -> Result<String, String> {
    let message = options.remove(0);
    let mut timer_length_string = options.remove(0);
    let modifier = match timer_length_string.pop() {
        Some(e) => e,
        None => return Err("Modifier not found!".to_string())
    };

    let mut timer_length_vec = timer_length_string.into_bytes();

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
        '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => return Err("no modifier found!".to_string()),
        e => return Err(format!("modifier unknown: {}!", e))
    };

    let mut interval_timestamp = time::Instant::now();

    print!("\n {}", "===================================================".blue());
    print!("\n {} {} {} {}:{}:{}\n", "Reminder:".green(), message.bright_red(), "started. I will remind you in:".green(), format!("{:02}", timer_length_in_ms/3_600_000).bright_red(), format!("{:02}", (timer_length_in_ms / 60_000) % 60).bright_red(), format!("{:02}", (timer_length_in_ms / 1000) % 60).bright_red());
    while timer_length_in_ms > 0 {
        if interval_timestamp.elapsed().as_secs() == 1 {
            print!("\r              {} {}:{}:{}   ", "Time left:".bright_green(), format!("{:02}", timer_length_in_ms/3_600_000).bright_red(), format!("{:02}", (timer_length_in_ms / 60_000) % 60).bright_red(), format!("{:02}", (timer_length_in_ms / 1000) % 60).bright_red());
            timer_length_in_ms -= 1000;
            interval_timestamp = time::Instant::now();
        }
    }
    
    print!("\n {}\n", "====================Time is up!====================".bright_red());
    Notification::new()
        .summary(message.as_str())
        .show().unwrap();

    let sl = Soloud::default().unwrap();
    let mut wav = audio::Wav::default();
    wav.load_mem(include_bytes!("../audio.mp3")).unwrap();
    sl.play(&wav);
    while sl.voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(String::new())
}