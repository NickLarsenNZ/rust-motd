use crate::command::BetterCommand;
use bytesize::ByteSize;
use itertools::Itertools;
use std::cmp;
use std::collections::HashMap;
use std::iter;
use systemstat::{Filesystem, Platform, System};
use termion::{color, style};
use thiserror::Error;

use crate::constants::{GlobalSettings, INDENT_WIDTH};

#[derive(Error, Debug)]
pub enum FilesystemsError {
    #[error("Empty configuration for filesystems. Please remove the entire block to disable this component.")]
    ConfigEmtpy,

    #[error("Could not find mount {mount_point:?}")]
    MountNotFound { mount_point: String },

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

#[derive(Debug)]
struct Entry<'a> {
    filesystem_name: String,
    dev: &'a str,
    mount_point: &'a str,
    fs_type: &'a str,
    used: String,
    total: String,
    used_ratio: f64,
}

pub type FilesystemsCfg = HashMap<String, String>;

fn parse_into_entry(filesystem_name: String, mount: &Filesystem) -> Entry {
    let total = mount.total.as_u64();
    let avail = mount.avail.as_u64();
    let used = total - avail;

    Entry {
        filesystem_name,
        mount_point: &mount.fs_mounted_on,
        dev: &mount.fs_mounted_from,
        fs_type: &mount.fs_type,
        used: ByteSize::b(used).to_string(),
        total: ByteSize::b(total).to_string(),
        used_ratio: (used as f64) / (total as f64),
    }
}

fn print_row<'a>(items: [&str; 6], column_sizes: impl IntoIterator<Item = &'a usize>) {
    println!(
        "{}",
        Itertools::intersperse(
            items
                .iter()
                .zip(column_sizes.into_iter())
                .map(|(name, size)| format!("{: <size$}", name, size = size)),
            " ".repeat(INDENT_WIDTH)
        )
        .collect::<String>()
    );
}

fn get_scrub_status(entry: &Entry) -> Option<String> {
    if entry.fs_type != "btrfs" {
        return None;
    }

    let output = BetterCommand::new("sudo")
        .arg("btrfs")
        .arg("scrub")
        .arg("status")
        .arg("/")
        .check_status_and_get_output_string()
        .ok()?;
    let things: HashMap<&str, &str> = output
        .lines()
        .filter_map(|l| l.splitn(2, ':').map(|x| x.trim()).collect_tuple())
        .collect();
    let status = *things.get("Status")?;
    let status = match status {
        "finished" => *things.get("Scrub started")?,
        "running" => "in progress...",
        _ => status,
    };
    Some(format!(
        "Last scrub: {} ({})",
        status,
        if *things.get("Error summary")? == "no errors found" {
            format!("{}✓{}", color::Fg(color::Green), style::Reset)
        } else {
            format!("{}✕{}", color::Fg(color::Red), style::Reset)
        }
    ))
}

pub fn disp_filesystem(
    config: FilesystemsCfg,
    global_settings: &GlobalSettings,
    sys: &System,
) -> Result<Option<usize>, FilesystemsError> {
    if config.is_empty() {
        return Err(FilesystemsError::ConfigEmtpy);
    }

    let mounts = sys.mounts()?;
    let mounts: HashMap<String, &Filesystem> = mounts
        .iter()
        .map(|fs| (fs.fs_mounted_on.clone(), fs))
        .collect();

    let entries = config
        .into_iter()
        .map(
            |(filesystem_name, mount_point)| match mounts.get(&mount_point) {
                Some(mount) => Ok(parse_into_entry(filesystem_name, mount)),
                _ => Err(FilesystemsError::MountNotFound { mount_point }),
            },
        )
        .collect::<Result<Vec<Entry>, FilesystemsError>>()?;

    let header = ["Filesystems", "Device", "Mount", "Type", "Used", "Total"];

    let column_sizes = entries
        .iter()
        .map(|entry| {
            vec![
                entry.filesystem_name.len() + INDENT_WIDTH,
                entry.dev.len(),
                entry.mount_point.len(),
                entry.fs_type.len(),
                entry.used.len(),
                entry.total.len(),
            ]
        })
        .chain(iter::once(header.iter().map(|x| x.len()).collect()))
        .fold(vec![0; header.len()], |acc, x| {
            x.iter()
                .zip(acc.iter())
                .map(|(a, b)| cmp::max(a, b).to_owned())
                .collect()
        });

    print_row(header, &column_sizes);

    // -2 because "Filesystems" does not count (it is not indented)
    // and because zero indexed
    let bar_width = column_sizes.iter().sum::<usize>() + (header.len() - 2) * INDENT_WIDTH
        - global_settings.progress_prefix.len()
        - global_settings.progress_suffix.len();
    let fs_display_width =
        bar_width + global_settings.progress_prefix.len() + global_settings.progress_suffix.len();

    for entry in entries {
        let bar_full = ((bar_width as f64) * entry.used_ratio) as usize;
        let bar_empty = bar_width - bar_full;

        print_row(
            [
                &[" ".repeat(INDENT_WIDTH), entry.filesystem_name.to_owned()].concat(),
                entry.dev,
                entry.mount_point,
                entry.fs_type,
                entry.used.as_str(),
                entry.total.as_str(),
            ],
            &column_sizes,
        );

        let full_color = match (entry.used_ratio * 100.0) as usize {
            0..=75 => color::Fg(color::Green).to_string(),
            76..=95 => color::Fg(color::Yellow).to_string(),
            _ => color::Fg(color::Red).to_string(),
        };

        println!(
            "{}",
            [
                " ".repeat(INDENT_WIDTH),
                global_settings.progress_prefix.to_string(),
                full_color,
                global_settings
                    .progress_full_character
                    .to_string()
                    .repeat(bar_full),
                color::Fg(color::LightBlack).to_string(),
                global_settings
                    .progress_empty_character
                    .to_string()
                    .repeat(bar_empty),
                style::Reset.to_string(),
                global_settings.progress_suffix.to_string(),
            ]
            .join("")
        );

        if let Some(scrub_status) = get_scrub_status(&entry) {
            println!("{}", scrub_status);
        } else {
            println!("Fuck");
        }
    }

    Ok(Some(fs_display_width))
}
