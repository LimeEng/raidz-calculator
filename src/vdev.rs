use crate::Disk;

#[derive(Clone, Debug)]
pub enum VdevType {
    Raidz1,
    Raidz2,
    Raidz3,
}

impl VdevType {
    #[must_use]
    fn usable_storage(&self, disk_type: &Disk, num_disks: u32) -> f64 {
        assert!(self.min_disks() <= num_disks);
        match self {
            Self::Raidz1 => disk_type.size * f64::from(num_disks - 1),
            Self::Raidz2 => disk_type.size * f64::from(num_disks - 2),
            Self::Raidz3 => disk_type.size * f64::from(num_disks - 3),
        }
    }

    #[must_use]
    pub fn min_disks(&self) -> u32 {
        match self {
            Self::Raidz1 => 3,
            Self::Raidz2 => 4,
            Self::Raidz3 => 5,
        }
    }

    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Raidz1 => "RAID-Z1",
            Self::Raidz2 => "RAID-Z2",
            Self::Raidz3 => "RAID-Z3",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Vdev {
    pub vdev_type: VdevType,
    pub disk_type: Disk,
    pub num_disks: u32,
}

impl Vdev {
    #[must_use]
    pub fn new(vdev_type: VdevType, disk_type: Disk, num_disks: u32) -> Self {
        if vdev_type.min_disks() > num_disks {
            panic!(
                "{} requires at least {} disks",
                vdev_type.name(),
                vdev_type.min_disks()
            );
        }
        Self {
            vdev_type,
            disk_type,
            num_disks,
        }
    }

    #[must_use]
    pub fn usable_storage(&self) -> f64 {
        self.vdev_type
            .usable_storage(&self.disk_type, self.num_disks)
    }

    #[must_use]
    pub fn raw_storage(&self) -> f64 {
        self.disk_type.size * f64::from(self.num_disks)
    }

    #[must_use]
    pub fn total_cost(&self) -> f64 {
        self.disk_type.cost * f64::from(self.num_disks)
    }
}
