use soroban_sdk::{Address, Env};

use crate::storage::types::error::Error;

use super::types::storage::DataKey;



pub(crate) fn has_admin(env: &Env) -> bool {
    let key = DataKey::Admin;

    env.storage().instance().has(&key)
}

pub(crate) fn read_admin(env: &Env) -> Result<Address, Error> {
    let key = DataKey::Admin;

    env.storage().instance().get(&key).ok_or(Error::AdminNotFound)
}


pub(crate) fn write_admin(env: &Env, admin: &Address) {
    let key = DataKey::Admin;

    env.storage().instance().set(&key, admin);
}

// Admin Fee functions
pub(crate) fn has_admin_fee(env: &Env) -> bool {
    let key = DataKey::AdminFee;
    env.storage().instance().has(&key)
}

pub(crate) fn read_admin_fee(env: &Env) -> i128 {
    let key = DataKey::AdminFee;
    env.storage().instance().get(&key).unwrap_or(0)
}

pub(crate) fn write_admin_fee(env: &Env, fee: &i128) {
    let key = DataKey::AdminFee;
    env.storage().instance().set(&key, fee);
}

// Admin Accumulated Fees functions
pub(crate) fn read_admin_accumulated_fees(env: &Env) -> i128 {
    let key = DataKey::AdminAccumulatedFees;
    env.storage().instance().get(&key).unwrap_or(0)
}

pub(crate) fn write_admin_accumulated_fees(env: &Env, amount: &i128) {
    let key = DataKey::AdminAccumulatedFees;
    env.storage().instance().set(&key, amount);
}