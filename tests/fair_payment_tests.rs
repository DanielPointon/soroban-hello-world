use soroban_sdk::Env;
use soroban_sdk::testutils::{contract::Client, Accounts};
use soroban_sdk::Address;

// Import your contract module
use hello_world::FairPaymentContract;

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::*, Vec};

    #[test]
    fn test_initialize_contract() {
        let env = Env::default();
        let (employer, worker, customer) = env.accounts().generate(3);

        let contract = FairPaymentContract::deploy(&env);

        // Initialize the contract
        contract.init(&env, employer.clone(), worker.clone(), customer.clone());

        // Check if initialized
        assert!(is_initialized(&env));

        // Verify stored addresses
        assert_eq!(env.storage().instance().get(&DataKey::Employer).unwrap(), employer);
        assert_eq!(env.storage().instance().get(&DataKey::Worker).unwrap(), worker);
        assert_eq!(env.storage().instance().get(&DataKey::Customer).unwrap(), customer);
    }

    #[test]
    fn test_deposit_salary() {
        let env = Env::default();
        let (employer, worker, customer) = env.accounts().generate(3);

        let contract = FairPaymentContract::deploy(&env);
        contract.init(&env, employer.clone(), worker.clone(), customer.clone());

        let token_address = Address::from([0u8; 32]); // Replace with actual token address
        let salary_amount = 1000;
        let time_bound_timestamp = env.ledger().timestamp() + 3600; // 1 hour in the future

        // Employer deposits salary
        contract.deposit_salary(&env, employer.clone(), token_address, salary_amount, time_bound_timestamp);

        // Verify balance is set correctly
        let claimable_balance: ClaimableBalance = env.storage().instance().get(&DataKey::Balance).unwrap();
        assert_eq!(claimable_balance.salary_amount, salary_amount);
        assert_eq!(claimable_balance.token, token_address);
        assert_eq!(claimable_balance.time_bound_timestamp, time_bound_timestamp);
    }

    #[test]
    fn test_deposit_tip() {
        let env = Env::default();
        let (employer, worker, customer) = env.accounts().generate(3);

        let contract = FairPaymentContract::deploy(&env);
        contract.init(&env, employer.clone(), worker.clone(), customer.clone());

        let token_address = Address::from([0u8; 32]); // Replace with actual token address
        let tip_amount = 500;

        // Employer deposits a tip
        contract.deposit_tip(&env, employer.clone(), token_address, tip_amount);

        // Verify total tips
        let total_tips: i128 = env.storage().instance().get(&DataKey::TotalTips).unwrap();
        assert_eq!(total_tips, tip_amount);
    }

    #[test]
    fn test_execute_payment() {
        let env = Env::default();
        let (employer, worker, customer) = env.accounts().generate(3);

        let contract = FairPaymentContract::deploy(&env);
        contract.init(&env, employer.clone(), worker.clone(), customer.clone());

        let token_address = Address::from([0u8; 32]); // Replace with actual token address
        let salary_amount = 1000;
        let time_bound_timestamp = env.ledger().timestamp() + 3600; // 1 hour in the future

        // Employer deposits salary
        contract.deposit_salary(&env, employer.clone(), token_address, salary_amount, time_bound_timestamp);

        // Employer deposits a tip
        let tip_amount = 500;
        contract.deposit_tip(&env, employer.clone(), token_address, tip_amount);

        // Simulate time passing
        env.ledger().advance_timestamp(3601); // Advance 1 hour

        // Execute payment
        contract.execute_payment(&env);

        // Verify balance removal
        assert!(env.storage().instance().get(&DataKey::Balance).is_none());

        // Verify total tips reset
        let total_tips: i128 = env.storage().instance().get(&DataKey::TotalTips).unwrap();
        assert_eq!(total_tips, 0);

        // Verify payment made to worker (if you have a way to check this)
        // This depends on how you can assert that tokens were sent to worker
    }

    #[test]
    #[should_panic(expected = "only the employer can deposit salary")]
    fn test_only_employer_can_deposit_salary() {
        let env = Env::default();
        let (employer, worker, customer) = env.accounts().generate(3);

        let contract = FairPaymentContract::deploy(&env);
        contract.init(&env, employer.clone(), worker.clone(), customer.clone());

        let token_address = Address::from([0u8; 32]); // Replace with actual token address
        let salary_amount = 1000;
        let time_bound_timestamp = env.ledger().timestamp() + 3600;

        // Worker attempts to deposit salary, should panic
        contract.deposit_salary(&env, worker.clone(), token_address, salary_amount, time_bound_timestamp);
    }

    #[test]
    #[should_panic(expected = "contract has not been initialized")]
    fn test_execute_payment_without_initialization() {
        let env = Env::default();
        let (employer, worker, customer) = env.accounts().generate(3);

        let contract = FairPaymentContract::deploy(&env);

        // Attempt to execute payment without initialization, should panic
        contract.execute_payment(&env);
    }
}

fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Init)
}
