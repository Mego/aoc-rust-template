use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
    pub(crate) year: u16,
    pub(crate) day: u8,
    pub(crate) level: u8,
}
