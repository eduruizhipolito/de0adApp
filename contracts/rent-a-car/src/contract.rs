use soroban_sdk::{contract, contractimpl, Address, Env};
use crate::{events, interfaces::contract::RentACarContractTrait, methods::{public::get_car_status::get_car_status,token::token::token_transfer}, storage::{admin::{has_admin, read_admin, write_admin, read_admin_fee, write_admin_fee, read_admin_accumulated_fees, write_admin_accumulated_fees}, car::{has_car, read_car, remove_car, write_car}, contract_balance::{read_contract_balance, write_contract_balance}, rental::write_rental, structs::{car::Car, rental::Rental}, token::write_token, types::{car_status::CarStatus, error::Error}}};

#[contract]
pub struct RentACarContract;

#[contractimpl]
impl RentACarContractTrait for RentACarContract {

    fn __constructor(env: &Env, admin: Address, token: Address) -> Result<(), Error> {

        if admin == token {
            return Err(Error::AdminTokenConflict);
        }

        if has_admin(&env) {
            return Err(Error::ContractInitialized);
        }
        
        write_admin(env,&admin);
        write_token(env,&token);

        events::contract::contract_initialized(env, admin, token);
        Ok(())
    }


    fn add_car(env: &Env, owner: Address, price_per_day: i128) -> Result<(), Error> {
        
        let admin = read_admin(env)?;
        admin.require_auth();

        if price_per_day <= 0 {
            return Err(Error::AmountMustBePositive);
        }

        if has_car(env, &owner) {
            return Err(Error::CarAlreadyExist);
        }

        let car = Car {
            price_per_day,
            car_status: CarStatus::Available,
            available_to_withdraw: 0,
        };

        write_car(env, &owner, &car);

        events::add_car::car_added(env, owner, price_per_day);
        Ok(())
    }

    fn get_car_status(env: &Env, owner: Address) -> Result<CarStatus, Error> {
              
        get_car_status(env, &owner)
    }

    fn rental(env: &Env, renter: Address, owner: Address, total_days_to_rent: u32, amount: i128) -> Result<(), Error> {
        // Validaciones -> Lógica -> Almacenamiento -> Eventos -> Resultado

        // Validaciones
        // Paso 1. Validar autorización de quien llama la función        
        renter.require_auth();

        // Paso 2. Validar parámetros de ingreso de la función        
        if amount < 0 {
            return Err(Error::AmountMustBePositive);
        }

        if total_days_to_rent == 0 {
            return Err(Error::RentalDurationCannotBeZero);
        }

        if renter == owner {
            return Err(Error::SelfRentalNotAllowed);
        }

        // Paso 3. Validar almacenamiento
        let mut car = read_car(env, &owner)?; // fees

        if car.car_status != CarStatus::Available {
            return Err(Error::CarAlreadyRented);
        }

        // Lógica
        // Leer la comisión del administrador
        let admin_fee = read_admin_fee(env);
        
        // Calcular el depósito del owner (amount ya incluye la comisión)
        // El frontend debe enviar: (precio_por_día * días) + admin_fee
        let owner_deposit = amount.checked_sub(admin_fee).ok_or(Error::UnderflowError)?;
        
        if owner_deposit < 0 {
            return Err(Error::AmountMustBePositive);
        }

        // Transferir el monto total al contrato
        token_transfer(&env, &renter, &env.current_contract_address(), &amount)?;

        // Actualizar el auto
        car.car_status = CarStatus::Rented;
        car.available_to_withdraw = car.available_to_withdraw.checked_add(owner_deposit).ok_or(Error::OverflowError)?;

        let rental = Rental {
            total_days_to_rent,
            amount: owner_deposit, // Guardamos solo el depósito del owner
        };
        
        // Actualizar balance del contrato
        let mut contract_balance = read_contract_balance(&env);
        contract_balance = contract_balance.checked_add(amount).ok_or(Error::OverflowError)?;

        // Acumular la comisión del admin
        let mut admin_accumulated = read_admin_accumulated_fees(env);
        admin_accumulated = admin_accumulated.checked_add(admin_fee).ok_or(Error::OverflowError)?;

       // Almacenamiento
       write_contract_balance(&env, &contract_balance);
       write_admin_accumulated_fees(env, &admin_accumulated);
       write_car(env, &owner, &car);
       write_rental(env, &renter, &owner, &rental);
       
       // Eventos
       events::rental::rented(env, renter, owner, total_days_to_rent, amount);

       // Resultado
       Ok(())
    }

    fn payout_owner(env: &Env, owner: Address, amount: i128) -> Result<(), Error> {
        owner.require_auth();

        if amount <= 0 {
            return Err(Error::AmountMustBePositive);
        }       

        let mut car = read_car(&env, &owner)?;

        if amount > car.available_to_withdraw {
            return Err(Error::InsufficientBalance);
        }

        let mut contract_balance = read_contract_balance(&env);

        if amount > contract_balance {
            return Err(Error::BalanceNotAvailableForAmountRequested);
        }

        token_transfer(&env, &env.current_contract_address(), &owner, &amount)?;

        car.available_to_withdraw = car.available_to_withdraw.checked_sub(amount).ok_or(Error::UnderflowError)?;
        contract_balance = contract_balance.checked_sub(amount).ok_or(Error::UnderflowError)?;

        write_car(&env, &owner, &car);
        write_contract_balance(&env, &contract_balance);
        
        events::payout_owner::payout_owner(env, owner, amount);
        Ok(())
    }

    fn remove_car(env: &Env, owner: Address) -> Result<(), Error> {
        
        let admin = read_admin(env)?;
        admin.require_auth();
        
        if !has_car(env, &owner) {
            return Err(Error::CarNotFound);
        }

        remove_car(env, &owner);

        events::remove_car::car_removed(env, owner);
        Ok(())
    }

    fn set_admin_fee(env: &Env, admin: Address, fee: i128) -> Result<(), Error> {
        let stored_admin = read_admin(env)?;
        stored_admin.require_auth();

        if admin != stored_admin {
            return Err(Error::AdminNotFound);
        }

        if fee < 0 {
            return Err(Error::AmountMustBePositive);
        }

        write_admin_fee(env, &fee);
        Ok(())
    }

    fn get_admin_fee(env: &Env) -> i128 {
        read_admin_fee(env)
    }

    fn get_admin_accumulated_fees(env: &Env) -> i128 {
        read_admin_accumulated_fees(env)
    }

    fn withdraw_admin_fees(env: &Env, admin: Address, amount: i128) -> Result<(), Error> {
        let stored_admin = read_admin(env)?;
        stored_admin.require_auth();

        if admin != stored_admin {
            return Err(Error::AdminNotFound);
        }

        if amount <= 0 {
            return Err(Error::AmountMustBePositive);
        }

        let accumulated_fees = read_admin_accumulated_fees(env);

        if amount > accumulated_fees {
            return Err(Error::InsufficientBalance);
        }

        let mut contract_balance = read_contract_balance(env);

        if amount > contract_balance {
            return Err(Error::BalanceNotAvailableForAmountRequested);
        }

        token_transfer(env, &env.current_contract_address(), &admin, &amount)?;

        let new_accumulated_fees = accumulated_fees.checked_sub(amount).ok_or(Error::UnderflowError)?;
        contract_balance = contract_balance.checked_sub(amount).ok_or(Error::UnderflowError)?;

        write_admin_accumulated_fees(env, &new_accumulated_fees);
        write_contract_balance(env, &contract_balance);

        Ok(())
    }

    fn return_car(env: &Env, renter: Address, owner: Address) -> Result<(), Error> {
        // Implementación temporal - se completará en el paso 7
        renter.require_auth();
        Ok(())
    }
}

