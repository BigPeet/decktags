mod data;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// path to the deck file
    deck: PathBuf,

    /// Flag to update the data files
    #[arg(short = 'u', long)]
    update_data: bool,
}

// TODO: add anyhow (for error handling in the app)
// TODO: add thiserror for custom error types in the "lib"

// constant with app name
const APP_NAME: &str = "decktags";

fn main() -> Result<(), std::io::Error> {
    // Read command line arguments
    let cli = Cli::parse();

    let mut data_dir = data::DataDirectory::ensure_exists(APP_NAME)?;
    let last_update = data_dir.updated_at();
    let two_weeks_ago = chrono::Utc::now().checked_sub_days(chrono::Days::new(14));
    let delta = chrono::TimeDelta::days(14);
    let now = chrono::Utc::now();
    println!("Updated at {}", last_update.unwrap());
    println!("Two weeks ago {}", two_weeks_ago.unwrap());
    println!("Two weeks ago {}", now - delta);

    // If --update-data flag is passed or the files do not exist, download the data files into XDG_DATA_HOME

    // Check age of data and warn if older than 14 days, but continue

    // parse given "deck" file into a Deck struct

    // Iterate over "oracle data" and find out the oracle ID for each card

    // Iterate over the "tag data" and find out the tag IDs for each card

    // Build data which outputs the "most-used" tags or "tag histogram" for the given deck.

    Ok(())
}
