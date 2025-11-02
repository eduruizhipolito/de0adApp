use soroban_sdk::{contracttype};

#[derive(Clone, PartialEq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum CarStatus {
    Available, // 0 -> 1 byte
    Rented, // 1 -> 1 byte
    Maintenance, // 2 -> 1 byte
}