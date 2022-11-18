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
    print!(" remindy testmeeting 15\n");
    print!("\n");
    print!("{}\n", " This will spawn a new countdown which will notify you in 15 minutes.".bright_green());
    std::process::exit(0);
}

fn build_notification(options: &mut Vec<String>) -> Result<String, String> {
    let message = options.remove(0);
    let timer_length_string = options.remove(0);

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
    timer_length_in_ms *= 60_000;

    print!(" Timer started! I will remind you in: {}:{:02}", timer_length_in_ms/3_600_000, (timer_length_in_ms / 60_000) % 60);
    while timer_length_in_ms > 0 {
        print!("\r Reminder: '{}' running. Time left: {}:{:02}", message, timer_length_in_ms/3_600_000, (timer_length_in_ms / 60_000) % 60);
        timer_length_in_ms -= 1000;
        std::thread::sleep(time::Duration::from_secs(1));
    }

    print!("\n Time is up!\n");
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