use serde::{Deserialize, Serialize};

mod vdev;

pub use vdev::{Vdev, VdevType};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Disk {
    pub size: f64,
    pub cost: f64,
    pub name: String,
    pub link: String,
}
