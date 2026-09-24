mod data;
use anyhow::Result;
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

// constant with app name
const APP_NAME: &str = "decktags";
const DATA_AGE_WARNING: i64 = 14;

fn main() -> Result<()> {
    // Read command line arguments
    let cli = Cli::parse();

    // TODO: move this logic into a separate function, e.g. an impl of `App` type
    let app_data = data::AppData::new(APP_NAME)?;

    // If --update-data flag is passed or the files do not exist, download the data files into XDG_DATA_HOME
    // TODO: move part of this decision into appropriate methods
    let last_update = app_data.updated_at();
    if cli.update_data || last_update.is_none() {
        let newest_update = app_data.check_for_updates();
        if newest_update > last_update {
            println!("There is a newer update!");
            app_data.download_data()?; // TODO: give user option to continue
        } else {
            println!("There is no newer update!");
        }
    // Check age of data and warn if older than 14 days, but continue
    } else if let Some(date) = last_update
        && date < chrono::Utc::now() - chrono::TimeDelta::days(DATA_AGE_WARNING)
    {
        println!(
            "Warning: The data has last been updated at {}. You might want to update by passing --update-data flag.",
            date
        );
    }

    // parse given "deck" file into a Deck struct

    // Iterate over "oracle data" and find out the oracle ID for each card
    let oracle_cards = app_data.oracle_cards()?;

    // Iterate over the "tag data" and find out the tag IDs for each card
    let oracle_tags = app_data.oracle_tags()?;

    // Build data which outputs the "most-used" tags or "tag histogram" for the given deck.

    Ok(())
}
