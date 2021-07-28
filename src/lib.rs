use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{env, near_bindgen};
use std::str;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct StatusMessage {
    records: LookupMap<String, String>,
}

#[derive(BorshDeserialize, BorshSerialize)]
struct SetMessageInput {
    // Note that the key does not have to be "message" like the argument name.
    msg: String,
}

impl Default for StatusMessage {
    fn default() -> Self {
        Self {
            records: LookupMap::new(b"r".to_vec()),
        }
    }
}

#[near_bindgen]
impl StatusMessage {
    pub fn set_status(&mut self, message: String) {
        let account_id = env::signer_account_id();
        self.records.insert(&account_id, &message);
    }

    pub fn get_status(&self, account_id: String) -> Option<String> {
        return self.records.get(&account_id);
    }

    /// This is an advanced example demonstrating cross-contract calls
    /// and a custom serializer.
    pub fn set_status_borsh(&mut self, #[serializer(borsh)] message: Vec<u8>) {
        let message: String = if cfg!(target_arch = "wasm32") {
            match str::from_utf8(message.as_slice()) {
                Ok(m) => m.to_string(),
                Err(_) => env::panic(b"Invalid UTF-8 sequence"),
            }
        } else {
            let message_obj: SetMessageInput = BorshDeserialize::try_from_slice(&message)
                .expect("Could not deserialize borsh.");
            message_obj.msg
        };

        self.records.insert(&env::signer_account_id(), &String::from(message));
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    fn set_get_message() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = StatusMessage::default();
        contract.set_status("hello".to_string());
        assert_eq!(
            "hello".to_string(),
            contract.get_status("bob_near".to_string()).unwrap()
        );
    }

    #[test]
    fn get_nonexistent_message() {
        let context = get_context(vec![], true);
        testing_env!(context);
        let contract = StatusMessage::default();
        assert_eq!(None, contract.get_status("francis.near".to_string()));
    }

    #[test]
    fn borsh_simple() {
        let status_message = "Aloha honua!".to_string();
        let borsh_input = SetMessageInput {
            msg: status_message.clone()
        };

        let borsh_serialized: Vec<u8> = borsh_input.try_to_vec().unwrap();
        let base64_encoded = near_primitives::serialize::to_base64(borsh_serialized.as_slice());
        println!("Using NEAR CLI, this is the base64-encoded value to use: {:?}", base64_encoded);

        // Set up testing environment and contract
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = StatusMessage::default();

        contract.set_status_borsh(borsh_serialized);
        let get_result = contract.get_status("bob_near".to_string()).unwrap();

        assert_eq!(status_message.to_string(), get_result);
    }
}
