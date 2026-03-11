use clap::{Parser, Subcommand, ValueHint};
use lamzu::{Atlantis, Mouse, Profile};
use std::fs::File;
use std::io::{self, stdin, Read};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "lamzu")]
#[command(about = "Lamzu mouse configuration tool", long_about = None)]
struct Cli {
    /// Force using untested devices
    #[arg(short, long)]
    force: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Read profile(s) from mouse and print
    Get {
        /// Output profile(s) in JSON instead of RON
        #[arg(short, long)]
        json: bool,

        /// Read from a specific profile by number
        #[arg(short, long)]
        profile: Option<usize>,
    },

    /// Write profile(s) to mouse
    Set {
        /// Input profile(s) in JSON instead of RON
        #[arg(short, long)]
        json: bool,

        /// Write to a specific profile by number
        #[arg(short, long)]
        profile: Option<usize>,

        /// Input profile configuration from file
        #[arg(group = "profile_in", short, long, value_hint = ValueHint::FilePath)]
        file: Option<PathBuf>,

        /// Input profile configuration
        #[arg(group = "profile_in")]
        config: Option<String>,
    },

    /// Get the active profile number on the mouse
    GetActive,

    /// Set the active profile number on the mouse
    SetActive {
        /// Active profile number to set
        profile_number: usize,
    },

    /// Get the battery charge percentage.
    GetBattery {
        /// Return raw millivolts instead of percent.
        #[arg(short, long)]
        millivolts: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let atlantis = Atlantis::connect(args.force).inspect_err(|e| match e {
        lamzu::Error::NoDevice => {
            eprintln!("No compatible devices found.");
        }
        lamzu::Error::UntestedDevice => {
            eprintln!(concat!(
                "No devices that have been tested with this tool have been found. ",
                "A device has been detected that may work, but has not been tested. ",
                "Use the `--force` to use untested devices. Use at your own risk."
            ));
        }
        _ => {}
    })?;

    eprintln!("You may need to move your mouse to wake it up...");

    match args.command {
        Command::Get { json, profile } => {
            if let Some(profile_number) = profile {
                // Profiles numbered from 1 for CLI.
                let profile = atlantis.profile(profile_number.saturating_sub(1))?;
                eprintln!("Profile {} retrieved from mouse:", profile_number);

                println!(
                    "{}",
                    if json {
                        serde_json::to_string_pretty(&profile)?
                    } else {
                        ron::ser::to_string_pretty(&profile, ron::ser::PrettyConfig::default())?
                    }
                );
            } else {
                let profiles = atlantis.profiles()?;
                eprintln!("All profiles retrieved from mouse:");
                println!(
                    "{}",
                    if json {
                        serde_json::to_string_pretty(&profiles)?
                    } else {
                        ron::ser::to_string_pretty(&profiles, ron::ser::PrettyConfig::default())?
                    }
                );
            }
        }

        Command::Set {
            json,
            profile,
            file,
            config,
        } => {
            // Test read for potentially untested devices to hopefully pick up any errors.
            if args.force {
                atlantis.profile(0)?;
            }

            let input = get_file_arg_or_stdin(file, config)?;
            if let Some(profile_number) = profile {
                let profile: Profile = if json {
                    serde_json::from_str(&input)?
                } else {
                    ron::de::from_str(&input).unwrap()
                };

                // Profiles numbered from 1 for CLI.
                atlantis.set_profile(profile_number.saturating_sub(1), &profile)?;
                eprintln!("Profile {} configured", profile_number);
            } else {
                let profiles: Vec<Profile> = if json {
                    serde_json::from_str(&input)?
                } else {
                    ron::de::from_str(&input).unwrap()
                };
                atlantis.set_profiles(&profiles)?;
                eprintln!("Profiles configured");
            }
        }

        Command::GetActive => {
            // Profiles numbered from 1 for CLI.
            let profile_number = atlantis.active_profile()? + 1;
            eprintln!("Active profile on mouse:");
            println!("{}", profile_number);
        }

        Command::SetActive { profile_number } => {
            // Profiles numbered from 1 for CLI.
            atlantis.set_active_profile(profile_number.saturating_sub(1))?;
            eprintln!("Set active profile to:");
            println!("{}", profile_number);
        }

        Command::GetBattery { millivolts } => {
            if millivolts {
                println!("{}", atlantis.battery_voltage()?);
            } else {
                println!("{}", atlantis.battery_percentage()?);
            }
        }
    }

    Ok(())
}

/// Returns input string from file, CLI argument, or from stdin, in that order.
fn get_file_arg_or_stdin(file: Option<PathBuf>, arg: Option<String>) -> io::Result<String> {
    let profile_text = if let Some(file_path) = file {
        let mut text = String::new();
        File::open(file_path)?.read_to_string(&mut text)?;
        text
    } else if let Some(text) = arg {
        text
    } else {
        let mut text = String::new();
        stdin().read_to_string(&mut text)?;
        text
    };

    Ok(profile_text)
}
