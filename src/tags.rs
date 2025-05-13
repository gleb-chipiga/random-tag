use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{create_dir_all, remove_file, File},
    io::{stdin, stdout, BufReader},
    iter::repeat_with,
    path::PathBuf,
};

use anyhow::{Context, Result};
use csv::ReaderBuilder;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use dirs::data_dir;
use itertools::Itertools;
use rand::{rng, seq::IndexedRandom};
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::command::Chars;

static TAGS_TABLE: TableDefinition<&str, i64> = TableDefinition::new("tags");

fn database_path() -> Result<PathBuf> {
    let data_dir = data_dir()
        .context("failed to get data_dir path")?
        .join(env!("CARGO_PKG_NAME"));
    if !data_dir.exists() {
        create_dir_all(&data_dir).context("failed to create data dir")?;
    }
    let db_path = data_dir.join("used-tags.redb");
    Ok(db_path)
}

fn database() -> Result<Database> {
    let db_path = database_path()?;
    Database::create(&db_path).with_context(|| format!("failed to create database {db_path:?}"))
}

fn create_table_if_not_exists(db: &Database) -> Result<()> {
    let write_txn = db.begin_write().context("failed to begin write transaction")?;
    write_txn
        .open_table(TAGS_TABLE)
        .context("failed to open table")?;
    write_txn.commit().context("failed to commit transaction")?;
    Ok(())
}

fn select_tag_index(tags: &[String]) -> Result<Option<usize>> {
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select one tag")
        .report(false)
        .items(tags)
        .interact_opt()
        .context("failed to select tag")
}

pub(crate) fn generate_tags(chars: Chars, length: usize, amount: usize) -> Result<()> {
    let db = database()?;
    let write_txn = db.begin_write().context("failed to begin write transaction")?;
    let mut table = write_txn
        .open_table(TAGS_TABLE)
        .with_context(|| format!("failed to open table {TAGS_TABLE}"))?;
    let max_repeats = length.div_ceil(chars.value.len()) as u8;
    let char_counts = {
        let mut char_counts = HashMap::with_capacity(chars.value.len());
        chars.value.iter().for_each(|char| {
            char_counts.insert(char, 0);
        });
        RefCell::new(char_counts)
    };
    let mut rng = rng();
    let tags = repeat_with(|| {
        repeat_with(|| chars.value.choose(&mut rng).unwrap())
            .take(length * 2)
            .filter(|char| *char_counts.borrow().get(char).unwrap() < max_repeats)
            .dedup()
            .inspect(|char| {
                *char_counts.borrow_mut().get_mut(char).unwrap() += 1;
            })
            .take(length)
            .collect()
    })
    .inspect(|_: &String| {
        char_counts
            .borrow_mut()
            .values_mut()
            .for_each(|value| *value = 0)
    })
    .filter(|tag| tag.len() == length)
    .filter(|tag| !chars.has_digit || tag.chars().any(|char| char.is_ascii_digit()))
    .filter(|tag| !chars.has_alphabetic || tag.chars().any(|char| char.is_ascii_alphabetic()))
    .filter_map(|tag| {
        table
            .get(tag.as_str())
            .with_context(|| format!("failed to get tag {tag} from table"))
            .map(|option| option.is_none().then_some(tag))
            .transpose()
    })
    .take(amount)
    .try_fold(
        Vec::with_capacity(amount),
        |mut tags, tag_result| -> Result<Vec<String>> {
            tags.push(tag_result?);
            Ok(tags)
        },
    )?;
    match tags.len() {
        1 => Some(tags.into_iter().next().unwrap()),
        _ => select_tag_index(&tags)?.map(|tag_index| tags.into_iter().nth(tag_index).unwrap()),
    }
    .into_iter()
    .try_for_each(|tag| -> Result<()> {
        table
            .insert(tag.as_str(), OffsetDateTime::now_utc().unix_timestamp())
            .with_context(|| format!("failed to insert tag {tag} to table"))?;
        println!("{tag}");
        Ok(())
    })?;
    drop(table);
    write_txn.commit().context("failed to commit write transaction")
}

fn serialize_timestamp<S>(timestamp: &i64, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let datetime_string = OffsetDateTime::from_unix_timestamp(*timestamp)
        .map_err(serde::ser::Error::custom)
        .and_then(|datetime| datetime.format(&Rfc3339).map_err(serde::ser::Error::custom))?;
    serializer.serialize_str(&datetime_string)
}

#[derive(Serialize)]
struct OutputRecord<'a> {
    tag: &'a str,
    #[serde(rename = "datetime", serialize_with = "serialize_timestamp")]
    timestamp: i64,
}

pub(crate) fn dump_tags() -> Result<()> {
    let db = database()?;
    create_table_if_not_exists(&db)?;
    let read_txn = db.begin_read().context("failed to begin read transaction")?;
    let table = read_txn
        .open_table(TAGS_TABLE)
        .with_context(|| format!("failed to open table {TAGS_TABLE}"))?;
    let mut csv_writer = csv::Writer::from_writer(stdout());
    table
        .iter()
        .context("failed to iterate tags table")?
        .try_for_each(|row_result| {
            row_result
                .context("failed to get tag table row")
                .and_then(|(tag, timestamp)| {
                    csv_writer
                        .serialize(OutputRecord {
                            tag: tag.value(),
                            timestamp: timestamp.value(),
                        })
                        .context("failed to write CSV record")
                })
        })?;
    Ok(())
}

fn deserialize_timestamp<'de, D>(deserializer: D) -> std::result::Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let datetime_string = String::deserialize(deserializer)?;
    OffsetDateTime::parse(&datetime_string, &Rfc3339)
        .map_err(serde::de::Error::custom)
        .map(|datetime| datetime.unix_timestamp())
}

#[derive(Deserialize)]
struct InputRecord {
    tag: String,
    #[serde(rename = "datetime", deserialize_with = "deserialize_timestamp")]
    timestamp: i64,
}

pub(crate) fn load_tags(path: Option<PathBuf>) -> Result<()> {
    let input: Box<dyn std::io::Read> = match path {
        Some(path) => Box::new(BufReader::new(
            File::open(path).context("failed to open input file")?,
        )),
        None => Box::new(stdin()),
    };
    let db = database()?;
    let write_txn = db.begin_write().context("failed to begin write transaction")?;
    {
        let mut table = write_txn
            .open_table(TAGS_TABLE)
            .with_context(|| format!("failed to open table {TAGS_TABLE}"))?;
        ReaderBuilder::new()
            .from_reader(input)
            .into_deserialize::<InputRecord>()
            .try_for_each(|record_result| -> Result<()> {
                let record = record_result.context("failed to read CSV record")?;
                table
                    .insert(record.tag.as_str(), record.timestamp)
                    .map(|_| ())
                    .with_context(|| format!("failed to insert tag {} to table", record.tag))
            })
            .context("failed to insert CSV records to table")?;
    }
    write_txn.commit().context("failed to commit write transaction")
}

pub(crate) fn check_db() -> Result<()> {
    let mut db = database()?;
    let passed = db
        .check_integrity()
        .context("failed to check database integrity")?;
    if passed {
        println!("Database is OK");
    } else {
        println!("Database was repaired");
    }
    create_table_if_not_exists(&db)?;
    db.compact().context("failed to compact database")?;
    Ok(())
}

pub(crate) fn drop_db() -> Result<()> {
    let db_path = database_path()?;
    if db_path.exists() {
        if Confirm::new()
            .with_prompt("Do you really want to drop database with used tags?")
            .interact()
            .context("failed to get confirm prompt answer")?
        {
            remove_file(&db_path).context("failed to drop database file")?;
            println!("Database was dropped");
        }
    } else {
        println!("Database not exists");
    }
    Ok(())
}
