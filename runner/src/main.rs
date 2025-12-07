use clap::Parser;

mod cli;
use cli::Cli;

fn run_year_day_part(year: u16, _day: u8, _level: u8, _input: &str) -> String {
    unimplemented!("year {year} has not been generated")
}

async fn run(year: u16, day: u8, level: u8) {
    let input = util::fetch_input(year, day).await;
    let answer = run_year_day_part(year, day, level, &input);
    let resp = util::submit_answer(year, day, level, &answer).await;
    println!("{resp}");
}

#[tokio::main]
pub async fn main() {
    let Cli { year, day, level } = Cli::parse();
    run(year, day, level).await;
}
