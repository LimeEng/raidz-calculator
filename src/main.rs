use raidz::{Disk, Vdev, VdevType};
use std::{env, io};

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

fn print_configurations(target_storage: f64, configs: &mut [Vdev]) {
    configs.sort_by(|a, b| a.total_cost().partial_cmp(&b.total_cost()).unwrap());
    println!(
        "{:<20} {:<10} {:<20} {:<20} {:<20} {:<20} {:<20}",
        "Disk Size (TB)",
        "# Disks",
        "Usable Storage (TB)",
        "Raw Storage (TB)",
        "Total Cost",
        "Cost per usable TB",
        "Cost per raw TB"
    );
    println!(
        "{:<20} {:<10} {:<20} {:<20} {:<20} {:<20} {:<20}",
        "---------------",
        "-------",
        "-----------------",
        "-----------------",
        "-----------",
        "----------------",
        "--------------"
    );
    for config in configs {
        let deviation = config.usable_storage() - target_storage;

        let usable_storage = if deviation.abs() < f64::EPSILON {
            format!("{:.2}", config.usable_storage())
        } else {
            let percentage_deviation = (deviation.abs() / target_storage) * 100.0;
            let sign = if deviation > 0.0 { "+" } else { "-" };
            format!(
                "{:.2} ({sign}{:.2}%)",
                config.usable_storage(),
                percentage_deviation
            )
        };

        println!(
            "{:<20} {:<10} {:<20} {:<20} {:<20} {:<20} {:<20}",
            format!("{:.2}", config.disk_type.size),
            config.num_disks,
            usable_storage,
            format!("{:.2}", config.raw_storage()),
            format!("{:.2}", config.total_cost()),
            format!("{:.2}", config.total_cost() / config.usable_storage()),
            format!("{:.2}", config.total_cost() / config.raw_storage())
        );
    }
}

fn main() {
    let disks = [
        Disk {
            size: 2.0,
            cost: 1128.0,
        },
        Disk {
            size: 3.0,
            cost: 1502.0,
        },
        Disk {
            size: 4.0,
            cost: 1318.0,
        },
        Disk {
            size: 6.0,
            cost: 2050.0,
        },
        Disk {
            size: 8.0,
            cost: 2328.0,
        },
        Disk {
            size: 12.0,
            cost: 3110.0,
        },
    ];

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
            println!("Strategy {}", strategy.name());
            print_configurations(target_storage, &mut config);
            println!();
        }
    }
}
