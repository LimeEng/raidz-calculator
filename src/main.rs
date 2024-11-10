use colored::Colorize;
use raidz::{Disk, Vdev, VdevType};
use std::{env, io};
use tabled::{
    builder::Builder,
    settings::{style::BorderSpanCorrection, Style},
};

fn find_configurations(
    target_storage: f64,
    disk_options: &[Disk],
    max_disks: u32,
    range_ratio: f64,
    vdev_strategy: &VdevType,
) -> Vec<Vdev> {
    let min_storage = target_storage * (1.0 - range_ratio);
    let max_storage = target_storage * (1.0 + range_ratio);

    let mut config = Vec::new();

    for disk in disk_options {
        for num_disks in vdev_strategy.min_disks()..=max_disks {
            let vdev = Vdev::new(vdev_strategy.clone(), disk.clone(), num_disks);
            let usable_storage = vdev.usable_storage();
            if usable_storage >= min_storage && usable_storage <= max_storage {
                config.push(vdev);
            }
        }
    }

    config
}

fn print_configurations(header: &str, target_storage: f64, configs: &mut [Vdev]) {
    configs.sort_by(|a, b| a.total_cost().partial_cmp(&b.total_cost()).unwrap());

    let mut builder = Builder::default();

    let col_header: Vec<_> = [
        "Name",
        "Disk Size (TB)",
        "# Disks",
        "Usable Storage (TB)",
        "Raw Storage (TB)",
        "Total Cost",
        "Cost per usable TB",
        "Cost per raw TB",
    ]
    .iter()
    .map(|t| t.bold().to_string())
    .collect();

    builder.push_record(col_header);

    for config in configs {
        let deviation = config.usable_storage() - target_storage;

        let usable_storage = if deviation.abs() < f64::EPSILON {
            format!("{:.2}", config.usable_storage())
        } else {
            let percentage_deviation = (deviation.abs() / target_storage) * 100.0;
            let percentage_deviation = if deviation > 0.0 {
                format!("+{percentage_deviation:.2}%").green()
            } else {
                format!("-{percentage_deviation:.2}%").red()
            };

            format!("{:.2} ({percentage_deviation})", config.usable_storage())
        };

        let row = [
            config.disk_type.name.clone(),
            format!("{:.2}", config.disk_type.size),
            config.num_disks.to_string(),
            usable_storage,
            format!("{:.2}", config.raw_storage()),
            format!("{:.2}", config.total_cost()),
            format!("{:.2}", config.total_cost() / config.usable_storage()),
            format!("{:.2}", config.total_cost() / config.raw_storage()),
        ];
        builder.push_record(row);
    }
    let table = builder
        .build()
        .with(Style::sharp())
        .with(BorderSpanCorrection)
        .to_string();

    println!("{header}");
    println!("{table}");
}

fn main() {
    let json = include_str!("../disks.json");
    let disks: Vec<Disk> = serde_json::from_str(json).unwrap();

    let range_percentage = 30.0;
    let max_disks = 24;

    println!("RAID-Z configuration calculator");

    let result: Option<f64> = env::args()
        .collect::<Vec<_>>()
        .get(1)
        .and_then(|arg| arg.parse::<f64>().ok());

    let target_storage = if let Some(target) = result {
        target
    } else {
        println!("Enter your target usable storage in TB:");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        if let Ok(num) = input.trim().parse() {
            num
        } else {
            println!("Invalid input. Please enter a valid number.");
            return;
        }
    };

    println!();

    println!("Assuming a maximum of {max_disks} disks");

    let vdev_strategies = [VdevType::Raidz1, VdevType::Raidz2, VdevType::Raidz3];

    let configs: Vec<_> = vdev_strategies
        .iter()
        .map(|strategy| {
            (
                strategy,
                find_configurations(
                    target_storage,
                    &disks,
                    max_disks,
                    range_percentage / 100.0,
                    strategy,
                ),
            )
        })
        .collect();

    if configs.is_empty() {
        println!(
            "No configurations available within ±{range_percentage}% of {target_storage:.2} TB of usable storage."
        );
    } else {
        println!(
            "Configurations that are within ±{range_percentage}% of {target_storage:.2} TB of usable storage:"
        );
        println!();
        for (strategy, mut config) in configs {
            let header = format!("Strategy {}", strategy.name());
            print_configurations(&header, target_storage, &mut config);
            println!();
        }
    }
}
