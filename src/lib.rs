mod vdev;

pub use vdev::{Vdev, VdevType};

#[derive(Clone, Debug)]
pub struct Disk {
    pub size: f64,
    pub cost: f64,
}
