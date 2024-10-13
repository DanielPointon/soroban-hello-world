#![no_std]

// This contract implements fair payment using a claimable balance concept.
// It allows an employer to deposit tokens for a worker and enables a customer to process the payment.
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Vec};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Init,
    Balance,
    Employer,
    Worker,
    Customer,
    TotalTips,  // Store the total tips
}

#[derive(Clone)]
#[contracttype]
pub enum TimeBoundKind {
    Before,
    After,
}

#[derive(Clone)]
#[contracttype]
pub struct ClaimableBalance {
    pub token: Address,
    pub salary_amount: i128,
    pub time_bound_timestamp: u64, // Store only the timestamp
}

#[contract]
pub struct FairPaymentContract;

// Check that the provided timestamp is after the current ledger timestamp.
fn check_time_bound(env: &Env, time_bound_timestamp: u64) -> bool {
    let ledger_timestamp = env.ledger().timestamp();
    ledger_timestamp >= time_bound_timestamp // Only allow payment after time_bound
}

#[contractimpl]
impl FairPaymentContract {
    // Initialize the contract with employer, worker, and customer addresses
    pub fn init(env: Env, employer: Address, worker: Address, customer: Address) {
        // Store the employer, worker, and customer addresses
        env.storage().instance().set(&DataKey::Employer, &employer);
        env.storage().instance().set(&DataKey::Worker, &worker);
        env.storage().instance().set(&DataKey::Customer, &customer);
        // Initialize total tips
        env.storage().instance().set(&DataKey::TotalTips, &0);
        // Mark contract as initialized
        env.storage().instance().set(&DataKey::Init, &());
    }

    pub fn make_payments(
        env: Env,
        from: Address,
        tip_recipients: Vec<Address>,
        business_recipient: Address,
        token: Address,
        value_amount: i128,
        tip_percent: i128,
    ){
        from.require_auth();

        token::Client::new(&env, &token).transfer(
            &from,
            &business_recipient,
            &value_amount,
        );

        let tip_amount = (value_amount * tip_percent) / 100;

        for worker in tip_recipients.iter() {
            // Transfer the total amount of tokens to the worker
            token::Client::new(&env, &token).transfer(
                &from,
                &worker,
                &tip_amount,
            );
        }
    }

    // Deposit salary and set up claimable balance
    pub fn deposit_salary(
        env: Env,
        from: Address,
        token: Address,
        salary_amount: i128,
        time_bound_timestamp: u64, // Accept timestamp directly
    ) {
        from.require_auth();

        if !is_initialized(&env) {
            panic!("contract has not been initialized");
        }

        // Ensure the sender is the employer
        let employer: Address = env.storage().instance().get(&DataKey::Employer).unwrap();
        if from != employer {
            panic!("only the employer can deposit salary");
        }

        // Transfer token from `from` to this contract address.
        token::Client::new(&env, &token).transfer(&from, &env.current_contract_address(), &salary_amount);
        
        // Store the salary info to allow the worker to claim it.
        env.storage().instance().set(
            &DataKey::Balance,
            &ClaimableBalance {
                token,
                salary_amount,
                time_bound_timestamp, // Store the timestamp
            },
        );
    }

    // Deposit tips into the pool
    pub fn deposit_tip(env: Env, from: Address, token: Address, amount: i32) {
        if !is_initialized(&env) {
            panic!("contract has not been initialized");
        }

        let amount_i128 = i128::from(amount);
        from.require_auth();
        // Transfer token from `from` to this contract address.
        token::Client::new(&env, &token).transfer(&from, &env.current_contract_address(), &amount_i128);

        // Update the total tips
        let mut total_tips: i32 = env.storage().instance().get(&DataKey::TotalTips).unwrap();
        total_tips += amount;
        env.storage().instance().set(&DataKey::TotalTips, &total_tips);
    }

    // Execute payment to the worker by the customer
    pub fn execute_payment(env: Env, claimant: Address, token: Address) {        // Require authorization from the caller (worker)
        claimant.require_auth();
    
        // Retrieve claimable balance
        let claimable_balance: ClaimableBalance =
            env.storage().instance().get(&DataKey::Balance).unwrap();
    
        // Check the time bounds
        if !check_time_bound(&env, claimable_balance.time_bound_timestamp) {
            panic!("payment cannot be executed before the time bound");
        }
    
        // Retrieve total tips
        let total_tips: i32 = env.storage().instance().get(&DataKey::TotalTips).unwrap();
    
        // Calculate the total amount to transfer to the worker
        let total_amount = claimable_balance.salary_amount + i128::from(total_tips);
    
        // Transfer the total amount of tokens to the worker
        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &claimant,
            &total_amount,
        );
    
        // Remove the balance entry to prevent any further claims
        env.storage().instance().remove(&DataKey::Balance);
        
        // Reset total tips after payment
        env.storage().instance().set(&DataKey::TotalTips, &0);
    }    
}

// Check if the contract has been initialized
fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Init)
}
