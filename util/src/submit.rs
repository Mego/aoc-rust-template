use std::{
    collections::HashMap,
    fs::{self, File, read_to_string},
    io::{BufWriter, Write},
    path::Path,
};

use reqwest::header::COOKIE;
use strip_tags::strip_tags;

const MY_COOKIE: &str = include_str!("../../cookie.txt");

const NO_COOKIE_ERROR: &str =
    "Puzzle inputs differ by user.  Please log in to get your puzzle input.";

pub async fn fetch_input(year: u16, day: u8) -> String {
    let fname = Path::new("inputs")
        .join(format!("{year}"))
        .join(format!("day{day}.txt"));
    fs::create_dir_all(fname.parent().unwrap()).unwrap();
    if let Ok(contents) = read_to_string(&fname) {
        return contents;
    }
    let url = format!("https://adventofcode.com/{}/day/{}/input", year, day);
    let data = reqwest::Client::new()
        .get(url)
        .header(COOKIE, MY_COOKIE)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    if data.trim() == NO_COOKIE_ERROR {
        panic!(
            "you need to set your cookie correctly in cookie.txt (including the `session=` part at the start)"
        );
    }
    let f = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&fname)
        .unwrap();
    let mut buf = BufWriter::new(f);
    write!(&mut buf, "{data}").unwrap();
    data
}

pub async fn submit_answer(year: u16, day: u8, level: u8, answer: &str) -> String {
    if let Some(check) = check(year, day, level, answer) {
        return if check {
            "correct (cached)"
        } else {
            "incorrect (cached)"
        }
        .to_string();
    }
    let level_str = level.to_string();
    let url = format!("https://adventofcode.com/{year}/day/{day}/answer");
    let form = HashMap::from([("level", level_str.as_str()), ("answer", answer)]);
    let resp = reqwest::Client::new()
        .post(url)
        .header(COOKIE, MY_COOKIE)
        .form(&form)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let start_idx = resp.find("<article><p>").unwrap() + "<article><p>".len();
    let end_idx = resp.find("</p></article>").unwrap();
    let clean_resp = strip_tags(resp[start_idx..end_idx].trim());
    if clean_resp.starts_with("That's the right answer!") {
        let fname = Path::new("solutions")
            .join(format!("{year}"))
            .join(format!("day{day}"))
            .join(format!("level{level}.txt"));
        let f = File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&fname)
            .unwrap();
        let mut buf = BufWriter::new(f);
        write!(&mut buf, "{answer}").unwrap();
    }
    clean_resp
}

pub fn check(year: u16, day: u8, level: u8, answer: &str) -> Option<bool> {
    let fname = Path::new("solutions")
        .join(format!("{year}"))
        .join(format!("day{day}"))
        .join(format!("level{level}.txt"));
    if let Ok(contents) = read_to_string(&fname) {
        Some(contents.trim() == answer)
    } else {
        None
    }
}
