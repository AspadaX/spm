use std::io::Write;

use anyhow::{Error, Result};
use console::style;
use prettytable::{Cell, Row, Table};

#[derive(Debug, Clone, Copy)]
pub enum Level {
    Logging,
    Error,
    Warn,
    Input,
}

pub fn display_message(level: Level, message: &str) {
    let indentation: String = ">> ".to_string();

    match level {
        Level::Logging => println!("{}{}", indentation, style(message).green()),
        Level::Error => println!("{}{}", indentation, style(message).red().bold()),
        Level::Warn => println!("{}{}", indentation, style(message).red()),
        Level::Input => print!("{}{} ", indentation, style(message).blue()),
    }
}

pub fn display_tree_message(indent_level: usize, message: &str) {
    let indentation: String = "\t".repeat(indent_level);
    println!("{}>> {}", indentation, style(message).green());
}

pub fn display_form(column_labels: Vec<&str>, rows: &Vec<Vec<String>>) {
    let mut table = Table::new();
    let top_line: Vec<Cell> = column_labels.iter().map(|item| Cell::new(item)).collect();
    table.add_row(Row::new(top_line));

    for row in rows {
        table.add_row(Row::new(row.iter().map(|item| Cell::new(item)).collect()));
    }

    table.printstd();
}

pub fn input_message(prompt: &str) -> Result<String, Error> {
    // display the prompt message for inputting values
    display_message(Level::Input, prompt);
    // collect the input as a string
    let mut input = String::new();
    // receive stdin
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;

    Ok(input)
}
