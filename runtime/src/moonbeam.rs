///
/// Moonbeam Runtime
/// 
/// Prototype implementation for Moonbeam, a smart contract de-fi parachain.
/// This includes a simplified implementation of the Uniswap protocol. Needless
/// to say all of the dex functionality is inspired by (stolen from?) Uniswap.
/// 
/// Derek Yoo
/// derek@purestake.com
/// 12-24-19
/// 

use frame_support::{decl_module, decl_storage, decl_event, dispatch, ensure};
use system::{ensure_signed, ensure_root};
use sp_runtime::traits::Saturating;

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

		/// The total liquid supply.
		TotalLiquidSupply get(total_liquid_supply): T::Balance;
		/// The liquid balance of each user.
		LiquidBalances get(liquid_balance_of): map T::AccountId => T::Balance;

		/// Current token price scratch placeholder.
		TokenPrice get(token_price): T::Balance;
		/// Current glmr price scratch placeholder.
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

		/// This function allows users to deposit liquidity into this exchange.
		/// A deposit consists of some number of gmlr and some number of tokens.
		/// In return the user will recieve a deposit of liquid.
		/// Liquid tokens give the user a right to a share of the profits generated
		/// by trading on the exchange.
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
				liquid_minted = glmr_value * total_liquid_supply / glmr_reserve;

				<GlmrBalances<T>>::insert(&sender, sender_glmr_balance - glmr_value);
				<GlmrPoolBalance<T>>::put(glmr_reserve.saturating_add(glmr_value));

				<TokenPoolBalance<T>>::put(token_reserve.saturating_add(token_amount));
				<TokenBalances<T>>::insert(&sender, sender_token_balance - token_amount);

				<TotalLiquidSupply<T>>::put(total_liquid_supply.saturating_add(liquid_minted));
				let sender_liquid_balance = Self::liquid_balance_of(&sender);
				<LiquidBalances<T>>::insert(&sender, sender_liquid_balance.saturating_add(liquid_minted));

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
			
			<LiquidBalances<T>>::insert(&sender, sender_liquid_balance - liquid_value);
			<TotalLiquidSupply<T>>::put(total_liquid_supply - liquid_value);
			
			<GlmrBalances<T>>::insert(&sender, sender_glmr_balance.saturating_add(glmr_amount));
			<GlmrPoolBalance<T>>::put(glmr_reserve - glmr_amount);
			
			<TokenBalances<T>>::insert(&sender, sender_token_balance.saturating_add(token_amount));
			<TokenPoolBalance<T>>::put(token_reserve - token_amount);
			
			Self::deposit_event(RawEvent::WithdrawLiquidity(sender, liquid_value));

			Ok(())
		}

		/// function to price glmr in terms of tokens
		fn get_glmr_to_token_price(origin, glmr_value: T::Balance) -> dispatch::Result {
			let _sender = ensure_signed(origin)?;
			ensure!(glmr_value > T::Balance::from(0), "No glmr specified to price tokens");

			let glmr_reserve = Self::glmr_pool_balance();
			let token_reserve = Self::token_pool_balance();
			// hack, this needs to be fixed.
			<TokenPrice<T>>::put(Self::get_price(glmr_value, glmr_reserve, token_reserve));

			Ok(())
		}

		/// function to price tokens in terms of glmr
		fn get_token_to_glmr_price(origin, token_value: T::Balance) -> dispatch::Result {
			let _sender = ensure_signed(origin)?;
			ensure!(token_value > T::Balance::from(0), "No tokens specified to price glmr");

			let glmr_reserve = Self::glmr_pool_balance();
			let token_reserve = Self::token_pool_balance();
			// hack, this needs to be fixed.
			<GlmrPrice<T>>::put(Self::get_price(token_value, token_reserve, glmr_reserve));

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
			let tokens_bought = Self::get_price(glmr_value, glmr_reserve, token_reserve);
			let sender_glmr_balance = Self::glmr_balance_of(&sender);
			ensure!(sender_glmr_balance >= glmr_value, "Not enough glmr to execute trade");
			let sender_token_balance = Self::token_balance_of(&sender);
			ensure!(token_reserve >= tokens_bought, "Not enough tokens to execute trade");

			// tranfer glmr in
			<GlmrPoolBalance<T>>::put(glmr_reserve.saturating_add(glmr_value));
			<GlmrBalances<T>>::insert(&sender, sender_glmr_balance - glmr_value);

			// transfer token out
			<TokenPoolBalance<T>>::put(token_reserve - tokens_bought);
			<TokenBalances<T>>::insert(&sender, sender_token_balance.saturating_add(tokens_bought));

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
			let glmr_bought = Self::get_price(token_value, token_reserve, glmr_reserve);
			let sender_token_balance = Self::token_balance_of(&sender);
			ensure!(sender_token_balance >= token_value, "Not enough tokens to execute trade");
			let sender_glmr_balance = Self::glmr_balance_of(&sender);
			ensure!(glmr_reserve >= glmr_bought, "Not enough glmr to execute trade");

			// tranfer token in
			<TokenPoolBalance<T>>::put(token_reserve + token_value);
			<TokenBalances<T>>::insert(&sender, sender_token_balance - token_value);

			// transfer glmr out
			<GlmrPoolBalance<T>>::put(glmr_reserve - glmr_bought);
			<GlmrBalances<T>>::insert(&sender, sender_glmr_balance.saturating_add(glmr_bought));

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
	fn get_price(amount: T::Balance, input_reserve: T::Balance, output_reserve: T::Balance) -> T::Balance {
		//ensure!(input_reserve > T::Balance::from(0), "There is no input reserve");
		//ensure!(output_reserve > T::Balance::from(0), "There is no output reserve");

		let net_amount = amount * T::Balance::from(997);
		let numerator = net_amount * output_reserve;
		let denominator = (input_reserve * T::Balance::from(1000)).saturating_add(net_amount);

		// this is unsafe.  need to come back and fix this.
		numerator / denominator
	}
}
