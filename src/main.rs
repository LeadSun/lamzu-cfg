use clap::{Parser, Subcommand, ValueHint};
use hidapi::HidApi;
use lamzu_cfg::device::{device_compatibility, Atlantis, Compatibility, Mouse};
use lamzu_cfg::Profile;
use std::fs::File;
use std::io::{self, stdin, Read};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "lamzu-cfg")]
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
        /// Read from a specific profile by number
        #[arg(short, long)]
        profile: Option<usize>,
    },

    /// Write profile(s) to mouse
    Set {
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
}

fn main() -> lamzu_cfg::Result<()> {
    let args = Cli::parse();
    let api = HidApi::new()?;

    // First compatible device, prioritising tested over untested.
    let device_compat = device_compatibility(&api)
        .into_iter()
        .reduce(|acc, compat| match acc {
            Compatibility::Tested(_) => acc,
            Compatibility::Untested(_) => match compat {
                Compatibility::Tested(_) => compat,
                _ => acc,
            },
            Compatibility::Incompatible(_) => compat,
        })
        .expect("No USB devices found.");

    let (device, tested) = match device_compat {
        Compatibility::Tested(device) => (device, true),
        Compatibility::Untested(device) => {
            if args.force {
                eprintln!("Warning: Using an untested device.");
                (device, false)
            } else {
                eprintln!(concat!(
                    "No devices that have been tested with this tool have been found. ",
                    "A device has been detected that may work, but has not been tested. ",
                    "Use the `--force` to use untested devices. Use at your own risk."
                ));
                return Err(lamzu_cfg::Error::NoDevice);
            }
        }
        Compatibility::Incompatible(_) => {
            eprintln!("No compatible devices found.");
            return Err(lamzu_cfg::Error::NoDevice);
        }
    };

    eprintln!(
        "Using device: {} {}",
        device
            .get_manufacturer_string()?
            .unwrap_or("Unknown".to_string()),
        device
            .get_product_string()?
            .unwrap_or("Unknown".to_string())
    );
    eprintln!("You may need to move your mouse to wake it up...");

    match args.command {
        Command::Get { profile } => {
            if let Some(profile_number) = profile {
                // Profiles numbered from 1 for CLI.
                let profile = Atlantis.profile(&device, profile_number.saturating_sub(1))?;
                eprintln!("Profile {} retrieved from mouse:", profile_number);
                println!(
                    "{}",
                    ron::ser::to_string_pretty(&profile, ron::ser::PrettyConfig::default())?
                );
            } else {
                let profiles = Atlantis.profiles(&device)?;
                eprintln!("All profiles retrieved from mouse:");
                println!(
                    "{}",
                    ron::ser::to_string_pretty(&profiles, ron::ser::PrettyConfig::default())?
                );
            }
        }

        Command::Set {
            profile,
            file,
            config,
        } => {
            // Test read for untested devices to pick up any errors.
            if !tested {
                Atlantis.profile(&device, 0)?;
            }

            if let Some(profile_number) = profile {
                let profile: Profile =
                    ron::de::from_str(&get_file_arg_or_stdin(file, config)?).unwrap();

                // Profiles numbered from 1 for CLI.
                Atlantis.set_profile(&device, profile_number.saturating_sub(1), &profile)?;
                eprintln!("Profile {} configured", profile_number);
            } else {
                let profiles: Vec<Profile> =
                    ron::de::from_str(&get_file_arg_or_stdin(file, config)?).unwrap();
                Atlantis.set_profiles(&device, &profiles)?;
                eprintln!("Profiles configured");
            }
        }

        Command::GetActive => {
            // Profiles numbered from 1 for CLI.
            let profile_number = Atlantis.active_profile_index(&device)? + 1;
            eprintln!("Active profile on mouse:");
            println!("{}", profile_number);
        }

        Command::SetActive { profile_number } => {
            // Profiles numbered from 1 for CLI.
            Atlantis.set_active_profile_index(&device, profile_number.saturating_sub(1))?;
            eprintln!("Set active profile to:");
            println!("{}", profile_number);
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
