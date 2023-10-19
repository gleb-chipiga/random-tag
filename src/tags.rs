use std::{cell::RefCell, collections::HashMap, fs::create_dir_all, io, iter::repeat_with};

use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use dirs::data_dir;
use itertools::Itertools;
use rand::{prelude::SliceRandom, thread_rng};
use redb::{Database, ReadableTable, TableDefinition};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::command::Chars;

static TAGS_TABLE: TableDefinition<&str, i64> = TableDefinition::new("tags");

fn database() -> Result<Database> {
    let data_dir = data_dir()
        .context("can't get data_dir path")?
        .join(env!("CARGO_PKG_NAME"));
    if !data_dir.exists() {
        create_dir_all(&data_dir).context("can't create data dir")?;
    }
    let db_path = data_dir.join("used-tags.redb");
    Database::create(&db_path).with_context(|| format!("can't create database {db_path:?}"))
}

pub(crate) fn generate_tags(chars: Chars, length: usize, amount: usize) -> Result<()> {
    let db = database()?;
    let write_txn = db.begin_write().context("can't begin write transaction")?;
    let mut table = write_txn
        .open_table(TAGS_TABLE)
        .with_context(|| format!("can't open table {TAGS_TABLE}"))?;
    let repeats = length.div_ceil(chars.value.len()) as u8;
    let char_counts = {
        let mut char_counts = HashMap::with_capacity(chars.value.len());
        chars.value.iter().for_each(|char| {
            char_counts.insert(char, 0);
        });
        RefCell::new(char_counts)
    };
    let mut rng = thread_rng();
    let tags = repeat_with(|| {
        repeat_with(|| chars.value.choose(&mut rng).unwrap())
            .take(length * 2)
            .filter(|char| *char_counts.borrow().get(char).unwrap() < repeats)
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
            .with_context(|| format!("can't get tag {tag} from table"))
            .map(|option| option.is_none().then_some(tag))
            .transpose()
    })
    .take(amount)
    .try_fold(Vec::with_capacity(amount), |mut tags, tag_result| {
        tags.push(tag_result?);
        Ok::<_, anyhow::Error>(tags)
    })?;
    let tag_index = match tags.len() {
        1 => Some(0),
        _ => Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select one tag")
            .report(false)
            .items(&tags)
            .interact_opt()
            .context("can't select tag")?,
    };
    if let Some(tag_index) = tag_index {
        table
            .insert(
                tags[tag_index].as_str(),
                OffsetDateTime::now_utc().unix_timestamp(),
            )
            .with_context(|| format!("can't insert tag {} to table", tags[tag_index]))?;
        println!("{}", tags[tag_index]);
    }
    drop(table);
    write_txn.commit().context("can't commit write transaction")
}

pub(crate) fn used_tags() -> Result<()> {
    let db = database()?;
    let read_txn = db.begin_read().context("can't begin read transaction")?;
    let table = read_txn
        .open_table(TAGS_TABLE)
        .with_context(|| format!("can't open table {TAGS_TABLE}"))?;
    let mut csv_writer = csv::Writer::from_writer(io::stdout());
    csv_writer
        .write_record(["tag", "datetime"])
        .context("can't write CSV header")?;
    table
        .iter()
        .context("can't iterate tags table")?
        .try_for_each(|row_result| {
            row_result
                .context("can't get tag table row")
                .and_then(|(tag, timestamp)| {
                    csv_writer
                        .write_record([
                            tag.value(),
                            OffsetDateTime::from_unix_timestamp(timestamp.value())
                                .context("can't decode unix timestamp")?
                                .format(&Rfc3339)
                                .context("can't format timestamp")?
                                .as_str(),
                        ])
                        .context("can't write CSV record")
                })
        })?;
    Ok(())
}
