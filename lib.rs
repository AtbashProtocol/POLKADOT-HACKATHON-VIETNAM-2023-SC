#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod atbash_voting {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    #[derive(scale::Decode, scale::Encode, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Voter {
        voter_address: AccountId,
        token_bought: u128,
        tokens_used_per_candidate: Vec<u128>,
    }

    #[ink(storage)]
    pub struct AtbashVoting {
        votes_received: ink::storage::Mapping<String, u128>,
        voter_info: ink::storage::Mapping<AccountId, Voter>,
        candidate_list: Vec<String>,
        total_tokens: u128,
        balance_tokens: u128,
        token_price: u128,
    }

    impl AtbashVoting {
        #[ink(constructor)]
        pub fn new() -> Self {
            let votes_received = Mapping::default();
            let voter_info = Mapping::default();

            Self {
                votes_received,
                candidate_list: Default::default(),
                total_tokens: Default::default(),
                balance_tokens: Default::default(),
                token_price: Default::default(),
                voter_info,
            }
        }

        #[ink(message)]
        pub fn voting(
            &mut self,
            tokens: u128,
            price_per_token: u128,
            candidate_names: Vec<String>,
        ) {
            self.candidate_list = candidate_names;
            self.total_tokens = tokens;
            self.balance_tokens = tokens;
            self.token_price = price_per_token;
        }

        #[ink(message)]
        pub fn valid_candidate(&self, candidate: String) -> bool {
            for i in 0..self.candidate_list.len() {
                if self.candidate_list[i] == candidate {
                    return true;
                };
            }
            return false;
        }

        #[ink(message)]
        pub fn total_votes_for(&self, candidate: String) -> u128 {
            assert!(
                !self.valid_candidate(candidate.clone()),
                "Not a valid candidate"
            );
            self.votes_received
                .get(candidate.clone())
                .unwrap_or_default()
        }

        #[ink(message)]
        pub fn index_of_candidate(&self, candidate: String) -> u128 {
            assert!(
                !self.valid_candidate(candidate.clone()),
                "Not a valid candidate"
            );

            let mut i = 0;

            let _result = loop {
                i += 1;

                if self.candidate_list[i] == candidate {
                    break i;
                };
            };

            return i as u128;
        }

        #[ink(message)]
        pub fn total_tokens_used(&self, _tokens_used_per_candidate: Vec<u128>) -> u128 {
            let mut total_used_tokens = 0;
            for i in 0.._tokens_used_per_candidate.len() {
                total_used_tokens += _tokens_used_per_candidate[i];
            }
            return total_used_tokens;
        }

        #[ink(message)]
        pub fn vote_for_candidate(&mut self, candidate: String, votes_in_token: u128) {
            assert!(
                !self.valid_candidate(candidate.clone()),
                "Not a valid candidate"
            );
            let index = self.index_of_candidate(candidate.clone());
            let caller = self.env().caller();

            if self
                .voter_info
                .get(caller)
                .unwrap()
                .tokens_used_per_candidate
                .len()
                == 0
            {
                for i in 0..self.candidate_list.len() {
                    self.voter_info
                        .get(caller)
                        .unwrap()
                        .tokens_used_per_candidate
                        .insert(i, 0)
                }

                let available_tokens: u128 = self.voter_info.get(caller).unwrap().token_bought
                    - self.total_tokens_used(
                        self.voter_info
                            .get(caller)
                            .unwrap()
                            .tokens_used_per_candidate,
                    );
                assert!(available_tokens >= votes_in_token);

                self.votes_received.insert(
                    candidate.clone(),
                    &(self.votes_received.get(candidate.clone()).unwrap_or(0) + votes_in_token),
                );

                self.voter_info
                    .get(caller)
                    .unwrap()
                    .tokens_used_per_candidate
                    .insert(
                        index as usize,
                        self.voter_info
                            .get(caller)
                            .unwrap()
                            .tokens_used_per_candidate[index as usize]
                            + votes_in_token,
                    )
            }
        }

        #[ink(message)]
        #[ink(payable)]
        pub fn buy(&mut self) -> u128 {
            let tokens_to_buy = self.env().transferred_value() / self.token_price;
            assert!(tokens_to_buy <= self.balance_tokens);

            let caller = self.env().caller();
            self.voter_info.get(caller).unwrap().voter_address = caller;
            self.voter_info.get(caller).unwrap().token_bought = tokens_to_buy;

            self.balance_tokens -= tokens_to_buy;

            return tokens_to_buy;
        }

        #[ink(message)]
        pub fn token_sold(&self) -> u128 {
            return self.total_tokens - self.balance_tokens;
        }

        #[ink(message)]
        pub fn voter_details(&self, user: AccountId) -> (u128, Vec<u128>) {
            return (
                self.voter_info.get(user).unwrap().token_bought,
                self.voter_info.get(user).unwrap().tokens_used_per_candidate,
            );
        }

        #[ink(message)]
        pub fn transfer_to(&self, account: AccountId) {
            if self.env().transfer(account, self.balance_tokens).is_err() {
                panic!("error transferring")
            }
        }

        #[ink(message)]
        pub fn all_candidates(&self) -> Vec<String> {
            return self.candidate_list.clone();
        }
    }
}
