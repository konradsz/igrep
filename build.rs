use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

use anyhow::{ensure, Context, Result};

fn main() -> Result<()> {
    keybindings_table().context("failed to prepare keybindings table")?;

    Ok(())
}

fn keybindings_table() -> Result<()> {
    let readme = File::open("README.md").context("failed to open README.md")?;
    let readme = BufReader::new(readme);

    let table: Vec<String> = readme
        .lines()
        .skip_while(|line| !matches!(line, Ok(line) if line == "<!-- keybindings start -->"))
        .skip(3) // begin marker, header, separator line
        .take_while(|line| !matches!(line, Ok(line) if line == "<!-- keybindings end -->"))
        .collect::<std::result::Result<_, _>>()
        .context("failed to read table")?;

    ensure!(
        table
            .iter()
            .all(|line| line.starts_with('|') && line.ends_with('|') && line.contains(" | ")),
        "table is not a table"
    );

    let content: Vec<(String, String)> = table
        .into_iter()
        .map(|line| {
            line.strip_prefix('|')
                .unwrap()
                .strip_suffix('|')
                .unwrap()
                .to_string()
        })
        .map(|line| {
            let (keys, description) = line.split_once('|').unwrap();

            let keys = keys.trim().chars().filter(|c| c != &'`').collect();

            let description = description.trim().to_string();

            (keys, description)
        })
        .collect();

    let max_key = content
        .iter()
        .map(|(key, _)| key.len())
        .max()
        .context("no max key length")?
        .max("Key(s)".len());
    let max_description = content
        .iter()
        .map(|(_, description)| description.len())
        .max()
        .context("no max description length")?
        .max("Action".len());
    let len = content.len();

    let out_dir = std::env::var("OUT_DIR").context("no $OUT_DIR")?;

    let table_file = File::create(format!("{out_dir}/keybindings.txt"))
        .context("failed to create table file")?;
    let mut table_file = BufWriter::new(table_file);
    writeln!(
        table_file,
        "{0:<1$} │ {2:<3$}",
        "Key(s)", max_key, "Action", max_description
    )
    .context("failed to write table file: header")?;
    writeln!(
        table_file,
        "{}┼{}",
        "─".repeat(max_key + 1),
        "─".repeat(max_description + 1)
    )
    .context("failed to write table file: separator")?;
    for (key, description) in content {
        writeln!(
            table_file,
            "{key:<0$} │ {description:<1$}",
            max_key, max_description
        )
        .context("failed to write table file: content")?;
    }
    writeln!(table_file, "\nPress any key to close…")
        .context("failed to write table file: close")?;

    let data_file =
        File::create(format!("{out_dir}/keybindings.rs")).context("failed to create data file")?;
    let mut data_file = BufWriter::new(data_file);
    writeln!(
        data_file,
        r#"const KEYBINDINGS_TABLE: &str = include_str!(concat!(env!("OUT_DIR"), "/keybindings.txt"));"#
    )
    .context("failed to write data file: table")?;
    writeln!(data_file, "const KEYBINDINGS_LEN: u16 = {};", len + 4)
        .context("failed to write data file: length")?;
    writeln!(
        data_file,
        "const KEYBINDINGS_LINE_LEN: u16 = {};",
        max_key + 3 + max_description
    )
    .context("failed to write data file: line length")?;

    Ok(())
}
