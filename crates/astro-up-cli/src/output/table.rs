// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

use color_eyre::eyre::Result;
use tabled::{Table, Tabled, settings::Style};

pub fn print_table<T: Tabled>(rows: &[T]) -> Result<()> {
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{table}");
    Ok(())
}
