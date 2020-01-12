///
/// Moonbeam Runtime
/// 
/// Prototype implementation for Moonbeam, a smart contract de-fi parachain.
/// This includes a simple implementation of a token trading system based on
/// a constant product market making formula (x * y = k) similar to how the
/// Uniswap protocol on Ethereum works.
/// 
/// Derek Yoo
/// derek@purestake.com
/// 12-24-19
/// 

use frame_support::{decl_module, decl_storage, decl_event, dispatch, ensure};
use system::{ensure_signed, ensure_root};
use sp_runtime::traits::{CheckedAdd, Saturating};
use sp_std::convert::TryInto;

pub trait Trait: balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Moonbeam {
		/// The glmr balance of each user.
		GlmrBalances get(glmr_balance_of): map T::AccountId => T::Balance;
		/// The glmr pool balance
		GlmrPoolBalance get(glmr_pool_balance): T::Balance;

		/// The token balance of each user.
		TokenBalances get(token_balance_of): map T::AccountId => T::Balance;
		/// The token pool balance
		TokenPoolBalance get(token_pool_balance): T::Balance;

		/// The liquid balance of each user.
		LiquidBalances get(liquid_balance_of): map T::AccountId => T::Balance;
		/// The total liquid supply.
		TotalLiquidSupply get(total_liquid_supply): T::Balance;

		/// Current price of 1 token in glmr - replace with callable readonly function
		TokenPrice get(token_price): T::Balance;
		/// Current price of 1 glmr in tokens - replace with callable readonly function
		GlmrPrice get(glmr_price): T::Balance;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		fn deposit_event() = default;
		
		/// Convenience function to set glmr balance for an account
		/// Only callable by root.
		fn set_glmr_balance(origin, account: T::AccountId, value: T::Balance) -> dispatch::Result {
			let _who = ensure_root(origin)?;

			<GlmrBalances<T>>::insert(account, value);

			Ok(())
		}

		/// Convenience function to set token balance for an account
		/// Only callable by root.
		fn set_token_balance(origin, account: T::AccountId, value: T::Balance) -> dispatch::Result {
			let _who = ensure_root(origin)?;

			<TokenBalances<T>>::insert(account, value);

			Ok(())
		}

		/// Convenience function to transfer glmr balances between accounts
		/// Only callable by root.
		fn transfer_glmr(origin, from: T::AccountId, to: T::AccountId, amount: T::Balance) -> dispatch::Result {
			let _who = ensure_root(origin)?;

			ensure!(<GlmrBalances<T>>::exists(&from), "Glmr from account does not exist");
			let from_balance = Self::glmr_balance_of(&from);
			ensure!(from_balance > amount, "Not enough glmr for transfer");

			let to_balance = Self::glmr_balance_of(&to);

			<GlmrBalances<T>>::insert(&from, from_balance - amount);
			<GlmrBalances<T>>::insert(&to, to_balance.saturating_add(amount));

			Ok(())
		}

		/// Convenience function to transfer glmr balances between accounts
		/// Only callable by root.
		fn transfer_token(origin, from: T::AccountId, to: T::AccountId, amount: T::Balance) -> dispatch::Result {
			let _who = ensure_root(origin)?;

			ensure!(<TokenBalances<T>>::exists(&from), "Token from account does not exist");
			let from_balance = Self::token_balance_of(&from);
			ensure!(from_balance > amount, "Not enough token for transfer");

			let to_balance = Self::token_balance_of(&to);

			<TokenBalances<T>>::insert(&from, from_balance - amount);
			<TokenBalances<T>>::insert(&to, to_balance.saturating_add(amount));

			Ok(())
		}

		/// Convenience function to transfer liquid balances between accounts
		/// Only callable by root.
		fn transfer_liquid(origin, from: T::AccountId, to: T::AccountId, amount: T::Balance) -> dispatch::Result {
			let _who = ensure_root(origin)?;

			ensure!(<LiquidBalances<T>>::exists(&from), "Liquid from account does not exist");
			let from_balance = Self::liquid_balance_of(&from);
			ensure!(from_balance > amount, "Not enough liquid for transfer");

			let to_balance = Self::liquid_balance_of(&to);

			<LiquidBalances<T>>::insert(&from, from_balance - amount);
			<LiquidBalances<T>>::insert(&to, to_balance.saturating_add(amount));

			Ok(())
		}

		/// This function allows users to deposit liquidity into this market.
		/// A deposit consists of some number of gmlr tokens and the token arg is
		/// ignored in all but the first deposit.  In the case that the liquidity pool is being 
		/// initialized, both the specified glmr and token specified amounts are used for the 
		/// initial deposit.  In return the user will recieve a deposit of liquid.
		/// Liquid tokens give the user a right to a share of the profits generated
		/// by trading on the market.
		fn deposit_liquidity(origin, glmr_value: T::Balance, token_value: T::Balance) -> dispatch::Result {
			let sender = ensure_signed(origin)?;
			let sender_glmr_balance = Self::glmr_balance_of(&sender);
			ensure!(sender_glmr_balance >= glmr_value, "Not enough glmr to cover liquidity deposit");
			let sender_token_balance = Self::token_balance_of(&sender);
			ensure!(sender_token_balance >= token_value, "Not enough tokens to cover liquidity deposit");
			
			let total_liquid_supply = Self::total_liquid_supply();
			let glmr_reserve = Self::glmr_pool_balance();
			let token_reserve = Self::token_pool_balance();
			let liquid_minted;

			if total_liquid_supply > T::Balance::from(0) {
				// add liquidity to pool
				ensure!(glmr_reserve > T::Balance::from(0), "There is liquidity in this exchange but the glmr reserve is empty");
				let token_amount = glmr_value * token_reserve / glmr_reserve + T::Balance::from(1);
				ensure!(token_amount <= sender_token_balance, "You do not have enough tokens to complete the deposit");
				liquid_minted = glmr_value * total_liquid_supply / glmr_reserve;

				let glmr_newbal = match glmr_reserve.checked_add(&glmr_value) {
					Some(val) => val,
					None => return Err("Glmr reserve balance overflow"),
				};

				let token_newbal = match token_reserve.checked_add(&token_amount) {
					Some(val) => val,
					None => return Err("Token reserve balance overflow"),
				};

				let sender_liquid_balance = Self::liquid_balance_of(&sender);
				let liquid_newbal = match sender_liquid_balance.checked_add(&liquid_minted) {
					Some(val) => val,
					None => return Err("User liquid balance overflow"),
				};

				let liquid_supply_newbal = match total_liquid_supply.checked_add(&liquid_minted) {
					Some(val) => val,
					None => return Err("Liquid supply balance overflow"),
				};

				<GlmrBalances<T>>::insert(&sender, sender_glmr_balance - glmr_value);
				<GlmrPoolBalance<T>>::put(glmr_newbal);

				<TokenBalances<T>>::insert(&sender, sender_token_balance - token_amount);
				<TokenPoolBalance<T>>::put(token_newbal);
				
				<LiquidBalances<T>>::insert(&sender, liquid_newbal);
				<TotalLiquidSupply<T>>::put(liquid_supply_newbal);
				

			} else {
				// initialize liquidity pool
				liquid_minted = glmr_value;

				<GlmrPoolBalance<T>>::put(glmr_value);
				<GlmrBalances<T>>::insert(&sender, sender_glmr_balance - glmr_value);

				<TokenPoolBalance<T>>::put(token_value);
				<TokenBalances<T>>::insert(&sender, sender_token_balance - token_value);
				
				<TotalLiquidSupply<T>>::put(liquid_minted);
				<LiquidBalances<T>>::insert(&sender, liquid_minted);
			}

			Self::update_prices();
			Self::deposit_event(RawEvent::DepositLiquidity(sender, liquid_minted));

			Ok(())
		}

		/// Liquid token holders may withdraw their deposit at any time.  When they return
		/// their liquid tokens they get back a proportional share of the liquidity pool.
		/// This consists of a number of glmr and a number of tokens and includes a pro rata
		/// portion of trading fees which have been collected since the deposit was made.
		fn withdraw_liquidity(origin, liquid_value: T::Balance) -> dispatch::Result {
			let sender = ensure_signed(origin)?;

			let total_liquid_supply = Self::total_liquid_supply();
			ensure!(total_liquid_supply > T::Balance::from(0) && 
				liquid_value <= total_liquid_supply,
				"Not enough liquidity in pool to withdraw");
			let glmr_reserve = Self::glmr_pool_balance();
			let token_reserve = Self::token_pool_balance();
			let glmr_amount = liquid_value * glmr_reserve / total_liquid_supply;
			let token_amount = liquid_value * token_reserve / total_liquid_supply;
			let sender_liquid_balance = Self::liquid_balance_of(&sender);
			ensure!(liquid_value <= sender_liquid_balance, "Trying to withdraw more than owned liquidity");
			let sender_glmr_balance = Self::glmr_balance_of(&sender);
			ensure!(glmr_amount <= glmr_reserve, "Trying to withdraw more GLMR than is in the pool");
			let sender_token_balance = Self::token_balance_of(&sender);
			ensure!(token_amount <= token_reserve, "Trying to withdraw more Token than is in the pool");

			let glmr_newbal = match sender_glmr_balance.checked_add(&glmr_amount) {
				Some(val) => val,
				None => return Err("Glmr user balance overflow"),
			};

			let token_newbal = match sender_token_balance.checked_add(&token_amount) {
				Some(val) => val,
				None => return Err("Token user balance overflow"),
			};
			
			<LiquidBalances<T>>::insert(&sender, sender_liquid_balance - liquid_value);
			<TotalLiquidSupply<T>>::put(total_liquid_supply - liquid_value);
			
			<GlmrBalances<T>>::insert(&sender, glmr_newbal);
			<GlmrPoolBalance<T>>::put(glmr_reserve - glmr_amount);
			
			<TokenBalances<T>>::insert(&sender, token_newbal);
			<TokenPoolBalance<T>>::put(token_reserve - token_amount);
			
			Self::update_prices();
			Self::deposit_event(RawEvent::WithdrawLiquidity(sender, liquid_value));

			Ok(())
		}

		/// users can call this function to execute a trade of glmr to tokens.
		/// the number of tokens you get for a specified input number of glmr
		/// is algorithmically determined by the x * y = k constant product
		/// market making formula.  there is also a 0.3% trading fee which is
		/// charged for every trade.  this fee is added to the liquidity pool
		/// and accrues to liquidity token holders.
		fn trade_glmr_to_token(origin, glmr_value: T::Balance) -> dispatch::Result {
			let sender = ensure_signed(origin)?;

			let glmr_reserve = Self::glmr_pool_balance();
			let token_reserve = Self::token_pool_balance();

			let tokens_bought = match Self::get_price(glmr_value, glmr_reserve, token_reserve) {
				Some(val) => val,
				None => return Err("Error caluculating number of tokens in trade"),
			};

			let sender_glmr_balance = Self::glmr_balance_of(&sender);
			ensure!(sender_glmr_balance >= glmr_value, "Not enough glmr to execute trade");
			let sender_token_balance = Self::token_balance_of(&sender);
			ensure!(token_reserve >= tokens_bought, "Not enough tokens to execute trade");

			let glmr_pool_newbal = match glmr_reserve.checked_add(&glmr_value) {
				Some(val) => val,
				None => return Err("GLMR pool balance overflow"),
			};

			let token_newbal = match sender_token_balance.checked_add(&tokens_bought) {
				Some(val) => val,
				None => return Err("User token balance overflow"),
			};

			// tranfer glmr in
			<GlmrBalances<T>>::insert(&sender, sender_glmr_balance - glmr_value);
			<GlmrPoolBalance<T>>::put(glmr_pool_newbal);

			// transfer token out
			<TokenBalances<T>>::insert(&sender, token_newbal);
			<TokenPoolBalance<T>>::put(token_reserve - tokens_bought);

			Self::update_prices();
			Self::deposit_event(RawEvent::TokenPurchase(sender, tokens_bought));

			Ok(())
		}

		/// users can call this function to trade tokens for glmr.  the number of
		/// glmr you get for a given amount of tokens is determined by the
		/// x * y = k constant product market making formula. there is also a 0.3% 
		/// trading fee which is charged for every trade.  this fee is added to the 
		/// liquidity pool and accrues to liquidity token holders.
		fn trade_token_to_glmr(origin, token_value: T::Balance) -> dispatch::Result {
			let sender = ensure_signed(origin)?;

			let glmr_reserve = Self::glmr_pool_balance();
			let token_reserve = Self::token_pool_balance();

			let glmr_bought = match Self::get_price(token_value, token_reserve, glmr_reserve) {
				Some(val) => val,
				None => return Err("Error caluculating number of GLMR in trade"),
			};

			let sender_token_balance = Self::token_balance_of(&sender);
			ensure!(sender_token_balance >= token_value, "Not enough tokens to execute trade");
			let sender_glmr_balance = Self::glmr_balance_of(&sender);
			ensure!(glmr_reserve >= glmr_bought, "Not enough glmr to execute trade");

			let token_pool_newbal = match token_reserve.checked_add(&token_value) {
				Some(val) => val,
				None => return Err("Token pool balance overflow"),
			};

			let glmr_newbal = match sender_glmr_balance.checked_add(&glmr_bought) {
				Some(val) => val,
				None => return Err("User GLMR balance overflow"),
			};

			// tranfer token in
			<TokenBalances<T>>::insert(&sender, sender_token_balance - token_value);
			<TokenPoolBalance<T>>::put(token_pool_newbal);

			// transfer glmr out
			<GlmrBalances<T>>::insert(&sender, glmr_newbal);
			<GlmrPoolBalance<T>>::put(glmr_reserve - glmr_bought);

			Self::update_prices();
			Self::deposit_event(RawEvent::GlmrPurchase(sender, glmr_bought));

			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> 
	where 
		AccountId = <T as system::Trait>::AccountId,
		Balance = <T as balances::Trait>::Balance
	{
		TokenPurchase(AccountId, Balance),
		GlmrPurchase(AccountId, Balance),
		DepositLiquidity(AccountId, Balance),
		WithdrawLiquidity(AccountId, Balance),
	}
);

impl<T: Trait> Module<T> {
	fn get_price(amount: T::Balance, input_reserve: T::Balance, output_reserve: T::Balance) -> Option<T::Balance> {
		if amount <= T::Balance::from(0) || input_reserve <= T::Balance::from(0) || output_reserve <= T::Balance::from(0) {
			return None	
		}

		let net_amount = match TryInto::<u128>::try_into(amount) {
			Ok(converted_val) => match converted_val.checked_mul(997) {
				Some(result_val) => result_val,
				None => return None,
			},
			Err(_e) => return None,
		};

		let numerator = match TryInto::<u128>::try_into(output_reserve) {
			Ok(converted_val) => match converted_val.checked_mul(net_amount) {
				Some(result_val) => result_val,
				None => return None,
			},
			Err(_e) => return None,
		};

		let denominator = match TryInto::<u128>::try_into(input_reserve) {
			Ok(converted_val) => match converted_val.checked_mul(1000) {
				Some(multiplied_val) => match multiplied_val.checked_add(net_amount) {
					Some(result_val) => result_val,
					None => return None,
				}
				None => return None,
			},
			Err(_e) => return None,
		};

		let result = match numerator.checked_div(denominator) {
			Some(val) => val,
			None => return None,
		};

		result.try_into().ok()
	}

	fn update_prices() {
		let glmr_reserve = Self::glmr_pool_balance();
		let token_reserve = Self::token_pool_balance();
		let glmr_price = Self::get_price(1000000000000u128.try_into().unwrap_or(T::Balance::from(0)), token_reserve, glmr_reserve);
		let token_price = Self::get_price(1000000000000u128.try_into().unwrap_or(T::Balance::from(0)), glmr_reserve, token_reserve);

		if ! glmr_price.is_none() && ! token_price.is_none() {
			<GlmrPrice<T>>::put(glmr_price.unwrap());
			<TokenPrice<T>>::put(token_price.unwrap());
		}
	}
}
