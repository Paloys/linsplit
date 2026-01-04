mod linsplit_data;
mod livesplitone;
mod memory_reader;
mod split_reader;
mod splitter_logic;

use crate::linsplit_data::LinSplitData;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Path for the splits (.lss file) to read data from
    #[arg(short = 's', long = "splits", value_name = "PATH", required = true)]
    splits: String,

    /// Address to listen to, for LiveSplitOne to connect to.
    /// Default is 0.0.0.0, which accepts from any IP
    #[arg(
        short = 'a',
        long = "address",
        value_name = "ADDRESS",
        default_value = "0.0.0.0"
    )]
    address: String,

    /// Port to listen to, for LiveSplitOne to connect to.
    /// Default is 51000
    #[arg(
        short = 'p',
        long = "port",
        value_name = "PORT",
        default_value = "51000"
    )]
    port: String,

    /// Path to the folder containing the save data (files like 0.celeste),
    /// default is the Steam Linux default
    #[arg(
        short = 'f',
        long = "save-location",
        value_name = "PATH",
        default_value = "~/.local/share/Celeste/Saves/"
    )]
    save_location: String,
}

#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut data = LinSplitData::new(&args.splits, &format!("{}:{}", args.address, args.port), &args.save_location).await;
    data.main_loop().await;
}

#[cfg(not(target_os = "linux"))]
fn main() {
    compile_error!(
        "This project is only made for Linux. Use the actual LiveSplit if you're on Windows!"
    );
}
