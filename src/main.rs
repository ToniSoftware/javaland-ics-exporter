use chrono::naive;
use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};
use clap::Parser;
use ics::{
    properties::{Categories, DtEnd, DtStart, Location, Status, Summary, TzName},
    Event, ICalendar,
};
use regex::Regex;
use scraper::{Html, Selector};
use std::borrow::BorrowMut;
#[warn(dead_code)]
use std::error::Error;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Talk {
    title: String,
    speaker: String,
    date: naive::NaiveDate,
    begin_time: naive::NaiveTime,
    end_time: naive::NaiveTime,
    location: String,
}

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long, default_value_t = false, action = clap::ArgAction::Set)]
    one_file: bool,
}

fn get_body_of_webpage(url: String) -> Result<String, Box<dyn Error>> {
    let resp = reqwest::blocking::get(url)?.text()?;
    Ok(resp)
}

fn get_list_of_events(url: String) -> Vec<Talk> {
    let parsed_html = Html::parse_document(&get_body_of_webpage(url).unwrap());

    let selector = &Selector::parse(
        "div.agendaEventSlotTeaser.teaser.eventSlot.withLink > a.eventSlotLink > div.teaserWrapper",
    )
    .expect("Error during the parsing using the given selector");

    let div_text = parsed_html
        .select(selector)
        .flat_map(|el| el.text())
        .collect::<Vec<&str>>();

    let mut talks: Vec<Talk> = vec![];
    let mut i: usize = 0;
    while i < div_text.len() - 7 {
        talks.push(map_div_content_to_talk(i, &div_text));
        if String::from(div_text[i + 5]) == "Empfang Hotel LingBao" {
            i = i + 6;
            continue;
        }
        i = i + 7;
    }
    talks
}

fn map_div_content_to_talk(i: usize, div_text: &Vec<&str>) -> Talk {
    Talk {
        title: String::from(div_text[i]),
        speaker: String::from(div_text[i + 1]),
        date: NaiveDate::parse_from_str(
            String::from(div_text[i + 2]).split(" ").last().unwrap(),
            "%d.%m.%Y",
        )
        .unwrap(),
        begin_time: NaiveTime::parse_from_str(div_text[i + 3], "%H:%M").unwrap(),
        end_time: NaiveTime::parse_from_str(div_text[i + 4], "%H:%M").unwrap(),
        location: String::from(div_text[i + 5]),
    }
}

fn add_talk_to_calendar(talk: &Talk, mut calendar: &mut ICalendar) {
    let talk_clone = talk.clone();
    let mut event = Event::new(Uuid::new_v4().to_string(), "20230304T231000Z");
    event.push(Location::new(talk_clone.location));
    event.push(Status::confirmed());
    event.push(Categories::new("CONFERENCE"));
    event.push(Summary::new(format!("{} - {}", talk.title, talk.speaker)));
    event.push(TzName::new("UTC"));

    if talk_clone.begin_time.hour() < 10 {
        if talk_clone.begin_time.minute() < 10 {
            event.push(DtStart::new(format!(
                "{}0{}{}T0{}0{}00Z",
                talk.date.year(),
                talk.date.month(),
                talk.date.day(),
                talk.begin_time.hour() - 1,
                talk.begin_time.minute()
            )));
        } else {
            event.push(DtStart::new(format!(
                "{}0{}{}T0{}{}00Z",
                talk.date.year(),
                talk.date.month(),
                talk.date.day(),
                talk.begin_time.hour() - 1,
                talk.begin_time.minute()
            )));
        }
    } else {
        if talk_clone.begin_time.minute() < 10 {
            event.push(DtStart::new(format!(
                "{}0{}{}T{}0{}00Z",
                talk.date.year(),
                talk.date.month(),
                talk.date.day(),
                talk.begin_time.hour() - 1,
                talk.begin_time.minute()
            )));
        } else {
            event.push(DtStart::new(format!(
                "{}0{}{}T{}{}00Z",
                talk.date.year(),
                talk.date.month(),
                talk.date.day(),
                talk.begin_time.hour() - 1,
                talk.begin_time.minute()
            )));
        }
    }

    if talk_clone.end_time.hour() < 10 {
        if talk_clone.end_time.minute() < 10 {
            event.push(DtEnd::new(format!(
                "{}0{}{}T0{}0{}00Z",
                talk.date.year(),
                talk.date.month(),
                talk.date.day(),
                talk.end_time.hour() - 1,
                talk.end_time.minute()
            )));
        } else {
            event.push(DtEnd::new(format!(
                "{}0{}{}T0{}{}00Z",
                talk.date.year(),
                talk.date.month(),
                talk.date.day(),
                talk.end_time.hour() - 1,
                talk.end_time.minute()
            )));
        }
    } else {
        if talk_clone.end_time.minute() < 10 {
            event.push(DtEnd::new(format!(
                "{}0{}{}T{}0{}00Z",
                talk.date.year(),
                talk.date.month(),
                talk.date.day(),
                talk.end_time.hour() - 1,
                talk.end_time.minute()
            )));
        } else {
            event.push(DtEnd::new(format!(
                "{}0{}{}T{}{}00Z",
                talk.date.year(),
                talk.date.month(),
                talk.date.day(),
                talk.end_time.hour() - 1,
                talk.end_time.minute()
            )));
        }
    }
    calendar.add_event(event);
}

fn save_calendar_to_ics(calendar: &ICalendar, filename: String) -> Result<(), std::io::Error> {
    println!("saving file for {}.ics", filename);
    calendar.save_file(format!("{}.ics", filename))
}

fn format_filename_for_talk(talk: &Talk) -> String {
    let re = Regex::new("[ +:&\",!?.\n]").unwrap();
    format!(
        "{}/{}_{}_{}",
        talk.date.day(),
        re.replace_all(&talk.speaker, "_"),
        re.replace_all(&talk.title, "-"),
        re.replace_all(&talk.location, "-")
    )
}

fn main() {
    let args = Args::parse();

    if args.one_file {
        let mut calendar = ICalendar::new("2.0", "ics-rs");
        for mut talk in get_list_of_events(String::from(
            "https://shop.doag.org/events/javaland/2023/agenda/#eventDay.all",
        )) {
            add_talk_to_calendar(&talk, &mut calendar);
        }
        save_calendar_to_ics(&calendar, "javaland_2023_all_events".to_string());
    } else {
        for mut talk in get_list_of_events(String::from(
            "https://shop.doag.org/events/javaland/2023/agenda/#eventDay.all",
        )) {
            let mut calendar = ICalendar::new("2.0", "ics-rs");
            add_talk_to_calendar(&talk, &mut calendar);
            save_calendar_to_ics(&calendar, format_filename_for_talk(&talk));
        }
    }
}
