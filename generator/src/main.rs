use std::{
    fs::File,
    io::Write,
    path::Path,
    process::Command,
    sync::{Arc, LazyLock},
};

use cargo_metadata::{Metadata, MetadataCommand};
use chrono::{Datelike, Utc};
use clap::Parser;
use itertools::Itertools;
use parking_lot::RwLock;
use rustc_hash::FxHashSet;
use upon::{Engine, Value, fmt};

static CURRENT_YEAR: LazyLock<i64> = LazyLock::new(|| Utc::now().year() as i64);

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// years to generate
    #[arg(value_parser = clap::value_parser!(u16).range(2015..=(*CURRENT_YEAR)))]
    years: Vec<u16>,
}

static METADATA: LazyLock<Metadata> = LazyLock::new(|| MetadataCommand::new().exec().unwrap());
static GENERATED_YEARS: LazyLock<Arc<RwLock<FxHashSet<u16>>>> = LazyLock::new(|| {
    Arc::new(RwLock::new(
        METADATA
            .workspace_packages()
            .iter()
            .filter(|&p| p.name.starts_with("aoc"))
            .map(|p| p.name[3..].parse().unwrap())
            .collect(),
    ))
});
static TEMPLATE_ENGINE: LazyLock<Engine> = LazyLock::new(|| {
    let mut engine = Engine::new();
    engine
        .add_template("day", include_str!("../templates/day_template.txt"))
        .unwrap();
    engine
        .add_template("lib", include_str!("../templates/lib_template.txt"))
        .unwrap();
    engine
        .add_template("runner", include_str!("../templates/runner_template.txt"))
        .unwrap();
    engine
        .add_template(
            "runner_cargo",
            include_str!("../templates/runner_cargo_template.txt"),
        )
        .unwrap();

    use std::fmt::Write;
    engine.add_formatter("day_fmt", |f, value| {
        match value {
            &Value::Integer(i) => write!(f, "day{i:02}")?,
            _ => fmt::default(f, value)?,
        };
        Ok(())
    });

    engine
});

fn year_package_already_generated(year: u16) -> bool {
    GENERATED_YEARS.read().contains(&year)
}

fn format_day_name(day: u8) -> String {
    format!("day{day:02}")
}

fn generate_year_package(year: u16) {
    if year_package_already_generated(year) {
        return;
    }
    let pkg_name = format!("aoc{year}");
    Command::new("cargo")
        .args(["new", &pkg_name])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    let mut lib_file = File::create(Path::new(&pkg_name).join("src").join("lib.rs")).unwrap();
    TEMPLATE_ENGINE
        .get_template("lib")
        .unwrap()
        .render(upon::value! {
            days: (if year >= 2025 { 1..=12 } else { 1..=25 })
                .collect::<Vec<_>>()
        })
        .to_writer(&mut lib_file)
        .unwrap();

    let mut cargo_file = File::options()
        .append(true)
        .open(Path::new(&pkg_name).join("Cargo.toml"))
        .unwrap();
    cargo_file
        .write_all(b"util = { path = \"../util\" }")
        .unwrap();

    GENERATED_YEARS.write().insert(year);
}

fn generate_days(year: u16) {
    let pkg_name = format!("aoc{year}");
    let max_day = if year >= 2025 { 12 } else { 25 };
    for day in 1..=max_day {
        let mut day_file = File::create(
            Path::new(&pkg_name)
                .join("src")
                .join(format!("{}.rs", format_day_name(day))),
        )
        .unwrap();
        TEMPLATE_ENGINE
            .get_template("day")
            .unwrap()
            .render(upon::value! {
                day: day,
                year: year
            })
            .to_writer(&mut day_file)
            .unwrap();
    }
}

fn generate_runner() {
    let years = GENERATED_YEARS
        .read()
        .iter()
        .copied()
        .sorted()
        .collect_vec();
    let ctx = upon::value! { years: years };

    let mut runner_cargo_file = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(Path::new("runner").join("Cargo.toml"))
        .unwrap();

    TEMPLATE_ENGINE
        .get_template("runner_cargo")
        .unwrap()
        .render(&ctx)
        .to_writer(&mut runner_cargo_file)
        .unwrap();

    let mut runner_file = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(Path::new("runner").join("src").join("main.rs"))
        .unwrap();

    TEMPLATE_ENGINE
        .get_template("runner")
        .unwrap()
        .render(&ctx)
        .to_writer(&mut runner_file)
        .unwrap();
}

fn generate(year: u16) {
    generate_year_package(year);
    generate_days(year);
    generate_runner();
    Command::new("cargo")
        .arg("fmt")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

pub fn main() {
    for year in Cli::parse().years {
        generate(year);
    }
}
