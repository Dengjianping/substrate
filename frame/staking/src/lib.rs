// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! # Staking Module
//!
//! The Staking module is used to manage funds at stake by network maintainers.
//!
//! - [`staking::Trait`](./trait.Trait.html)
//! - [`Call`](./enum.Call.html)
//! - [`Module`](./struct.Module.html)
//!
//! ## Overview
//!
//! The Staking module is the means by which a set of network maintainers (known as _authorities_
//! in some contexts and _validators_ in others) are chosen based upon those who voluntarily place
//! funds under deposit. Under deposit, those funds are rewarded under normal operation but are
//! held at pain of _slash_ (expropriation) should the staked maintainer be found not to be
//! discharging its duties properly.
//!
//! ### Terminology
//! <!-- Original author of paragraph: @gavofyork -->
//!
//! - Staking: The process of locking up funds for some time, placing them at risk of slashing
//! (loss) in order to become a rewarded maintainer of the network.
//! - Validating: The process of running a node to actively maintain the network, either by
//! producing blocks or guaranteeing finality of the chain.
//! - Nominating: The process of placing staked funds behind one or more validators in order to
//! share in any reward, and punishment, they take.
//! - Stash account: The account holding an owner's funds used for staking.
//! - Controller account: The account that controls an owner's funds for staking.
//! - Era: A (whole) number of sessions, which is the period that the validator set (and each
//! validator's active nominator set) is recalculated and where rewards are paid out.
//! - Slash: The punishment of a staker by reducing its funds.
//!
//! ### Goals
//! <!-- Original author of paragraph: @gavofyork -->
//!
//! The staking system in Substrate NPoS is designed to make the following possible:
//!
//! - Stake funds that are controlled by a cold wallet.
//! - Withdraw some, or deposit more, funds without interrupting the role of an entity.
//! - Switch between roles (nominator, validator, idle) with minimal overhead.
//!
//! ### Scenarios
//!
//! #### Staking
//!
//! Almost any interaction with the Staking module requires a process of _**bonding**_ (also known
//! as being a _staker_). To become *bonded*, a fund-holding account known as the _stash account_,
//! which holds some or all of the funds that become frozen in place as part of the staking process,
//! is paired with an active **controller** account, which issues instructions on how they shall be
//! used.
//!
//! An account pair can become bonded using the [`bond`](./enum.Call.html#variant.bond) call.
//!
//! Stash accounts can change their associated controller using the
//! [`set_controller`](./enum.Call.html#variant.set_controller) call.
//!
//! There are three possible roles that any staked account pair can be in: `Validator`, `Nominator`
//! and `Idle` (defined in [`StakerStatus`](./enum.StakerStatus.html)). There are three
//! corresponding instructions to change between roles, namely:
//! [`validate`](./enum.Call.html#variant.validate),
//! [`nominate`](./enum.Call.html#variant.nominate), and [`chill`](./enum.Call.html#variant.chill).
//!
//! #### Validating
//!
//! A **validator** takes the role of either validating blocks or ensuring their finality,
//! maintaining the veracity of the network. A validator should avoid both any sort of malicious
//! misbehavior and going offline. Bonded accounts that state interest in being a validator do NOT
//! get immediately chosen as a validator. Instead, they are declared as a _candidate_ and they
//! _might_ get elected at the _next era_ as a validator. The result of the election is determined
//! by nominators and their votes.
//!
//! An account can become a validator candidate via the
//! [`validate`](./enum.Call.html#variant.validate) call.
//!
//! #### Nomination
//!
//! A **nominator** does not take any _direct_ role in maintaining the network, instead, it votes on
//! a set of validators  to be elected. Once interest in nomination is stated by an account, it
//! takes effect at the next election round. The funds in the nominator's stash account indicate the
//! _weight_ of its vote. Both the rewards and any punishment that a validator earns are shared
//! between the validator and its nominators. This rule incentivizes the nominators to NOT vote for
//! the misbehaving/offline validators as much as possible, simply because the nominators will also
//! lose funds if they vote poorly.
//!
//! An account can become a nominator via the [`nominate`](enum.Call.html#variant.nominate) call.
//!
//! #### Rewards and Slash
//!
//! The **reward and slashing** procedure is the core of the Staking module, attempting to _embrace
//! valid behavior_ while _punishing any misbehavior or lack of availability_.
//!
//! Reward must be claimed by stakers for each era before it gets too old by $HISTORY_DEPTH using
//! `payout_nominator` and `payout_validator` calls.
//! Only the [`T::MaxNominatorRewardedPerValidator`] biggest stakers can claim their reward. This
//! limit the i/o cost to compute nominators payout.
//!
//! Slashing can occur at any point in time, once misbehavior is reported. Once slashing is
//! determined, a value is deducted from the balance of the validator and all the nominators who
//! voted for this validator (values are deducted from the _stash_ account of the slashed entity).
//!
//! Slashing logic is further described in the documentation of the `slashing` module.
//!
//! Similar to slashing, rewards are also shared among a validator and its associated nominators.
//! Yet, the reward funds are not always transferred to the stash account and can be configured.
//! See [Reward Calculation](#reward-calculation) for more details.
//!
//! #### Chilling
//!
//! Finally, any of the roles above can choose to step back temporarily and just chill for a while.
//! This means that if they are a nominator, they will not be considered as voters anymore and if
//! they are validators, they will no longer be a candidate for the next election.
//!
//! An account can step back via the [`chill`](enum.Call.html#variant.chill) call.
//!
//! ### Session managing
//!
//! The module implement the trait `SessionManager`. Which is the only API to query new validator
//! set and allowing these validator set to be rewarded once their era is ended.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! The dispatchable functions of the Staking module enable the steps needed for entities to accept
//! and change their role, alongside some helper functions to get/set the metadata of the module.
//!
//! ### Public Functions
//!
//! The Staking module contains many public storage items and (im)mutable functions.
//!
//! ## Usage
//!
//! ### Example: Rewarding a validator by id.
//!
//! ```
//! use frame_support::{decl_module, dispatch};
//! use frame_system::{self as system, ensure_signed};
//! use pallet_staking::{self as staking};
//!
//! pub trait Trait: staking::Trait {}
//!
//! decl_module! {
//! 	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
//!			/// Reward a validator.
//! 		pub fn reward_myself(origin) -> dispatch::DispatchResult {
//! 			let reported = ensure_signed(origin)?;
//! 			<staking::Module<T>>::reward_by_ids(vec![(reported, 10)]);
//! 			Ok(())
//! 		}
//! 	}
//! }
//! # fn main() { }
//! ```
//!
//! ## Implementation Details
//!
//! ### Reward Calculation
//!
//! Validators and nominators are rewarded at the end of each era. The total reward of an era is
//! calculated using the era duration and the staking rate (the total amount of tokens staked by
//! nominators and validators, divided by the total token supply). It aims to incentivize toward a
//! defined staking rate. The full specification can be found
//! [here](https://research.web3.foundation/en/latest/polkadot/Token%20Economics.html#inflation-model).
//!
//! Total reward is split among validators and their nominators depending on the number of points
//! they received during the era. Points are added to a validator using
//! [`reward_by_ids`](./enum.Call.html#variant.reward_by_ids) or
//! [`reward_by_indices`](./enum.Call.html#variant.reward_by_indices).
//!
//! [`Module`](./struct.Module.html) implements
//! [`pallet_authorship::EventHandler`](../pallet_authorship/trait.EventHandler.html) to add reward
//! points to block producer and block producer of referenced uncles.
//!
//! The validator and its nominator split their reward as following:
//!
//! The validator can declare an amount, named
//! [`commission`](./struct.ValidatorPrefs.html#structfield.commission), that does not
//! get shared with the nominators at each reward payout through its
//! [`ValidatorPrefs`](./struct.ValidatorPrefs.html). This value gets deducted from the total reward
//! that is paid to the validator and its nominators. The remaining portion is split among the
//! validator and all of the nominators that nominated the validator, proportional to the value
//! staked behind this validator (_i.e._ dividing the
//! [`own`](./struct.Exposure.html#structfield.own) or
//! [`others`](./struct.Exposure.html#structfield.others) by
//! [`total`](./struct.Exposure.html#structfield.total) in [`Exposure`](./struct.Exposure.html)).
//!
//! All entities who receive a reward have the option to choose their reward destination
//! through the [`Payee`](./struct.Payee.html) storage item (see
//! [`set_payee`](enum.Call.html#variant.set_payee)), to be one of the following:
//!
//! - Controller account, (obviously) not increasing the staked value.
//! - Stash account, not increasing the staked value.
//! - Stash account, also increasing the staked value.
//!
//! ### Additional Fund Management Operations
//!
//! Any funds already placed into stash can be the target of the following operations:
//!
//! The controller account can free a portion (or all) of the funds using the
//! [`unbond`](enum.Call.html#variant.unbond) call. Note that the funds are not immediately
//! accessible. Instead, a duration denoted by [`BondingDuration`](./struct.BondingDuration.html)
//! (in number of eras) must pass until the funds can actually be removed. Once the
//! `BondingDuration` is over, the [`withdraw_unbonded`](./enum.Call.html#variant.withdraw_unbonded)
//! call can be used to actually withdraw the funds.
//!
//! Note that there is a limitation to the number of fund-chunks that can be scheduled to be
//! unlocked in the future via [`unbond`](enum.Call.html#variant.unbond). In case this maximum
//! (`MAX_UNLOCKING_CHUNKS`) is reached, the bonded account _must_ first wait until a successful
//! call to `withdraw_unbonded` to remove some of the chunks.
//!
//! ### Election Algorithm
//!
//! The current election algorithm is implemented based on Phragmén.
//! The reference implementation can be found
//! [here](https://github.com/w3f/consensus/tree/master/NPoS).
//!
//! The election algorithm, aside from electing the validators with the most stake value and votes,
//! tries to divide the nominator votes among candidates in an equal manner. To further assure this,
//! an optional post-processing can be applied that iteratively normalizes the nominator staked
//! values until the total difference among votes of a particular nominator are less than a
//! threshold.
//!
//! ## GenesisConfig
//!
//! The Staking module depends on the [`GenesisConfig`](./struct.GenesisConfig.html).
//! The `GenesisConfig` is optional and allow to set some initial stakers.
//!
//! ## Related Modules
//!
//! - [Balances](../pallet_balances/index.html): Used to manage values at stake.
//! - [Session](../pallet_session/index.html): Used to manage sessions. Also, a list of new
//!   validators is stored in the Session module's `Validators` at the end of each era.

#![recursion_limit = "128"]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
#[cfg(feature = "testing-utils")]
pub mod testing_utils;

pub mod slashing;
pub mod offchain_election;
pub mod inflation;

use sp_std::{prelude::*, result, collections::btree_map::BTreeMap, convert::{TryInto, From}};
use codec::{HasCompact, Encode, Decode};
use frame_support::{
	decl_module, decl_event, decl_storage, ensure, decl_error, debug, Parameter,
	weights::SimpleDispatchInfo,
	dispatch::{IsSubType, DispatchResult},
	traits::{
		Currency, LockIdentifier, LockableCurrency, WithdrawReasons, OnUnbalanced, Imbalance, Get,
		Time, EstimateNextNewSession,
	}
};
use pallet_session::historical;
use sp_runtime::{
	Perbill, PerU16, PerThing, RuntimeDebug, RuntimeAppPublic,
	curve::PiecewiseLinear,
	traits::{
		Convert, Zero, StaticLookup, CheckedSub, Saturating, SaturatedConversion, AtLeast32Bit,
		EnsureOrigin, Member, SignedExtension,
	},
	transaction_validity::{
		TransactionValidityError, TransactionValidity, ValidTransaction, InvalidTransaction,
		UnknownTransaction,
	},
};
use sp_staking::{
	SessionIndex,
	offence::{OnOffenceHandler, OffenceDetails, Offence, ReportOffence, OffenceError},
};
#[cfg(feature = "std")]
use sp_runtime::{Serialize, Deserialize};
use frame_system::{
	self as system, ensure_signed, ensure_root, ensure_none,
	offchain::SubmitUnsignedTransaction,
};
use sp_phragmen::{
	ExtendedBalance, Assignment, PhragmenScore, PhragmenResult, build_support_map, evaluate_support,
	elect, generate_compact_solution_type, is_score_better, VotingLimit, SupportMap,
};

const DEFAULT_MINIMUM_VALIDATOR_COUNT: u32 = 4;
const MAX_UNLOCKING_CHUNKS: usize = 32;
const MAX_NOMINATIONS: usize = <CompactAssignments as VotingLimit>::LIMIT;
const STAKING_ID: LockIdentifier = *b"staking ";

/// Data type used to index nominators in the compact type
pub type NominatorIndex = u32;

/// Data type used to index validators in the compact type.
pub type ValidatorIndex = u16;

// Ensure the size of both ValidatorIndex and NominatorIndex
static_assertions::const_assert!(sp_std::mem::size_of::<ValidatorIndex>() <= sp_std::mem::size_of::<usize>());
static_assertions::const_assert!(sp_std::mem::size_of::<NominatorIndex>() <= sp_std::mem::size_of::<usize>());

/// Maximum number of stakers that can be stored in a snapshot.
pub(crate) const MAX_VALIDATORS: usize = ValidatorIndex::max_value() as usize;
pub(crate) const MAX_NOMINATORS: usize = NominatorIndex::max_value() as usize;

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

/// Counter for the number of "reward" points earned by a given validator.
pub type RewardPoint = u32;

// Note: Maximum nomination limit is set here -- 16.
generate_compact_solution_type!(pub GenericCompactAssignments, 16);

/// Information regarding the active era (era in used in session).
#[derive(Encode, Decode, Debug)]
pub struct ActiveEraInfo<Moment> {
	/// Index of era.
	index: EraIndex,
	/// Moment of start
	///
	/// Start can be none if start hasn't been set for the era yet,
	/// Start is set on the first on_finalize of the era to guarantee usage of `Time`.
	start: Option<Moment>,
}

/// Accuracy used for on-chain phragmen
pub type ChainAccuracy = Perbill;

/// Accuracy used for off-chain phragmen. This better be small.
pub type OffchainAccuracy = PerU16;

/// The balance type of this module.
pub type BalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// The compact type for election solutions.
pub type CompactAssignments =
	GenericCompactAssignments<NominatorIndex, ValidatorIndex, OffchainAccuracy>;

type PositiveImbalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::PositiveImbalance;
type NegativeImbalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;
type MomentOf<T> = <<T as Trait>::Time as Time>::Moment;

/// Staking's key type used for submitting unsigned solutions.
pub mod sr25519 {
	mod app_sr25519 {
		use sp_application_crypto::{app_crypto, key_types::STAKING, sr25519};
		app_crypto!(sr25519, STAKING);
	}

	sp_application_crypto::with_pair! {
		/// A staking keypair using sr25519 as its crypto.
		pub type AuthorityPair = app_sr25519::Pair;
	}

	/// Staking signature using sr25519 as its crypto.
	pub type AuthoritySignature = app_sr25519::Signature;

	/// Staking identifier using sr25519 as its crypto.
	pub type AuthorityId = app_sr25519::Public;
}

/// Reward points of an era. Used to split era total payout between validators.
///
/// This points will be used to reward validators and their respective nominators.
#[derive(PartialEq, Encode, Decode, Default, Debug)]
pub struct EraRewardPoints<AccountId: Ord> {
	/// Total number of points. Equals the sum of reward points for each validator.
	total: RewardPoint,
	/// The reward points earned by a given validator.
	individual: BTreeMap<AccountId, RewardPoint>,
}

/// Indicates the initial status of the staker.
#[derive(RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum StakerStatus<AccountId> {
	/// Chilling.
	Idle,
	/// Declared desire in validating or already participating in it.
	Validator,
	/// Nominating for a group of other stakers.
	Nominator(Vec<AccountId>),
}

/// A destination account for payment.
#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug)]
pub enum RewardDestination {
	/// Pay into the stash account, increasing the amount at stake accordingly.
	Staked,
	/// Pay into the stash account, not increasing the amount at stake.
	Stash,
	/// Pay into the controller account.
	Controller,
}

impl Default for RewardDestination {
	fn default() -> Self {
		RewardDestination::Staked
	}
}

/// Preference of what happens regarding validation.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ValidatorPrefs {
	/// Reward that validator takes up-front; only the rest is split between themselves and
	/// nominators.
	#[codec(compact)]
	pub commission: Perbill,
}

impl Default for ValidatorPrefs {
	fn default() -> Self {
		ValidatorPrefs {
			commission: Default::default(),
		}
	}
}

/// Just a Balance/BlockNumber tuple to encode when a chunk of funds will be unlocked.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct UnlockChunk<Balance: HasCompact> {
	/// Amount of funds to be unlocked.
	#[codec(compact)]
	value: Balance,
	/// Era number at which point it'll be unlocked.
	#[codec(compact)]
	era: EraIndex,
}

/// The ledger of a (bonded) stash.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct StakingLedger<AccountId, Balance: HasCompact> {
	/// The stash account whose balance is actually locked and at stake.
	pub stash: AccountId,
	/// The total amount of the stash's balance that we are currently accounting for.
	/// It's just `active` plus all the `unlocking` balances.
	#[codec(compact)]
	pub total: Balance,
	/// The total amount of the stash's balance that will be at stake in any forthcoming
	/// rounds.
	#[codec(compact)]
	pub active: Balance,
	/// Any balance that is becoming free, which may eventually be transferred out
	/// of the stash (assuming it doesn't get slashed first).
	pub unlocking: Vec<UnlockChunk<Balance>>,
	/// The latest and highest era which the staker has claimed reward for.
	pub last_reward: Option<EraIndex>,
}

impl<
	AccountId,
	Balance: HasCompact + Copy + Saturating + AtLeast32Bit,
> StakingLedger<AccountId, Balance> {
	/// Remove entries from `unlocking` that are sufficiently old and reduce the
	/// total by the sum of their balances.
	fn consolidate_unlocked(self, current_era: EraIndex) -> Self {
		let mut total = self.total;
		let unlocking = self.unlocking.into_iter()
			.filter(|chunk| if chunk.era > current_era {
				true
			} else {
				total = total.saturating_sub(chunk.value);
				false
			})
			.collect();

		Self {
			stash: self.stash,
			total,
			active: self.active,
			unlocking,
			last_reward: self.last_reward
		}
	}

	/// Re-bond funds that were scheduled for unlocking.
	fn rebond(mut self, value: Balance) -> Self {
		let mut unlocking_balance: Balance = Zero::zero();

		while let Some(last) = self.unlocking.last_mut() {
			if unlocking_balance + last.value <= value {
				unlocking_balance += last.value;
				self.active += last.value;
				self.unlocking.pop();
			} else {
				let diff = value - unlocking_balance;

				unlocking_balance += diff;
				self.active += diff;
				last.value -= diff;
			}

			if unlocking_balance >= value {
				break
			}
		}

		self
	}
}

impl<AccountId, Balance> StakingLedger<AccountId, Balance> where
	Balance: AtLeast32Bit + Saturating + Copy,
{
	/// Slash the validator for a given amount of balance. This can grow the value
	/// of the slash in the case that the validator has less than `minimum_balance`
	/// active funds. Returns the amount of funds actually slashed.
	///
	/// Slashes from `active` funds first, and then `unlocking`, starting with the
	/// chunks that are closest to unlocking.
	fn slash(
		&mut self,
		mut value: Balance,
		minimum_balance: Balance,
	) -> Balance {
		let pre_total = self.total;
		let total = &mut self.total;
		let active = &mut self.active;

		let slash_out_of = |
			total_remaining: &mut Balance,
			target: &mut Balance,
			value: &mut Balance,
		| {
			let mut slash_from_target = (*value).min(*target);

			if !slash_from_target.is_zero() {
				*target -= slash_from_target;

				// don't leave a dust balance in the staking system.
				if *target <= minimum_balance {
					slash_from_target += *target;
					*value += sp_std::mem::replace(target, Zero::zero());
				}

				*total_remaining = total_remaining.saturating_sub(slash_from_target);
				*value -= slash_from_target;
			}
		};

		slash_out_of(total, active, &mut value);

		let i = self.unlocking.iter_mut()
			.map(|chunk| {
				slash_out_of(total, &mut chunk.value, &mut value);
				chunk.value
			})
			.take_while(|value| value.is_zero()) // take all fully-consumed chunks out.
			.count();

		// kill all drained chunks.
		let _ = self.unlocking.drain(..i);

		pre_total.saturating_sub(*total)
	}
}

/// A record of the nominations made by a specific account.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Nominations<AccountId> {
	/// The targets of nomination.
	pub targets: Vec<AccountId>,
	/// The era the nominations were submitted.
	///
	/// Except for initial nominations which are considered submitted at era 0.
	pub submitted_in: EraIndex,
	/// Whether the nominations have been suppressed.
	pub suppressed: bool,
}

/// The amount of exposure (to slashing) than an individual nominator has.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct IndividualExposure<AccountId, Balance: HasCompact> {
	/// The stash account of the nominator in question.
	who: AccountId,
	/// Amount of funds exposed.
	#[codec(compact)]
	value: Balance,
}

/// A snapshot of the stake backing a single validator in the system.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct Exposure<AccountId, Balance: HasCompact> {
	/// The total balance backing this validator.
	#[codec(compact)]
	pub total: Balance,
	/// The validator's own stash that is exposed.
	#[codec(compact)]
	pub own: Balance,
	/// The portions of nominators stashes that are exposed.
	pub others: Vec<IndividualExposure<AccountId, Balance>>,
}

/// A pending slash record. The value of the slash has been computed but not applied yet,
/// rather deferred for several eras.
#[derive(Encode, Decode, Default, RuntimeDebug)]
pub struct UnappliedSlash<AccountId, Balance: HasCompact> {
	/// The stash ID of the offending validator.
	validator: AccountId,
	/// The validator's own slash.
	own: Balance,
	/// All other slashed stakers and amounts.
	others: Vec<(AccountId, Balance)>,
	/// Reporters of the offence; bounty payout recipients.
	reporters: Vec<AccountId>,
	/// The amount of payout.
	payout: Balance,
}

/// Indicate how an election round was computed.
#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug)]
pub enum ElectionCompute {
	/// Result was forcefully computed on chain at the end of the session.
	OnChain,
	/// Result was submitted and accepted to the chain via a signed transaction.
	Signed,
	/// Result was submitted by an authority (probably via an unsigned transaction)
	Authority,
}

/// The result of an election round.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ElectionResult<AccountId, Balance: HasCompact> {
	/// Flat list of validators who have been elected.
	elected_stashes: Vec<AccountId>,
	/// Flat list of new exposures, to be updated in the [`Exposure`] storage.
	exposures: Vec<(AccountId, Exposure<AccountId, Balance>)>,
	/// Type of the result. This is kept on chain only to track and report the best score's
	/// submission type. An optimisation could remove this.
	compute: ElectionCompute,
}

/// The status of the upcoming (offchain) election.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum ElectionStatus<BlockNumber> {
	/// Nothing has and will happen for now. submission window is not open.
	Closed,
	/// The submission window has been open since the contained block number.
	Open(BlockNumber),
}

impl<BlockNumber: PartialEq> ElectionStatus<BlockNumber> {
	fn is_open_at(&self, n: BlockNumber) -> bool {
		*self == Self::Open(n)
	}

	fn is_closed(&self) -> bool {
		match self {
			Self::Closed => true,
			_ => false
		}
	}

	fn is_open(&self) -> bool {
		!self.is_closed()
	}
}

impl<BlockNumber> Default for ElectionStatus<BlockNumber> {
	fn default() -> Self {
		Self::Closed
	}
}

/// Means for interacting with a specialized version of the `session` trait.
///
/// This is needed because `Staking` sets the `ValidatorIdOf` of the `pallet_session::Trait`
pub trait SessionInterface<AccountId>: frame_system::Trait {
	/// Disable a given validator by stash ID.
	///
	/// Returns `true` if new era should be forced at the end of this session.
	/// This allows preventing a situation where there is too many validators
	/// disabled and block production stalls.
	fn disable_validator(validator: &AccountId) -> Result<bool, ()>;
	/// Get the validators from session.
	fn validators() -> Vec<AccountId>;
	/// Prune historical session tries up to but not including the given index.
	fn prune_historical_up_to(up_to: SessionIndex);
}

impl<T: Trait> SessionInterface<<T as frame_system::Trait>::AccountId> for T where
	T: pallet_session::Trait<ValidatorId = <T as frame_system::Trait>::AccountId>,
	T: pallet_session::historical::Trait<
		FullIdentification = Exposure<<T as frame_system::Trait>::AccountId, BalanceOf<T>>,
		FullIdentificationOf = ExposureOf<T>,
	>,
	T::SessionHandler: pallet_session::SessionHandler<<T as frame_system::Trait>::AccountId>,
	T::SessionManager: pallet_session::SessionManager<<T as frame_system::Trait>::AccountId>,
	T::ValidatorIdOf: Convert<<T as frame_system::Trait>::AccountId, Option<<T as frame_system::Trait>::AccountId>>,
{
	fn disable_validator(validator: &<T as frame_system::Trait>::AccountId) -> Result<bool, ()> {
		<pallet_session::Module<T>>::disable(validator)
	}

	fn validators() -> Vec<<T as frame_system::Trait>::AccountId> {
		<pallet_session::Module<T>>::validators()
	}

	fn prune_historical_up_to(up_to: SessionIndex) {
		<pallet_session::historical::Module<T>>::prune_up_to(up_to);
	}
}

pub trait Trait: frame_system::Trait {
	/// The staking balance.
	type Currency: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;

	/// Time used for computing era duration.
	///
	/// It is guaranteed to start being called from the first `on_finalize`. Thus value at genesis
	/// is not used.
	type Time: Time;

	/// Convert a balance into a number used for election calculation.
	/// This must fit into a `u64` but is allowed to be sensibly lossy.
	/// TODO: #1377
	/// The backward convert should be removed as the new Phragmen API returns ratio.
	/// The post-processing needs it but will be moved to off-chain. TODO: #2908
	type CurrencyToVote: Convert<BalanceOf<Self>, u64> + Convert<u128, BalanceOf<Self>>;

	/// Tokens have been minted and are unused for validator-reward.
	type RewardRemainder: OnUnbalanced<NegativeImbalanceOf<Self>>;

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// Handler for the unbalanced reduction when slashing a staker.
	type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

	/// Handler for the unbalanced increment when rewarding a staker.
	type Reward: OnUnbalanced<PositiveImbalanceOf<Self>>;

	/// Number of sessions per era.
	type SessionsPerEra: Get<SessionIndex>;

	/// Number of eras that staked funds must remain bonded for.
	type BondingDuration: Get<EraIndex>;

	/// Number of eras that slashes are deferred by, after computation. This
	/// should be less than the bonding duration. Set to 0 if slashes should be
	/// applied immediately, without opportunity for intervention.
	type SlashDeferDuration: Get<EraIndex>;

	/// The origin which can cancel a deferred slash. Root can always do this.
	type SlashCancelOrigin: EnsureOrigin<Self::Origin>;

	/// Interface for interacting with a session module.
	type SessionInterface: self::SessionInterface<Self::AccountId>;

	/// The NPoS reward curve to use.
	type RewardCurve: Get<&'static PiecewiseLinear<'static>>;

	/// Something that can estimate the next session change, accurately or as a best effort guess.
	type NextNewSession: EstimateNextNewSession<Self::BlockNumber>;

	/// How many blocks ahead of the era, within the last do we try to run the phragmen offchain?
	/// Setting this to zero will disable the offchain compute and only on-chain seq-phragmen will
	/// be used.
	type ElectionLookahead: Get<Self::BlockNumber>;

	/// The overarching call type.
	type Call: From<Call<Self>> + IsSubType<Module<Self>, Self> + Clone;

	/// A transaction submitter.
	type SubmitTransaction: SubmitUnsignedTransaction<Self, <Self as Trait>::Call>;

	/// Key type used to sign and verify transaction.
	type KeyType: RuntimeAppPublic + Member + Parameter + Default;

	/// The maximum number of nominator rewarded for each validator.
	///
	/// For each validator only the `$MaxNominatorRewardedPerValidator` biggest stakers can claim
	/// their reward. This used to limit the i/o cost for the nominator payout.
	type MaxNominatorRewardedPerValidator: Get<u32>;
}

/// Mode of era-forcing.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Forcing {
	/// Not forcing anything - just let whatever happen.
	NotForcing,
	/// Force a new era, then reset to `NotForcing` as soon as it is done.
	ForceNew,
	/// Avoid a new era indefinitely.
	ForceNone,
	/// Force a new era at the end of all sessions indefinitely.
	ForceAlways,
}

impl Default for Forcing {
	fn default() -> Self { Forcing::NotForcing }
}

decl_storage! {
	trait Store for Module<T: Trait> as Staking {
		/// Number of era to keep in history.
		///
		/// Information is kept for eras in `[current_era - history_depth; current_era]
		///
		/// Must be more than the number of era delayed by session otherwise.
		/// i.e. active era must always be in history.
		/// i.e. `active_era > current_era - history_depth` must be guaranteed.
		HistoryDepth get(fn history_depth) config(): u32 = 84;

		/// The ideal number of staking participants.
		pub ValidatorCount get(fn validator_count) config(): u32;

		/// Minimum number of staking participants before emergency conditions are imposed.
		pub MinimumValidatorCount get(fn minimum_validator_count) config():
			u32 = DEFAULT_MINIMUM_VALIDATOR_COUNT;

		/// Any validators that may never be slashed or forcibly kicked. It's a Vec since they're
		/// easy to initialize and the performance hit is minimal (we expect no more than four
		/// invulnerables) and restricted to testnets.
		pub Invulnerables get(fn invulnerables) config(): Vec<T::AccountId>;

		/// Map from all locked "stash" accounts to the controller account.
		pub Bonded get(fn bonded): map hasher(blake2_256) T::AccountId => Option<T::AccountId>;

		/// Map from all (unlocked) "controller" accounts to the info regarding the staking.
		pub Ledger get(fn ledger):
			map hasher(blake2_256) T::AccountId
			=> Option<StakingLedger<T::AccountId, BalanceOf<T>>>;

		/// Where the reward payment should be made. Keyed by stash.
		pub Payee get(fn payee): map hasher(blake2_256) T::AccountId => RewardDestination;

		/// The map from (wannabe) validator stash key to the preferences of that validator.
		pub Validators get(fn validators):
			linked_map hasher(blake2_256) T::AccountId => ValidatorPrefs;

		/// The map from nominator stash key to the set of stash keys of all validators to nominate.
		pub Nominators get(fn nominators):
			linked_map hasher(blake2_256) T::AccountId => Option<Nominations<T::AccountId>>;

		/// The current era index.
		///
		/// This is the latest planned era, depending on how session module queues the validator
		/// set, it might be active or not.
		pub CurrentEra get(fn current_era): Option<EraIndex>;

		/// The active era information, it holds index and start.
		///
		/// The active era is the era currently rewarded.
		/// Validator set of this era must be equal to `SessionInterface::validators`.
		pub ActiveEra get(fn active_era): Option<ActiveEraInfo<MomentOf<T>>>;

		/// The session index at which the era start for the last `HISTORY_DEPTH` eras
		pub ErasStartSessionIndex get(fn eras_start_session_index):
			map hasher(blake2_256) EraIndex => Option<SessionIndex>;

		/// Exposure of validator at era.
		///
		/// This is keyed first by the era index to allow bulk deletion and then the stash account.
		///
		/// Is it removed after `HISTORY_DEPTH` eras.
		/// If stakers hasn't been set or has been removed then empty exposure is returned.
		pub ErasStakers get(fn eras_stakers):
			double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
			=> Exposure<T::AccountId, BalanceOf<T>>;

		/// Clipped Exposure of validator at era.
		///
		/// This is similar to [`ErasStakers`] but number of nominators exposed is reduce to the
		/// `T::MaxNominatorRewardedPerValidator` biggest stakers.
		/// This is used to limit the i/o cost for the nominator payout.
		///
		/// This is keyed fist by the era index to allow bulk deletion and then the stash account.
		///
		/// Is it removed after `HISTORY_DEPTH` eras.
		/// If stakers hasn't been set or has been removed then empty exposure is returned.
		pub ErasStakersClipped get(fn eras_stakers_clipped):
			double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
			=> Exposure<T::AccountId, BalanceOf<T>>;

		/// Similarly to `ErasStakers` this holds the preferences of validators.
		///
		/// This is keyed fist by the era index to allow bulk deletion and then the stash account.
		///
		/// Is it removed after `HISTORY_DEPTH` eras.
		// If prefs hasn't been set or has been removed then 0 commission is returned.
		pub ErasValidatorPrefs get(fn eras_validator_prefs):
			double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
			=> ValidatorPrefs;

		/// The total validator era payout for the last `HISTORY_DEPTH` eras.
		///
		/// Eras that haven't finished yet or has been removed doesn't have reward.
		pub ErasValidatorReward get(fn eras_validator_reward):
			map hasher(blake2_256) EraIndex => Option<BalanceOf<T>>;

		/// Rewards for the last `HISTORY_DEPTH` eras.
		/// If reward hasn't been set or has been removed then 0 reward is returned.
		pub ErasRewardPoints get(fn eras_reward_points):
			map hasher(blake2_256) EraIndex => EraRewardPoints<T::AccountId>;

		/// The total amount staked for the last `HISTORY_DEPTH` eras.
		/// If total hasn't been set or has been removed then 0 stake is returned.
		pub ErasTotalStake get(fn eras_total_stake):
			map hasher(blake2_256) EraIndex => BalanceOf<T>;

		/// Mode of era forcing.
		pub ForceEra get(fn force_era) config(): Forcing;

		/// The percentage of the slash that is distributed to reporters.
		///
		/// The rest of the slashed value is handled by the `Slash`.
		pub SlashRewardFraction get(fn slash_reward_fraction) config(): Perbill;

		/// The amount of currency given to reporters of a slash event which was
		/// canceled by extraordinary circumstances (e.g. governance).
		pub CanceledSlashPayout get(fn canceled_payout) config(): BalanceOf<T>;

		/// All unapplied slashes that are queued for later.
		pub UnappliedSlashes:
			map hasher(blake2_256) EraIndex => Vec<UnappliedSlash<T::AccountId, BalanceOf<T>>>;

		/// A mapping from still-bonded eras to the first session index of that era.
		///
		/// Must contains information for eras for the range:
		/// `[active_era - bounding_duration; active_era]`
		BondedEras: Vec<(EraIndex, SessionIndex)>;

		/// All slashing events on validators, mapped by era to the highest slash proportion
		/// and slash value of the era.
		ValidatorSlashInEra:
			double_map hasher(blake2_256) EraIndex, hasher(twox_128) T::AccountId
			=> Option<(Perbill, BalanceOf<T>)>;

		/// All slashing events on nominators, mapped by era to the highest slash value of the era.
		NominatorSlashInEra:
			double_map hasher(blake2_256) EraIndex, hasher(twox_128) T::AccountId
			=> Option<BalanceOf<T>>;

		/// Slashing spans for stash accounts.
		SlashingSpans: map hasher(blake2_256) T::AccountId => Option<slashing::SlashingSpans>;

		/// Records information about the maximum slash of a stash within a slashing span,
		/// as well as how much reward has been paid out.
		SpanSlash:
			map hasher(blake2_256) (T::AccountId, slashing::SpanIndex)
			=> slashing::SpanRecord<BalanceOf<T>>;

		/// The earliest era for which we have a pending, unapplied slash.
		EarliestUnappliedSlash: Option<EraIndex>;

		/// Snapshot of validators at the beginning of the current election window. This should only
		/// have a value when [`EraElectionStatus`] == `ElectionStatus::Open(_)`.
		SnapshotValidators get(fn snapshot_validators): Option<Vec<T::AccountId>>;

		/// Snapshot of nominators at the beginning of the current election window. This should only
		/// have a value when [`EraElectionStatus`] == `ElectionStatus::Open(_)`.
		SnapshotNominators get(fn snapshot_nominators): Option<Vec<T::AccountId>>;

		/// The current set of staking keys.
		Keys get(fn keys): Vec<T::KeyType>;

		/// The next validator set. At the end of an era, if this is available (potentially from the
		/// result of an offchain worker), it is immediately used. Otherwise, the on-chain election
		/// is executed.
		QueuedElected get(fn queued_elected): Option<ElectionResult<T::AccountId, BalanceOf<T>>>;

		/// The score of the current [`QueuedElected`].
		QueuedScore get(fn queued_score): Option<PhragmenScore>;

		/// Flag to control the execution of the offchain election.
		EraElectionStatus get(fn era_election_status): ElectionStatus<T::BlockNumber>;

		/// True of the current planned session is final
		IsCurrentSessionFinal get(fn is_current_session_final): bool = false;

		/// True if network has been upgraded to this version.
		///
		/// True for new networks.
		IsUpgraded build(|_| true): bool;
	}
	add_extra_genesis {
		config(stakers):
			Vec<(T::AccountId, T::AccountId, BalanceOf<T>, StakerStatus<T::AccountId>)>;
		build(|config: &GenesisConfig<T>| {
			for &(ref stash, ref controller, balance, ref status) in &config.stakers {
				assert!(
					T::Currency::free_balance(&stash) >= balance,
					"Stash does not have enough balance to bond."
				);
				let _ = <Module<T>>::bond(
					T::Origin::from(Some(stash.clone()).into()),
					T::Lookup::unlookup(controller.clone()),
					balance,
					RewardDestination::Staked,
				);
				let _ = match status {
					StakerStatus::Validator => {
						<Module<T>>::validate(
							T::Origin::from(Some(controller.clone()).into()),
							Default::default(),
						)
					},
					StakerStatus::Nominator(votes) => {
						<Module<T>>::nominate(
							T::Origin::from(Some(controller.clone()).into()),
							votes.iter().map(|l| T::Lookup::unlookup(l.clone())).collect(),
						)
					}, _ => Ok(())
				};
			}
		});
	}
}

decl_event!(
	pub enum Event<T> where Balance = BalanceOf<T>, <T as frame_system::Trait>::AccountId {
		/// The staker has been rewarded by this amount. AccountId is controller account.
		Reward(AccountId, Balance),
		/// One validator (and its nominators) has been slashed by the given amount.
		Slash(AccountId, Balance),
		/// An old slashing report from a prior era was discarded because it could
		/// not be processed.
		OldSlashingReportDiscarded(SessionIndex),
		/// A new set of stakers was elected with the given computation method.
		StakingElection(ElectionCompute),
	}
);

decl_error! {
	/// Error for the staking module.
	pub enum Error for Module<T: Trait> {
		/// Not a controller account.
		NotController,
		/// Not a stash account.
		NotStash,
		/// Stash is already bonded.
		AlreadyBonded,
		/// Controller is already paired.
		AlreadyPaired,
		/// Targets cannot be empty.
		EmptyTargets,
		/// Duplicate index.
		DuplicateIndex,
		/// Slash record index out of bounds.
		InvalidSlashIndex,
		/// Can not bond with value less than minimum balance.
		InsufficientValue,
		/// Can not schedule more unlock chunks.
		NoMoreChunks,
		/// Can not rebond without unlocking chunks.
		NoUnlockChunk,
		/// Attempting to target a stash that still has funds.
		FundedTarget,
		/// Invalid era to reward.
		InvalidEraToReward,
		/// Invalid number of nominations.
		InvalidNumberOfNominations,
		/// The submitted result is received out of the open window.
		PhragmenEarlySubmission,
		/// The submitted result is not as good as the one stored on chain.
		PhragmenWeakSubmission,
		/// The snapshot data of the current window is missing.
		SnapshotUnavailable,
		/// Incorrect number of winners were presented.
		PhragmenBogusWinnerCount,
		/// One of the submitted winners is not an active candidate on chain (index is out of range
		/// in snapshot).
		PhragmenBogusWinner,
		/// Error while building the assignment type from the compact. This can happen if an index
		/// is invalid, or if the weights _overflow_.
		PhragmenBogusCompact,
		/// One of the submitted nominators is not an active nominator on chain.
		PhragmenBogusNominator,
		/// One of the submitted nominators has an edge to which they have not voted on chain.
		PhragmenBogusNomination,
		/// A self vote must only be originated from a validator to ONLY themselves.
		PhragmenBogusSelfVote,
		/// The submitted result has unknown edges that are not among the presented winners.
		PhragmenBogusEdge,
		/// The claimed score does not match with the one computed from the data.
		PhragmenBogusScore,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Number of sessions per era.
		const SessionsPerEra: SessionIndex = T::SessionsPerEra::get();

		/// Number of eras that staked funds must remain bonded for.
		const BondingDuration: EraIndex = T::BondingDuration::get();

		type Error = Error<T>;

		fn deposit_event() = default;

		/// Does the following:
		///
		/// 1. potential storage migration
		/// 2. sets `ElectionStatus` to `Open(now)` where `now` is the block number at which
		///    the election window has opened. The offchain worker, if applicable, will execute at
		///    the end of the current block. `submit_election_solution` will accept solutions from
		///    this block until the end of the era.
		fn on_initialize(now: T::BlockNumber) {
			Self::ensure_storage_upgraded();
			if
				// if we don't have any ongoing offchain compute.
				Self::era_election_status().is_closed() &&
				Self::is_current_session_final()
			{
				let next_session_change =
					T::NextNewSession::estimate_next_new_session(now);
				if let Some(remaining) = next_session_change.checked_sub(&now) {
					if remaining <= T::ElectionLookahead::get() && !remaining.is_zero() {
						// create snapshot.
						if Self::create_stakers_snapshot() {
							// Set the flag to make sure we don't waste any compute here in the same era
							// after we have triggered the offline compute.
							<EraElectionStatus<T>>::put(
								ElectionStatus::<T::BlockNumber>::Open(now)
							);
							debug::native::info!(
								target: "staking",
								"Election window is Open({:?}). Snapshot created",
								now,
							);
						} else {
							debug::native::warn!(
								target: "staking",
								"Failed to create snapshot at {:?}. Election window will remain closed.",
								now,
							);
						}

					}
				}
			}
		}

		/// Check if the current block number is the one at which the election window has been set
		/// to open. If so, it runs the offchain worker code.
		fn offchain_worker(now: T::BlockNumber) {
			use offchain_election::{set_check_offchain_execution_status, compute_offchain_election};

			let window_open = Self::era_election_status().is_open_at(now);

			if window_open {
				let offchain_status = set_check_offchain_execution_status::<T>(now);
				if let Err(why) = offchain_status {
					debug::native::warn!(
						target: "staking",
						"skipping offchain call in open election window due to [{}]",
						why,
					);
				} else {
					if let Err(e) = compute_offchain_election::<T>() {
						debug::native::warn!(
							target: "staking",
							"Error in phragmen offchain worker call: {:?}",
							e,
						);
					};
				}
			}
		}

		fn on_finalize() {
			// Set the start of the first era.
			if let Some(mut active_era) = Self::active_era() {
				if active_era.start.is_none() {
					active_era.start = Some(T::Time::now());
					<ActiveEra<T>>::put(active_era);
				}
			}
		}

		/// Take the origin account as a stash and lock up `value` of its balance. `controller` will
		/// be the account that controls it.
		///
		/// `value` must be more than the `minimum_balance` specified by `T::Currency`.
		///
		/// The dispatch origin for this call must be _Signed_ by the stash account.
		///
		/// # <weight>
		/// - Independent of the arguments. Moderate complexity.
		/// - O(1).
		/// - Three extra DB entries.
		///
		/// NOTE: Two of the storage writes (`Self::bonded`, `Self::payee`) are _never_ cleaned
		/// unless the `origin` falls below _existential deposit_ and gets removed as dust.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn bond(origin,
			controller: <T::Lookup as StaticLookup>::Source,
			#[compact] value: BalanceOf<T>,
			payee: RewardDestination,
		) {
			let stash = ensure_signed(origin)?;

			if <Bonded<T>>::contains_key(&stash) {
				Err(Error::<T>::AlreadyBonded)?
			}

			let controller = T::Lookup::lookup(controller)?;

			if <Ledger<T>>::contains_key(&controller) {
				Err(Error::<T>::AlreadyPaired)?
			}

			// reject a bond which is considered to be _dust_.
			if value < T::Currency::minimum_balance() {
				Err(Error::<T>::InsufficientValue)?
			}

			// You're auto-bonded forever, here. We might improve this by only bonding when
			// you actually validate/nominate and remove once you unbond __everything__.
			<Bonded<T>>::insert(&stash, &controller);
			<Payee<T>>::insert(&stash, payee);

			system::Module::<T>::inc_ref(&stash);

			let stash_balance = T::Currency::free_balance(&stash);
			let value = value.min(stash_balance);
			let item = StakingLedger {
				stash,
				total: value,
				active: value,
				unlocking: vec![],
				last_reward: Self::current_era(),
			};
			Self::update_ledger(&controller, &item);
		}

		/// Add some extra amount that have appeared in the stash `free_balance` into the balance up
		/// for staking.
		///
		/// Use this if there are additional funds in your stash account that you wish to bond.
		/// Unlike [`bond`] or [`unbond`] this function does not impose any limitation on the amount
		/// that can be added.
		///
		/// The dispatch origin for this call must be _Signed_ by the stash, not the controller.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - O(1).
		/// - One DB entry.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn bond_extra(origin, #[compact] max_additional: BalanceOf<T>) {
			let stash = ensure_signed(origin)?;

			let controller = Self::bonded(&stash).ok_or(Error::<T>::NotStash)?;
			let mut ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;

			let stash_balance = T::Currency::free_balance(&stash);

			if let Some(extra) = stash_balance.checked_sub(&ledger.total) {
				let extra = extra.min(max_additional);
				ledger.total += extra;
				ledger.active += extra;
				Self::update_ledger(&controller, &ledger);
			}
		}

		/// Schedule a portion of the stash to be unlocked ready for transfer out after the bond
		/// period ends. If this leaves an amount actively bonded less than
		/// T::Currency::minimum_balance(), then it is increased to the full amount.
		///
		/// Once the unlock period is done, you can call `withdraw_unbonded` to actually move
		/// the funds out of management ready for transfer.
		///
		/// No more than a limited number of unlocking chunks (see `MAX_UNLOCKING_CHUNKS`)
		/// can co-exists at the same time. In that case, [`Call::withdraw_unbonded`] need
		/// to be called first to remove some of the chunks (if possible).
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		///
		/// See also [`Call::withdraw_unbonded`].
		///
		/// # <weight>
		/// - Independent of the arguments. Limited but potentially exploitable complexity.
		/// - Contains a limited number of reads.
		/// - Each call (requires the remainder of the bonded balance to be above `minimum_balance`)
		///   will cause a new entry to be inserted into a vector (`Ledger.unlocking`) kept in storage.
		///   The only way to clean the aforementioned storage item is also user-controlled via
		///   `withdraw_unbonded`.
		/// - One DB entry.
		/// </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(400_000)]
		fn unbond(origin, #[compact] value: BalanceOf<T>) {
			let controller = ensure_signed(origin)?;
			let mut ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
			ensure!(
				ledger.unlocking.len() < MAX_UNLOCKING_CHUNKS,
				Error::<T>::NoMoreChunks,
			);

			let mut value = value.min(ledger.active);

			if !value.is_zero() {
				ledger.active -= value;

				// Avoid there being a dust balance left in the staking system.
				if ledger.active < T::Currency::minimum_balance() {
					value += ledger.active;
					ledger.active = Zero::zero();
				}

				// Note: in case there is no current era it is fine to bond one era more.
				let era = Self::current_era().unwrap_or(0) + T::BondingDuration::get();
				ledger.unlocking.push(UnlockChunk { value, era });
				Self::update_ledger(&controller, &ledger);
			}
		}

		/// Remove any unlocked chunks from the `unlocking` queue from our management.
		///
		/// This essentially frees up that balance to be used by the stash account to do
		/// whatever it wants.
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		///
		/// See also [`Call::unbond`].
		///
		/// # <weight>
		/// - Could be dependent on the `origin` argument and how much `unlocking` chunks exist.
		///  It implies `consolidate_unlocked` which loops over `Ledger.unlocking`, which is
		///  indirectly user-controlled. See [`unbond`] for more detail.
		/// - Contains a limited number of reads, yet the size of which could be large based on `ledger`.
		/// - Writes are limited to the `origin` account key.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(400_000)]
		fn withdraw_unbonded(origin) {
			let controller = ensure_signed(origin)?;
			let mut ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
			if let Some(current_era) = Self::current_era() {
				ledger = ledger.consolidate_unlocked(current_era)
			}

			if ledger.unlocking.is_empty() && ledger.active.is_zero() {
				// This account must have called `unbond()` with some value that caused the active
				// portion to fall below existential deposit + will have no more unlocking chunks
				// left. We can now safely remove this.
				let stash = ledger.stash;
				// remove all staking-related information.
				Self::kill_stash(&stash)?;
				// remove the lock.
				T::Currency::remove_lock(STAKING_ID, &stash);
			} else {
				// This was the consequence of a partial unbond. just update the ledger and move on.
				Self::update_ledger(&controller, &ledger);
			}
		}

		/// Declare the desire to validate for the origin controller.
		///
		/// Effects will be felt at the beginning of the next era.
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - Contains a limited number of reads.
		/// - Writes are limited to the `origin` account key.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(750_000)]
		fn validate(origin, prefs: ValidatorPrefs) {
			Self::ensure_storage_upgraded();

			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
			let stash = &ledger.stash;
			<Nominators<T>>::remove(stash);
			<Validators<T>>::insert(stash, prefs);
		}

		/// Declare the desire to nominate `targets` for the origin controller.
		///
		/// Effects will be felt at the beginning of the next era.
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		///
		/// # <weight>
		/// - The transaction's complexity is proportional to the size of `targets`,
		/// which is capped at CompactAssignments::LIMIT.
		/// - Both the reads and writes follow a similar pattern.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(750_000)]
		fn nominate(origin, targets: Vec<<T::Lookup as StaticLookup>::Source>) {
			Self::ensure_storage_upgraded();

			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
			let stash = &ledger.stash;
			ensure!(!targets.is_empty(), Error::<T>::EmptyTargets);
			let targets = targets.into_iter()
				.take(<CompactAssignments as VotingLimit>::LIMIT)
				.map(|t| T::Lookup::lookup(t))
				.collect::<result::Result<Vec<T::AccountId>, _>>()?;

			let nominations = Nominations {
				targets,
				// initial nominations are considered submitted at era 0. See `Nominations` doc
				submitted_in: Self::current_era().unwrap_or(0),
				suppressed: false,
			};

			<Validators<T>>::remove(stash);
			<Nominators<T>>::insert(stash, &nominations);
		}

		/// Declare no desire to either validate or nominate.
		///
		/// Effects will be felt at the beginning of the next era.
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - Contains one read.
		/// - Writes are limited to the `origin` account key.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn chill(origin) {
			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
			Self::chill_stash(&ledger.stash);
		}

		/// (Re-)set the payment target for a controller.
		///
		/// Effects will be felt at the beginning of the next era.
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - Contains a limited number of reads.
		/// - Writes are limited to the `origin` account key.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn set_payee(origin, payee: RewardDestination) {
			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
			let stash = &ledger.stash;
			<Payee<T>>::insert(stash, payee);
		}

		/// (Re-)set the controller of a stash.
		///
		/// Effects will be felt at the beginning of the next era.
		///
		/// The dispatch origin for this call must be _Signed_ by the stash, not the controller.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - Contains a limited number of reads.
		/// - Writes are limited to the `origin` account key.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(750_000)]
		fn set_controller(origin, controller: <T::Lookup as StaticLookup>::Source) {
			let stash = ensure_signed(origin)?;
			let old_controller = Self::bonded(&stash).ok_or(Error::<T>::NotStash)?;
			let controller = T::Lookup::lookup(controller)?;
			if <Ledger<T>>::contains_key(&controller) {
				Err(Error::<T>::AlreadyPaired)?
			}
			if controller != old_controller {
				<Bonded<T>>::insert(&stash, &controller);
				if let Some(l) = <Ledger<T>>::take(&old_controller) {
					<Ledger<T>>::insert(&controller, l);
				}
			}
		}

		// ----- Root calls.

		/// The ideal number of validators.
		#[weight = SimpleDispatchInfo::FixedNormal(5_000)]
		fn set_validator_count(origin, #[compact] new: u32) {
			ensure_root(origin)?;
			ValidatorCount::put(new);
		}

		/// Force there to be no new eras indefinitely.
		///
		/// # <weight>
		/// - No arguments.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(5_000)]
		fn force_no_eras(origin) {
			ensure_root(origin)?;
			ForceEra::put(Forcing::ForceNone);
		}

		/// Force there to be a new era at the end of the next session. After this, it will be
		/// reset to normal (non-forced) behaviour.
		///
		/// # <weight>
		/// - No arguments.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(5_000)]
		fn force_new_era(origin) {
			ensure_root(origin)?;
			ForceEra::put(Forcing::ForceNew);
		}

		/// Set the validators who cannot be slashed (if any).
		#[weight = SimpleDispatchInfo::FixedNormal(5_000)]
		fn set_invulnerables(origin, validators: Vec<T::AccountId>) {
			ensure_root(origin)?;
			<Invulnerables<T>>::put(validators);
		}

		/// Force a current staker to become completely unstaked, immediately.
		#[weight = SimpleDispatchInfo::FixedNormal(10_000)]
		fn force_unstake(origin, stash: T::AccountId) {
			ensure_root(origin)?;

			// remove all staking-related information.
			Self::kill_stash(&stash)?;

			// remove the lock.
			T::Currency::remove_lock(STAKING_ID, &stash);
		}

		/// Force there to be a new era at the end of sessions indefinitely.
		///
		/// # <weight>
		/// - One storage write
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(5_000)]
		fn force_new_era_always(origin) {
			ensure_root(origin)?;
			ForceEra::put(Forcing::ForceAlways);
		}

		/// Cancel enactment of a deferred slash. Can be called by either the root origin or
		/// the `T::SlashCancelOrigin`.
		/// passing the era and indices of the slashes for that era to kill.
		///
		/// # <weight>
		/// - One storage write.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
		fn cancel_deferred_slash(origin, era: EraIndex, slash_indices: Vec<u32>) {
			T::SlashCancelOrigin::try_origin(origin)
				.map(|_| ())
				.or_else(ensure_root)?;

			let mut slash_indices = slash_indices;
			slash_indices.sort_unstable();
			let mut unapplied = <Self as Store>::UnappliedSlashes::get(&era);

			for (removed, index) in slash_indices.into_iter().enumerate() {
				let index = index as usize;

				// if `index` is not duplicate, `removed` must be <= index.
				ensure!(removed <= index, Error::<T>::DuplicateIndex);

				// all prior removals were from before this index, since the
				// list is sorted.
				let index = index - removed;
				ensure!(index < unapplied.len(), Error::<T>::InvalidSlashIndex);

				unapplied.remove(index);
			}

			<Self as Store>::UnappliedSlashes::insert(&era, &unapplied);
		}

		/// Make one nominator's payout for one era.
		///
		/// - `who` is the controller account of the nominator to pay out.
		/// - `era` may not be lower than one following the most recently paid era. If it is higher,
		///   then it indicates an instruction to skip the payout of all previous eras.
		/// - `validators` is the list of all validators that `who` had exposure to during `era`.
		///   If it is incomplete, then less than the full reward will be paid out.
		///   It must not exceed `MAX_NOMINATIONS`.
		///
		/// WARNING: once an era is payed for a validator such validator can't claim the payout of
		/// previous era.
		///
		/// WARNING: Incorrect arguments here can result in loss of payout. Be very careful.
		///
		/// # <weight>
		/// - Number of storage read of `O(validators)`; `validators` is the argument of the call,
		///   and is bounded by `MAX_NOMINATIONS`.
		/// - Each storage read is `O(N)` size and decode complexity; `N` is the  maximum
		///   nominations that can be given to a single validator.
		/// - Computation complexity: `O(MAX_NOMINATIONS * logN)`; `MAX_NOMINATIONS` is the
		///   maximum number of validators that may be nominated by a single nominator, it is
		///   bounded only economically (all nominators are required to place a minimum stake).
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn payout_nominator(origin, era: EraIndex, validators: Vec<(T::AccountId, u32)>)
			-> DispatchResult
		{
			let who = ensure_signed(origin)?;
			Self::do_payout_nominator(who, era, validators)
		}

		/// Make one validator's payout for one era.
		///
		/// - `who` is the controller account of the validator to pay out.
		/// - `era` may not be lower than one following the most recently paid era. If it is higher,
		///   then it indicates an instruction to skip the payout of all previous eras.
		///
		/// WARNING: once an era is payed for a validator such validator can't claim the payout of
		/// previous era.
		///
		/// WARNING: Incorrect arguments here can result in loss of payout. Be very careful.
		///
		/// # <weight>
		/// - Time complexity: O(1).
		/// - Contains a limited number of reads and writes.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn payout_validator(origin, era: EraIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_payout_validator(who, era)
		}

		/// Rebond a portion of the stash scheduled to be unlocked.
		///
		/// # <weight>
		/// - Time complexity: O(1). Bounded by `MAX_UNLOCKING_CHUNKS`.
		/// - Storage changes: Can't increase storage, only decrease it.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(500_000)]
		fn rebond(origin, #[compact] value: BalanceOf<T>) {
			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or(Error::<T>::NotController)?;
			ensure!(
				ledger.unlocking.len() > 0,
				Error::<T>::NoUnlockChunk,
			);

			let ledger = ledger.rebond(value);
			Self::update_ledger(&controller, &ledger);
		}

		/// Set history_depth value.
		///
		/// Origin must be root.
		#[weight = SimpleDispatchInfo::FixedOperational(500_000)]
		fn set_history_depth(origin, #[compact] new_history_depth: EraIndex) {
			ensure_root(origin)?;
			if let Some(current_era) = Self::current_era() {
				HistoryDepth::mutate(|history_depth| {
					let last_kept = current_era.checked_sub(*history_depth).unwrap_or(0);
					let new_last_kept = current_era.checked_sub(new_history_depth).unwrap_or(0);
					for era_index in last_kept..new_last_kept {
						Self::clear_era_information(era_index);
					}
					*history_depth = new_history_depth
				})
			}
		}

		/// Remove all data structure concerning a staker/stash once its balance is zero.
		/// This is essentially equivalent to `withdraw_unbonded` except it can be called by anyone
		/// and the target `stash` must have no funds left.
		///
		/// This can be called from any origin.
		///
		/// - `stash`: The stash account to reap. Its balance must be zero.
		fn reap_stash(_origin, stash: T::AccountId) {
			Self::ensure_storage_upgraded();
			ensure!(T::Currency::total_balance(&stash).is_zero(), Error::<T>::FundedTarget);
			Self::kill_stash(&stash)?;
			T::Currency::remove_lock(STAKING_ID, &stash);
		}

		/// Submit a phragmen result to the chain. If the solution:
		///
		/// 1. is valid
		/// 2. has a better score than a potentially existing solution on chain
		///
		/// then, it will be put on chain.
		///
		/// A solution consists of two pieces of data:
		///
		/// 1. `winners`: a flat vector of all the winners of the round.
		/// 2. `assignments`: the compact version of an assignment vector that encodes the edge
		///    weights.
		///
		/// Both of which may be computed using [`phragmen`], or any other algorithm.
		///
		/// Additionally, the submitter must provide:
		///
		/// - The score that they claim their solution has.
		///
		/// Both validators and nominators will be represented by indices in the solution. The
		/// indices should respect the corresponding types ([`ValidatorIndex`] and
		/// [`NominatorIndex`]). Moreover, they should be valid when used to index into
		/// [`SnapshotValidators`] and [`SnapshotNominators`]. Any invalid index will cause the
		/// solution to be rejected.
		///
		/// A solution is valid if:
		///
		/// 0. It is submitted when [`EraElectionStatus`] is `Open`.
		/// 1. Its claimed score is equal to the score computed on-chain.
		/// 2. Presents the correct number of winners.
		/// 3. All indexes must be value according to the snapshot vectors. All edge values must
		///    also be correct and should not overflow the granularity of the ratio type (i.e. 256
		///    or billion).
		/// 4. For each edge, all targets are actually nominated by the voter.
		/// 5. Has correct self-votes.
		///
		/// A solutions score is consisted of 3 parameters:
		///
		/// 1. `min { support.total }` for each support of a winner. This value should be maximized.
		/// 2. `sum { support.total }` for each support of a winner. This value should be minimized.
		/// 3. `sum { support.total^2 }` for each support of a winner. This value should be
		///    minimized (to ensure less variance)
		///
		/// # <weight>
		/// E: number of edges.
		/// m: size of winner committee.
		/// n: number of nominators.
		/// d: edge degree (16 for now)
		/// v: number of on-chain validator candidates.
		///
		/// NOTE: given a solution which is reduced, we can enable a new check the ensure `|E| < n +
		/// m`.
		///
		/// major steps (all done in `check_and_replace_solution`):
		///
		/// - Storage: O(1) read `ElectionStatus`.
		/// - Storage: O(1) read `PhragmenScore`.
		/// - Storage: O(1) read `ValidatorCount`.
		/// - Storage: O(1) length read from `SnapshotValidators`.
		/// - Storage: O(v) reads of `AccountId`.
		/// - Memory: O(m) iterations to map winner index to validator id.
		/// - Storage: O(n) reads `AccountId`.
		/// - Memory: O(n + m) reads to map index to `AccountId` for un-compact.
		/// - Storage: O(e) accountid reads from `Nomination` to read correct nominations.
		/// - Storage: O(e) calls into `slashable_balance_of_extended` to convert ratio to staked.
		/// - Memory: build_support_map. O(e).
		/// - Memory: evaluate_support: O(E).
		/// - Storage: O(e) writes to `QueuedElected`.
		/// - Storage: O(1) write to `QueuedScore`
		///
		/// The weight of this call is 1/10th of the blocks total weight.
		/// # </weight>
		#[weight = SimpleDispatchInfo::FixedNormal(100_000_000)]
		pub fn submit_election_solution(
			origin,
			winners: Vec<ValidatorIndex>,
			compact_assignments: CompactAssignments,
			score: PhragmenScore,
		) {
			let _who = ensure_signed(origin)?;
			Self::check_and_replace_solution(
				winners,
				compact_assignments,
				ElectionCompute::Signed,
				score,
			)?
		}

		/// Unsigned version of `submit_election_solution`. Will only be accepted from those who are
		/// in the current validator set.
		#[weight = SimpleDispatchInfo::FixedNormal(100_000_000)]
		pub fn submit_election_solution_unsigned(
			origin,
			winners: Vec<ValidatorIndex>,
			compact_assignments: CompactAssignments,
			score: PhragmenScore,
			// already used and checked in ValidateUnsigned.
			_validator_index: u32,
			_signature: <T::KeyType as RuntimeAppPublic>::Signature,
		) {
			ensure_none(origin)?;
			Self::check_and_replace_solution(
				winners,
				compact_assignments,
				ElectionCompute::Authority,
				score,
			)?
		}
	}
}

impl<T: Trait> Module<T> {
	/// The total balance that can be slashed from a stash account as of right now.
	pub fn slashable_balance_of(stash: &T::AccountId) -> BalanceOf<T> {
		Self::bonded(stash).and_then(Self::ledger).map(|l| l.active).unwrap_or_default()
	}

	/// internal impl of [`slashable_balance_of`] that returns [`ExtendedBalance`].
	fn slashable_balance_of_extended(stash: &T::AccountId) -> ExtendedBalance {
		<T::CurrencyToVote as Convert<BalanceOf<T>, u64>>::convert(
			Self::slashable_balance_of(stash)
		) as ExtendedBalance
	}

	/// Dump the list of validators and nominators into vectors and keep them on-chain.
	///
	/// This data is used to efficiently evaluate election results. returns `true` if the operation
	/// is successful.
	fn create_stakers_snapshot() -> bool {
		let validators = <Validators<T>>::enumerate().map(|(v, _)| v).collect::<Vec<_>>();
		let mut nominators = <Nominators<T>>::enumerate().map(|(n, _)| n).collect::<Vec<_>>();

		let num_validators = validators.len();
		let num_nominators = nominators.len();
		if
			num_validators > MAX_VALIDATORS ||
			num_nominators.saturating_add(num_validators) > MAX_NOMINATORS
		{
			debug::native::warn!(
				target: "staking",
				"Snapshot size too big [{} <> {}][{} <> {}].",
				num_validators,
				MAX_VALIDATORS,
				num_nominators,
				MAX_NOMINATORS,
			);
			false
		} else {
			// all validators nominate themselves;
			nominators.extend(validators.clone());

			<SnapshotValidators<T>>::put(validators);
			<SnapshotNominators<T>>::put(nominators);
			true
		}
	}

	/// Clears both snapshots of stakers.
	fn kill_stakers_snapshot() {
		<SnapshotValidators<T>>::kill();
		<SnapshotNominators<T>>::kill();
	}

	fn do_payout_nominator(who: T::AccountId, era: EraIndex, validators: Vec<(T::AccountId, u32)>)
		-> DispatchResult
	{
		// validators len must not exceed `MAX_NOMINATIONS` to avoid querying more validator
		// exposure than necessary.
		if validators.len() > MAX_NOMINATIONS {
			return Err(Error::<T>::InvalidNumberOfNominations.into());
		}

		// Note: if era has no reward to be claimed, era may be future. better not to update
		// `nominator_ledger.last_reward` in this case.
		let era_payout = <ErasValidatorReward<T>>::get(&era)
			.ok_or_else(|| Error::<T>::InvalidEraToReward)?;

		let mut nominator_ledger = <Ledger<T>>::get(&who).ok_or_else(|| Error::<T>::NotController)?;

		if nominator_ledger.last_reward.map(|last_reward| last_reward >= era).unwrap_or(false) {
			return Err(Error::<T>::InvalidEraToReward.into());
		}

		nominator_ledger.last_reward = Some(era);
		<Ledger<T>>::insert(&who, &nominator_ledger);

		let mut reward = Perbill::zero();
		let era_reward_points = <ErasRewardPoints<T>>::get(&era);

		for (validator, nominator_index) in validators.into_iter() {
			let commission = Self::eras_validator_prefs(&era, &validator).commission;
			let validator_exposure = <ErasStakersClipped<T>>::get(&era, &validator);

			if let Some(nominator_exposure) = validator_exposure.others
				.get(nominator_index as usize)
			{
				if nominator_exposure.who != nominator_ledger.stash {
					continue;
				}

				let nominator_exposure_part = Perbill::from_rational_approximation(
					nominator_exposure.value,
					validator_exposure.total,
				);
				let validator_point = era_reward_points.individual.get(&validator)
					.map(|points| *points)
					.unwrap_or_else(|| Zero::zero());
				let validator_point_part = Perbill::from_rational_approximation(
					validator_point,
					era_reward_points.total,
				);
				reward = reward.saturating_add(
					validator_point_part
						.saturating_mul(Perbill::one().saturating_sub(commission))
						.saturating_mul(nominator_exposure_part)
				);
			}
		}

		if let Some(imbalance) = Self::make_payout(&nominator_ledger.stash, reward * era_payout) {
			Self::deposit_event(RawEvent::Reward(who, imbalance.peek()));
		}

		Ok(())
	}

	fn do_payout_validator(who: T::AccountId, era: EraIndex) -> DispatchResult {
		// Note: if era has no reward to be claimed, era may be future. better not to update
		// `ledger.last_reward` in this case.
		let era_payout = <ErasValidatorReward<T>>::get(&era)
			.ok_or_else(|| Error::<T>::InvalidEraToReward)?;

		let mut ledger = <Ledger<T>>::get(&who).ok_or_else(|| Error::<T>::NotController)?;
		if ledger.last_reward.map(|last_reward| last_reward >= era).unwrap_or(false) {
			return Err(Error::<T>::InvalidEraToReward.into());
		}

		ledger.last_reward = Some(era);
		<Ledger<T>>::insert(&who, &ledger);

		let era_reward_points = <ErasRewardPoints<T>>::get(&era);
		let commission = Self::eras_validator_prefs(&era, &ledger.stash).commission;
		let exposure = <ErasStakers<T>>::get(&era, &ledger.stash);

		let exposure_part = Perbill::from_rational_approximation(
			exposure.own,
			exposure.total,
		);
		let validator_point = era_reward_points.individual.get(&ledger.stash)
			.map(|points| *points)
			.unwrap_or_else(|| Zero::zero());
		let validator_point_part = Perbill::from_rational_approximation(
			validator_point,
			era_reward_points.total,
		);
		let reward = validator_point_part.saturating_mul(
			commission.saturating_add(
				Perbill::one().saturating_sub(commission).saturating_mul(exposure_part)
			)
		);

		if let Some(imbalance) = Self::make_payout(&ledger.stash, reward * era_payout) {
			Self::deposit_event(RawEvent::Reward(who, imbalance.peek()));
		}

		Ok(())
	}

	/// Update the ledger for a controller. This will also update the stash lock. The lock will
	/// will lock the entire funds except paying for further transactions.
	fn update_ledger(
		controller: &T::AccountId,
		ledger: &StakingLedger<T::AccountId, BalanceOf<T>>
	) {
		T::Currency::set_lock(
			STAKING_ID,
			&ledger.stash,
			ledger.total,
			WithdrawReasons::all(),
		);
		<Ledger<T>>::insert(controller, ledger);
	}

	/// Chill a stash account.
	fn chill_stash(stash: &T::AccountId) {
		<Validators<T>>::remove(stash);
		<Nominators<T>>::remove(stash);
	}

	/// Ensures storage is upgraded to most recent necessary state.
	fn ensure_storage_upgraded() {
		if !IsUpgraded::get() {
			IsUpgraded::put(true);
			Self::do_upgrade();
		}
	}

	/// Actually make a payment to a staker. This uses the currency's reward function
	/// to pay the right payee for the given staker account.
	fn make_payout(stash: &T::AccountId, amount: BalanceOf<T>) -> Option<PositiveImbalanceOf<T>> {
		let dest = Self::payee(stash);
		match dest {
			RewardDestination::Controller => Self::bonded(stash)
				.and_then(|controller|
					T::Currency::deposit_into_existing(&controller, amount).ok()
				),
			RewardDestination::Stash =>
				T::Currency::deposit_into_existing(stash, amount).ok(),
			RewardDestination::Staked => Self::bonded(stash)
				.and_then(|c| Self::ledger(&c).map(|l| (c, l)))
				.and_then(|(controller, mut l)| {
					l.active += amount;
					l.total += amount;
					let r = T::Currency::deposit_into_existing(stash, amount).ok();
					Self::update_ledger(&controller, &l);
					r
				}),
		}
	}

	/// Plan a new session potentially trigger a new era.
	fn new_session(session_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		if let Some(current_era) = Self::current_era() {
			// Initial era has been set.

			let current_era_start_session_index = Self::eras_start_session_index(current_era)
				.unwrap_or_else(|| {
					frame_support::print("Error: start_session_index must be set for current_era");
					0
				});

			let era_length = session_index.checked_sub(current_era_start_session_index)
				.unwrap_or(0); // Must never happen.

			match ForceEra::get() {
				Forcing::ForceNew => ForceEra::kill(),
				Forcing::ForceAlways => (),
				Forcing::NotForcing if era_length >= T::SessionsPerEra::get() => (),
				_ => {
					// not forcing, not a new era either. If final, set the flag.
					if era_length + 1 >= T::SessionsPerEra::get() {
						IsCurrentSessionFinal::put(true);
					}
					return None
				},
			}

			// new era.
			IsCurrentSessionFinal::put(false);
			Self::new_era(session_index)
		} else {
			// Set initial era
			Self::new_era(session_index)
		}
	}

	/// Checks a given solution and if correct and improved, writes it on chain as the queued result
	/// of the next round. This may be called by both a signed and an unsigned transaction.
	fn check_and_replace_solution(
		winners: Vec<ValidatorIndex>,
		compact_assignments: CompactAssignments,
		compute: ElectionCompute,
		claimed_score: PhragmenScore,
	) -> Result<(), Error<T>> {
		// discard early solutions
		ensure!(
			Self::era_election_status().is_open(),
			Error::<T>::PhragmenEarlySubmission,
		);

		// assume the given score is valid. Is it better than what we have on-chain, if we have any?
		if let Some(queued_score) = Self::queued_score() {
			ensure!(
				is_score_better(queued_score, claimed_score),
				Error::<T>::PhragmenWeakSubmission,
			)
		}

		// Check that the number of presented winners is sane. Most often we have more candidates
		// that we need. Then it should be Self::validator_count(). Else it should be all the
		// candidates.
		let snapshot_length = <SnapshotValidators<T>>::decode_len()
			.map_err(|_| Error::<T>::SnapshotUnavailable)?;
		let desired_winners = Self::validator_count().min(snapshot_length as u32);
		ensure!(winners.len() as u32 == desired_winners, Error::<T>::PhragmenBogusWinnerCount);

		// decode snapshot validators.
		let snapshot_validators = Self::snapshot_validators()
			.ok_or(Error::<T>::SnapshotUnavailable)?;

		// check if all winners were legit; this is rather cheap. Replace with accountId.
		let winners = winners.into_iter().map(|widx| {
			snapshot_validators.get(widx as usize).cloned().ok_or(Error::<T>::PhragmenBogusWinner)
		}).collect::<Result<Vec<T::AccountId>, Error<T>>>()?;

		// decode the rest of the snapshot.
		let snapshot_nominators = <Module<T>>::snapshot_nominators()
			.ok_or(Error::<T>::SnapshotUnavailable)?;

		// helpers
		let nominator_at = |i: NominatorIndex| -> Option<T::AccountId> {
			snapshot_nominators.get(i as usize).cloned()
		};
		let validator_at = |i: ValidatorIndex| -> Option<T::AccountId> {
			snapshot_validators.get(i as usize).cloned()
		};

		// un-compact.
		let assignments = compact_assignments.into_assignment(
			nominator_at,
			validator_at,
		).map_err(|e| {
			debug::native::warn!(
				target: "staking",
				"un-compacting solution failed due to {:?}",
				e,
			);
			Error::<T>::PhragmenBogusCompact
		})?;

		// check all nominators actually including the claimed vote. Also check correct self votes.
		// Note that we assume all validators and nominators in `assignments` are properly bonded,
		// because they are coming from the snapshot via a given index.
		for Assignment { who, distribution } in assignments.iter() {
			let is_validator = <Validators<T>>::contains_key(&who);
			let maybe_nomination = Self::nominators(&who);

			if !(maybe_nomination.is_some() ^ is_validator) {
				// all of the indices must map to either a validator or a nominator. If this is ever
				// not the case, then the locking system of staking is most likely faulty, or we
				// have bigger problems.
				debug::native::error!(
					target: "staking",
					"detected an error in the staking locking and snapshot."
				);
				// abort.
				return Err(Error::<T>::PhragmenBogusNominator);
			}

			if !is_validator {
				// a normal vote
				let nomination = maybe_nomination.expect(
					"exactly one of maybe_validator and maybe_nomination is true. \
					is_validator is false; maybe_nomination is some; qed"
				);
				// NOTE: we don't really have to check here if the sum of all edges are the
				// nominator correct. Un-compacting assures this by definition.
				ensure!(
					// each target in the provided distribution must be actually nominated by the
					// nominator after the last non-zero slash.
					distribution.into_iter().all(|(t, _)| {
						nomination.targets.iter().find(|&tt| tt == t).is_some()
						&&
						<Self as Store>::SlashingSpans::get(&t).map_or(
							true,
							|spans| nomination.submitted_in >= spans.last_nonzero_slash(),
						)
					}),
					Error::<T>::PhragmenBogusNomination,
				);
			} else {
				// a self vote
				ensure!(distribution.len() == 1, Error::<T>::PhragmenBogusSelfVote);
				ensure!(distribution[0].0 == *who, Error::<T>::PhragmenBogusSelfVote);
				// defensive only. A compact assignment of length one does NOT encode the weight and
				// it is always created to be 100%.
				ensure!(
					distribution[0].1 == OffchainAccuracy::one(),
					Error::<T>::PhragmenBogusSelfVote,
				);
			}
		}

		// convert into staked assignments.
		let staked_assignments = sp_phragmen::assignment_ratio_to_staked(
			assignments,
			Self::slashable_balance_of_extended,
		);

		// build the support map thereof in order to evaluate.
		// OPTIMIZATION: loop to create the staked assignments but it would bloat the code. Okay for
		// now as it does not add to the complexity order.
		let (supports, num_error) = build_support_map::<T::AccountId>(
			&winners,
			&staked_assignments,
		);
		// This technically checks that all targets in all nominators were among the winners.
		ensure!(num_error == 0, Error::<T>::PhragmenBogusEdge);

		// Check if the score is the same as the claimed one.
		let submitted_score = evaluate_support(&supports);
		ensure!(submitted_score == claimed_score, Error::<T>::PhragmenBogusScore);

		// At last, alles Ok. Exposures and store the result.
		let exposures = Self::collect_exposure(supports);

		debug::native::info!(
			target: "staking",
			"A better solution has been validated and stored on chain.",
		);

		<QueuedElected<T>>::put(ElectionResult {
			elected_stashes: winners,
			compute,
			exposures,
		});
		QueuedScore::put(submitted_score);

		Ok(())

	}

	/// Start a session potentially starting an era.
	fn start_session(start_session: SessionIndex) {
		let next_active_era = Self::active_era().map(|e| e.index + 1).unwrap_or(0);
		if let Some(next_active_era_start_session_index) =
			Self::eras_start_session_index(next_active_era)
		{
			if next_active_era_start_session_index == start_session {
				Self::start_era(start_session);
			} else if next_active_era_start_session_index < start_session {
				// This arm should never happen, but better handle it than to stall the
				// staking pallet.
				frame_support::print("Warning: A session appears to have been skipped.");
				Self::start_era(start_session);
			}
		}
	}

	/// End a session potentially ending an era.
	fn end_session(session_index: SessionIndex) {
		if let Some(active_era) = Self::active_era() {
			let next_active_era_start_session_index =
				Self::eras_start_session_index(active_era.index + 1)
					.unwrap_or_else(|| {
						frame_support::print(
							"Error: start_session_index must be set for active_era + 1"
						);
						0
					});

			if next_active_era_start_session_index == session_index + 1 {
				Self::end_era(active_era, session_index);
			}
		}
	}

	/// * Increment `active_era.index`,
	/// * reset `active_era.start`,
	/// * update `BondedEras` and apply slashes.
	fn start_era(start_session: SessionIndex) {
		let active_era = <ActiveEra<T>>::mutate(|active_era| {
			let new_index = active_era.as_ref().map(|info| info.index + 1).unwrap_or(0);
			*active_era = Some(ActiveEraInfo {
				index: new_index,
				// Set new active era start in next `on_finalize`. To guarantee usage of `Time`
				start: None,
			});
			new_index
		});

		let bonding_duration = T::BondingDuration::get();

		BondedEras::mutate(|bonded| {
			bonded.push((active_era, start_session));

			if active_era > bonding_duration {
				let first_kept = active_era - bonding_duration;

				// prune out everything that's from before the first-kept index.
				let n_to_prune = bonded.iter()
					.take_while(|&&(era_idx, _)| era_idx < first_kept)
					.count();

				// kill slashing metadata.
				for (pruned_era, _) in bonded.drain(..n_to_prune) {
					slashing::clear_era_metadata::<T>(pruned_era);
				}

				if let Some(&(_, first_session)) = bonded.first() {
					T::SessionInterface::prune_historical_up_to(first_session);
				}
			}
		});

		Self::apply_unapplied_slashes(active_era);
	}

	/// Compute payout for era.
	fn end_era(active_era: ActiveEraInfo<MomentOf<T>>, _session_index: SessionIndex) {
		// Note: active_era_start can be None if end era is called during genesis config.
		if let Some(active_era_start) = active_era.start {
			let now = T::Time::now();

			let era_duration = now - active_era_start;
			let (total_payout, _max_payout) = inflation::compute_total_payout(
				&T::RewardCurve::get(),
				Self::eras_total_stake(&active_era.index),
				T::Currency::total_issuance(),
				// Duration of era; more than u64::MAX is rewarded as u64::MAX.
				era_duration.saturated_into::<u64>(),
			);

			// Set ending era reward.
			<ErasValidatorReward<T>>::insert(&active_era.index, total_payout);
		}
	}

	/// Plan a new era. Return the potential new staking set.
	fn new_era(start_session_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		// Increment or set current era.
		let current_era = CurrentEra::mutate(|s| {
			*s = Some(s.map(|s| s + 1).unwrap_or(0));
			s.unwrap()
		});
		ErasStartSessionIndex::insert(&current_era, &start_session_index);

		// Clean old era information.
		if let Some(old_era) = current_era.checked_sub(Self::history_depth() + 1) {
			Self::clear_era_information(old_era);
		}

		// Set staking information for new era.
		let maybe_new_validators = Self::select_and_update_validators(current_era);

		maybe_new_validators
	}

	/// Select the new validator set at the end of the era.
	///
	/// Runs [`try_do_phragmen`] and updates the following storage items:
	/// - [`EraElectionStatus`]: with `None`.
	/// - [`ErasStakers`]: with the new staker set.
	/// - [`ErasStakersClipped`].
	/// - [`ErasValidatorPrefs`].
	/// - [`ErasTotalStake`]: with the new total stake.
	/// - [`SnapshotValidators`] and [`SnapshotNominators`] are both removed.
	///
	/// Internally, [`QueuedElected`], snapshots and [`QueuedScore`] are also consumed.
	///
	/// If the election has been successful, It passes the new set upwards.
	///
	/// This should only be called at the end of an era.
	fn select_and_update_validators(current_era: EraIndex) -> Option<Vec<T::AccountId>> {
		if let Some(ElectionResult::<T::AccountId, BalanceOf<T>> {
			elected_stashes,
			exposures,
			compute,
		}) = Self::try_do_phragmen() {
			// We have chosen the new validator set. Submission is no longer allowed.
			<EraElectionStatus<T>>::put(ElectionStatus::Closed);

			// kill the snapshots.
			Self::kill_stakers_snapshot();

			// Populate Stakers and write slot stake.
			let mut total_stake: BalanceOf<T> = Zero::zero();
			exposures.into_iter().for_each(|(stash, exposure)| {
				total_stake = total_stake.saturating_add(exposure.total);
				<ErasStakers<T>>::insert(current_era, &stash, &exposure);

				let mut exposure_clipped = exposure;
				let clipped_max_len = T::MaxNominatorRewardedPerValidator::get() as usize;
				if exposure_clipped.others.len() > clipped_max_len {
					exposure_clipped.others.sort_unstable_by(|a, b| a.value.cmp(&b.value).reverse());
					exposure_clipped.others.truncate(clipped_max_len);
				}
				<ErasStakersClipped<T>>::insert(&current_era, &stash, exposure_clipped);
			});

			// Insert current era staking information
			<ErasTotalStake<T>>::insert(&current_era, total_stake);

			// --------
			// TODO: this snapshot need to be taken elsewhere... this is super inefficient now.
			// The current abstraction is such that we do `<Validators<T>>::enumerate()` down to line
			// in `do_phragmen` and don't really update the values there. There are numerous ways to fix this.
			// check @guillaume.
			let mut all_validators_and_prefs = BTreeMap::new();
			for (validator, preference) in <Validators<T>>::enumerate() {
				all_validators_and_prefs.insert(validator.clone(), preference);
			}
			// ---------

			let default_pref = ValidatorPrefs::default();
			for stash in &elected_stashes {
				let pref = all_validators_and_prefs.get(stash)
					.unwrap_or(&default_pref); // Must never happen, but better to be safe.
				<ErasValidatorPrefs<T>>::insert(&current_era, stash, pref);
			}

			// emit event
			Self::deposit_event(RawEvent::StakingElection(compute));

			debug::native::info!(
				target: "staking",
				"new validator set of size {:?} has been elected via {:?} for era {:?}",
				elected_stashes.len(),
				compute,
				current_era,
			);

			Some(elected_stashes)
		} else {
			None
		}
	}

	/// Select a new validator set from the assembled stakers and their role preferences. It tries
	/// first to peek into [`QueuedElected`]. Otherwise, it runs a new phragmen.
	///
	/// If [`QueuedElected`] and [`QueuedScore`] exists, they are both removed. No further storage
	/// is updated.
	fn try_do_phragmen() -> Option<ElectionResult<T::AccountId, BalanceOf<T>>> {
		// a phragmen result from either a stored submission or locally executed one.
		let next_result = <QueuedElected<T>>::take().or_else(||
			Self::do_phragmen_with_post_processing::<ChainAccuracy>(ElectionCompute::OnChain)
		);

		// either way, kill this. We remove it here to make sure it always has the exact same
		// lifetime as `QueuedElected`.
		QueuedScore::kill();

		next_result
	}

	/// Execute phragmen and return the new results. The edge weights are processed into support
	/// values.
	///
	/// This is basically a wrapper around [`do_phragmen`] which translates `PhragmenResult` into
	/// `ElectionResult`.
	///
	/// No storage item is updated.
	fn do_phragmen_with_post_processing<Accuracy: PerThing>(compute: ElectionCompute)
	-> Option<ElectionResult<T::AccountId, BalanceOf<T>>>
		where
			Accuracy: sp_std::ops::Mul<ExtendedBalance, Output=ExtendedBalance>,
			ExtendedBalance: From<<Accuracy as PerThing>::Inner>,
	{
		if let Some(phragmen_result) = Self::do_phragmen::<Accuracy>() {
			let elected_stashes = phragmen_result.winners.iter()
				.map(|(s, _)| s.clone())
				.collect::<Vec<T::AccountId>>();
			let assignments = phragmen_result.assignments;

			let staked_assignments = sp_phragmen::assignment_ratio_to_staked(
				assignments,
				Self::slashable_balance_of_extended,
			);

			let (supports, _) = build_support_map::<T::AccountId>(
				&elected_stashes,
				&staked_assignments,
			);

			// collect exposures
			let exposures = Self::collect_exposure(supports);

			// In order to keep the property required by `n_session_ending` that we must return the
			// new validator set even if it's the same as the old, as long as any underlying
			// economic conditions have changed, we don't attempt to do any optimization where we
			// compare against the prior set.
			Some(ElectionResult::<T::AccountId, BalanceOf<T>> {
				elected_stashes,
				exposures,
				compute,
			})
		} else {
			// There were not enough candidates for even our minimal level of functionality. This is
			// bad. We should probably disable all functionality except for block production and let
			// the chain keep producing blocks until we can decide on a sufficiently substantial
			// set. TODO: #2494
			None
		}
	}

	/// Execute phragmen and return the new results. No post-processing is applied and the raw edge
	/// weights are returned.
	///
	/// Self votes are added and nominations before the most recent slashing span are reaped.
	///
	/// No storage item is updated.
	fn do_phragmen<Accuracy: PerThing>() -> Option<PhragmenResult<T::AccountId, Accuracy>> {
		let mut all_nominators: Vec<(T::AccountId, Vec<T::AccountId>)> = Vec::new();
		let all_validators = <Validators<T>>::enumerate().map(|(who, _pref)| {
			// append self vote
			let self_vote = (who.clone(), vec![who.clone()]);
			all_nominators.push(self_vote);

			who
		}).collect::<Vec<T::AccountId>>();

		let nominator_votes = <Nominators<T>>::enumerate().map(|(nominator, nominations)| {
			let Nominations { submitted_in, mut targets, suppressed: _ } = nominations;

			// Filter out nomination targets which were nominated before the most recent
			// slashing span.
			targets.retain(|stash| {
				<Self as Store>::SlashingSpans::get(&stash).map_or(
					true,
					|spans| submitted_in >= spans.last_nonzero_slash(),
				)
			});

			(nominator, targets)
		});
		all_nominators.extend(nominator_votes);

		elect::<_, _, _, T::CurrencyToVote, Accuracy>(
			Self::validator_count() as usize,
			Self::minimum_validator_count().max(1) as usize,
			all_validators,
			all_nominators,
			Self::slashable_balance_of,
		)
	}

	/// Consume a set of [`Supports`] from [`sp_phragmen`] and collect them into a [`Exposure`]
	fn collect_exposure(supports: SupportMap<T::AccountId>) -> Vec<(T::AccountId, Exposure<T::AccountId, BalanceOf<T>>)> {
		let to_balance = |e: ExtendedBalance|
			<T::CurrencyToVote as Convert<ExtendedBalance, BalanceOf<T>>>::convert(e);

		supports.into_iter().map(|(validator, support)| {
			// build `struct exposure` from `support`
			let mut others = Vec::new();
			let mut own: BalanceOf<T> = Zero::zero();
			let mut total: BalanceOf<T> = Zero::zero();
			support.voters
				.into_iter()
				.map(|(nominator, weight)| (nominator, to_balance(weight)))
				.for_each(|(nominator, stake)| {
					if nominator == validator {
						own = own.saturating_add(stake);
					} else {
						others.push(IndividualExposure { who: nominator, value: stake });
					}
					total = total.saturating_add(stake);
				});

			let exposure = Exposure {
				own,
				others,
				total,
			};

			(validator, exposure)
		}).collect::<Vec<(T::AccountId, Exposure<_, _>)>>()
	}

	/// Remove all associated data of a stash account from the staking system.
	///
	/// Assumes storage is upgraded before calling.
	///
	/// This is called:
	/// - after a `withdraw_unbond()` call that frees all of a stash's bonded balance.
	/// - through `reap_stash()` if the balance has fallen to zero (through slashing).
	fn kill_stash(stash: &T::AccountId) -> DispatchResult {
		let controller = Bonded::<T>::take(stash).ok_or(Error::<T>::NotStash)?;
		<Ledger<T>>::remove(&controller);

		<Payee<T>>::remove(stash);
		<Validators<T>>::remove(stash);
		<Nominators<T>>::remove(stash);

		slashing::clear_stash_metadata::<T>(stash);

		system::Module::<T>::dec_ref(stash);

		Ok(())
	}

	/// Clear all era information for given era.
	fn clear_era_information(era_index: EraIndex) {
		<ErasStakers<T>>::remove_prefix(era_index);
		<ErasStakersClipped<T>>::remove_prefix(era_index);
		<ErasValidatorPrefs<T>>::remove_prefix(era_index);
		<ErasValidatorReward<T>>::remove(era_index);
		<ErasRewardPoints<T>>::remove(era_index);
		<ErasTotalStake<T>>::remove(era_index);
		ErasStartSessionIndex::remove(era_index);
	}

	/// Apply previously-unapplied slashes on the beginning of a new era, after a delay.
	fn apply_unapplied_slashes(active_era: EraIndex) {
		let slash_defer_duration = T::SlashDeferDuration::get();
		<Self as Store>::EarliestUnappliedSlash::mutate(|earliest| if let Some(ref mut earliest) = earliest {
			let keep_from = active_era.saturating_sub(slash_defer_duration);
			for era in (*earliest)..keep_from {
				let era_slashes = <Self as Store>::UnappliedSlashes::take(&era);
				for slash in era_slashes {
					slashing::apply_slash::<T>(slash);
				}
			}

			*earliest = (*earliest).max(keep_from)
		})
	}

	/// Add reward points to validators using their stash account ID.
	///
	/// Validators are keyed by stash account ID and must be in the current elected set.
	///
	/// For each element in the iterator the given number of points in u32 is added to the
	/// validator, thus duplicates are handled.
	///
	/// At the end of the era each the total payout will be distributed among validator
	/// relatively to their points.
	///
	/// COMPLEXITY: Complexity is `number_of_validator_to_reward x current_elected_len`.
	/// If you need to reward lots of validator consider using `reward_by_indices`.
	pub fn reward_by_ids(
		validators_points: impl IntoIterator<Item = (T::AccountId, u32)>
	) {
		if let Some(active_era) = Self::active_era() {
			<ErasRewardPoints<T>>::mutate(active_era.index, |era_rewards| {
				for (validator, points) in validators_points.into_iter() {
					*era_rewards.individual.entry(validator).or_default() += points;
					era_rewards.total += points;
				}
			});
		}
	}

	/// Ensures that at the end of the current session there will be a new era.
	fn ensure_new_era() {
		match ForceEra::get() {
			Forcing::ForceAlways | Forcing::ForceNew => (),
			_ => ForceEra::put(Forcing::ForceNew),
		}
	}

	/// Update storages to current version
	///
	/// In old version the staking module has several issue about handling session delay, the
	/// current era was always considered the active one.
	///
	/// After the migration the current era will still be considered the active one for the era of
	/// the upgrade. And the delay issue will be fixed when planning the next era.
	// * create:
	//   * ActiveEraStart
	//   * ErasRewardPoints
	//   * ActiveEra
	//   * ErasStakers
	//   * ErasStakersClipped
	//   * ErasValidatorPrefs
	//   * ErasTotalStake
	//   * ErasStartSessionIndex
	// * translate StakingLedger
	// * removal of:
	//   * Stakers
	//   * SlotStake
	//   * CurrentElected
	//   * CurrentEraStart
	//   * CurrentEraStartSessionIndex
	//   * CurrentEraPointsEarned
	fn do_upgrade() {
		/// Deprecated storages used for migration only.
		mod deprecated {
			use crate::{Trait, BalanceOf, MomentOf, SessionIndex, Exposure};
			use codec::{Encode, Decode};
			use frame_support::{decl_module, decl_storage};
			use sp_std::prelude::*;

			/// Reward points of an era. Used to split era total payout between validators.
			#[derive(Encode, Decode, Default)]
			pub struct EraPoints {
				/// Total number of points. Equals the sum of reward points for each validator.
				pub total: u32,
				/// The reward points earned by a given validator. The index of this vec corresponds to the
				/// index into the current validator set.
				pub individual: Vec<u32>,
			}

			decl_module! {
				pub struct Module<T: Trait> for enum Call where origin: T::Origin { }
			}

			decl_storage! {
				pub trait Store for Module<T: Trait> as Staking {
					pub SlotStake: BalanceOf<T>;

					/// The currently elected validator set keyed by stash account ID.
					pub CurrentElected: Vec<T::AccountId>;

					/// The start of the current era.
					pub CurrentEraStart: MomentOf<T>;

					/// The session index at which the current era started.
					pub CurrentEraStartSessionIndex: SessionIndex;

					/// Rewards for the current era. Using indices of current elected set.
					pub CurrentEraPointsEarned: EraPoints;

					/// Nominators for a particular account that is in action right now. You can't iterate
					/// through validators here, but you can find them in the Session module.
					///
					/// This is keyed by the stash account.
					pub Stakers: map hasher(blake2_256) T::AccountId => Exposure<T::AccountId, BalanceOf<T>>;
				}
			}
		}

		#[derive(Encode, Decode)]
		struct OldStakingLedger<AccountId, Balance: HasCompact> {
			stash: AccountId,
			#[codec(compact)]
			total: Balance,
			#[codec(compact)]
			active: Balance,
			unlocking: Vec<UnlockChunk<Balance>>,
		}

		let current_era_start_index = deprecated::CurrentEraStartSessionIndex::get();
		let current_era = <Module<T> as Store>::CurrentEra::get().unwrap_or(0);
		let current_era_start = deprecated::CurrentEraStart::<T>::get();
		<Module<T> as Store>::ErasStartSessionIndex::insert(current_era, current_era_start_index);
		<Module<T> as Store>::ActiveEra::put(ActiveEraInfo {
			index: current_era,
			start: Some(current_era_start),
		});

		let current_elected = deprecated::CurrentElected::<T>::get();
		let mut current_total_stake = <BalanceOf<T>>::zero();
		for validator in &current_elected {
			let exposure = deprecated::Stakers::<T>::get(validator);
			current_total_stake += exposure.total;
			<Module<T> as Store>::ErasStakers::insert(current_era, validator, &exposure);

			let mut exposure_clipped = exposure;
			let clipped_max_len = T::MaxNominatorRewardedPerValidator::get() as usize;
			if exposure_clipped.others.len() > clipped_max_len {
				exposure_clipped.others.sort_unstable_by(|a, b| a.value.cmp(&b.value).reverse());
				exposure_clipped.others.truncate(clipped_max_len);
			}
			<Module<T> as Store>::ErasStakersClipped::insert(current_era, validator, exposure_clipped);

			let pref = <Module<T> as Store>::Validators::get(validator);
			<Module<T> as Store>::ErasValidatorPrefs::insert(current_era, validator, pref);
		}
		<Module<T> as Store>::ErasTotalStake::insert(current_era, current_total_stake);

		let points = deprecated::CurrentEraPointsEarned::get();
		<Module<T> as Store>::ErasRewardPoints::insert(current_era, EraRewardPoints {
			total: points.total,
			individual: current_elected.iter().cloned().zip(points.individual.iter().cloned()).collect(),
		});

		let res = <Module<T> as Store>::Ledger::translate_values(
			|old: OldStakingLedger<T::AccountId, BalanceOf<T>>| StakingLedger {
				stash: old.stash,
				total: old.total,
				active: old.active,
				unlocking: old.unlocking,
				last_reward: None,
			}
		);
		if let Err(e) = res {
			frame_support::print("Encountered error in migration of Staking::Ledger map.");
			frame_support::print("The number of removed key/value is:");
			frame_support::print(e);
		}


		// Kill old storages
		deprecated::Stakers::<T>::remove_all();
		deprecated::SlotStake::<T>::kill();
		deprecated::CurrentElected::<T>::kill();
		deprecated::CurrentEraStart::<T>::kill();
		deprecated::CurrentEraStartSessionIndex::kill();
		deprecated::CurrentEraPointsEarned::kill();
	}
}

/// In this implementation `new_session(session)` must be called before `end_session(session-1)`
/// i.e. the new session must be planned before the ending of the previous session.
///
/// Once the first new_session is planned, all session must start and then end in order, though
/// some session can lag in between the newest session planned and the latest session started.
impl<T: Trait> pallet_session::SessionManager<T::AccountId> for Module<T> {
	fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		Self::ensure_storage_upgraded();
		Self::new_session(new_index)
	}
	fn start_session(start_index: SessionIndex) {
		Self::start_session(start_index)
	}
	fn end_session(end_index: SessionIndex) {
		Self::end_session(end_index)
	}
}

impl<T: Trait> historical::SessionManager<T::AccountId, Exposure<T::AccountId, BalanceOf<T>>> for Module<T> {
	fn new_session(new_index: SessionIndex)
		-> Option<Vec<(T::AccountId, Exposure<T::AccountId, BalanceOf<T>>)>>
	{
		<Self as pallet_session::SessionManager<_>>::new_session(new_index).map(|validators| {
			let current_era = Self::current_era()
				// Must be some as a new era has been created.
				.unwrap_or(0);

			validators.into_iter().map(|v| {
				let exposure = Self::eras_stakers(current_era, &v);
				(v, exposure)
			}).collect()
		})
	}
	fn start_session(start_index: SessionIndex) {
		<Self as pallet_session::SessionManager<_>>::start_session(start_index)
	}
	fn end_session(end_index: SessionIndex) {
		<Self as pallet_session::SessionManager<_>>::end_session(end_index)
	}
}

/// Add reward points to block authors:
/// * 20 points to the block producer for producing a (non-uncle) block in the relay chain,
/// * 2 points to the block producer for each reference to a previously unreferenced uncle, and
/// * 1 point to the producer of each referenced uncle block.
impl<T> pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for Module<T>
	where
		T: Trait + pallet_authorship::Trait + pallet_session::Trait
{
	fn note_author(author: T::AccountId) {
		Self::reward_by_ids(vec![(author, 20)])
	}
	fn note_uncle(author: T::AccountId, _age: T::BlockNumber) {
		Self::reward_by_ids(vec![
			(<pallet_authorship::Module<T>>::author(), 2),
			(author, 1)
		])
	}
}

/// A `Convert` implementation that finds the stash of the given controller account,
/// if any.
pub struct StashOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Trait> Convert<T::AccountId, Option<T::AccountId>> for StashOf<T> {
	fn convert(controller: T::AccountId) -> Option<T::AccountId> {
		<Module<T>>::ledger(&controller).map(|l| l.stash)
	}
}

/// A typed conversion from stash account ID to the active exposure of nominators
/// on that account.
///
/// Active exposure is the exposure of the validator set currently validating, i.e. in
/// `active_era`. It can differ from the latest planned exposure in `current_era`.
pub struct ExposureOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Trait> Convert<T::AccountId, Option<Exposure<T::AccountId, BalanceOf<T>>>>
	for ExposureOf<T>
{
	fn convert(validator: T::AccountId) -> Option<Exposure<T::AccountId, BalanceOf<T>>> {
		if let Some(active_era) = <Module<T>>::active_era() {
			Some(<Module<T>>::eras_stakers(active_era.index, &validator))
		} else {
			None
		}
	}
}

/// This is intended to be used with `FilterHistoricalOffences`.
impl <T: Trait> OnOffenceHandler<T::AccountId, pallet_session::historical::IdentificationTuple<T>> for Module<T> where
	T: pallet_session::Trait<ValidatorId = <T as frame_system::Trait>::AccountId>,
	T: pallet_session::historical::Trait<
		FullIdentification = Exposure<<T as frame_system::Trait>::AccountId, BalanceOf<T>>,
		FullIdentificationOf = ExposureOf<T>,
	>,
	T::SessionHandler: pallet_session::SessionHandler<<T as frame_system::Trait>::AccountId>,
	T::SessionManager: pallet_session::SessionManager<<T as frame_system::Trait>::AccountId>,
	T::ValidatorIdOf: Convert<<T as frame_system::Trait>::AccountId, Option<<T as frame_system::Trait>::AccountId>>
{
	fn on_offence(
		offenders: &[OffenceDetails<T::AccountId, pallet_session::historical::IdentificationTuple<T>>],
		slash_fraction: &[Perbill],
		slash_session: SessionIndex,
	) -> Result<(), ()> {
		<Module<T>>::ensure_storage_upgraded();

		if !Self::can_report() {
			return Err(())
		}

		let reward_proportion = SlashRewardFraction::get();

		let active_era = {
			let active_era = Self::active_era();
			if active_era.is_none() {
				// this offence need not be re-submitted.
				return Ok(())
			}
			active_era.expect("value checked not to be `None`; qed").index
		};
		let active_era_start_session_index = Self::eras_start_session_index(active_era)
			.unwrap_or_else(|| {
				frame_support::print("Error: start_session_index must be set for current_era");
				0
			});

		let window_start = active_era.saturating_sub(T::BondingDuration::get());

		// fast path for active-era report - most likely.
		// `slash_session` cannot be in a future active era. It must be in `active_era` or before.
		let slash_era = if slash_session >= active_era_start_session_index {
			active_era
		} else {
			let eras = BondedEras::get();

			// reverse because it's more likely to find reports from recent eras.
			match eras.iter().rev().filter(|&&(_, ref sesh)| sesh <= &slash_session).next() {
				None => return Ok(()), // before bonding period. defensive - should be filtered out.
				Some(&(ref slash_era, _)) => *slash_era,
			}
		};

		<Self as Store>::EarliestUnappliedSlash::mutate(|earliest| {
			if earliest.is_none() {
				*earliest = Some(active_era)
			}
		});

		let slash_defer_duration = T::SlashDeferDuration::get();

		for (details, slash_fraction) in offenders.iter().zip(slash_fraction) {
			let (stash, exposure) = &details.offender;

			// Skip if the validator is invulnerable.
			if Self::invulnerables().contains(stash) {
				continue
			}

			let unapplied = slashing::compute_slash::<T>(slashing::SlashParams {
				stash,
				slash: *slash_fraction,
				exposure,
				slash_era,
				window_start,
				now: active_era,
				reward_proportion,
			});

			if let Some(mut unapplied) = unapplied {
				unapplied.reporters = details.reporters.clone();
				if slash_defer_duration == 0 {
					// apply right away.
					slashing::apply_slash::<T>(unapplied);
				} else {
					// defer to end of some `slash_defer_duration` from now.
					<Self as Store>::UnappliedSlashes::mutate(
						active_era,
						move |for_later| for_later.push(unapplied),
					);
				}
			}
		}

		Ok(())
	}

	fn can_report() -> bool {
		Self::era_election_status().is_closed()
	}
}

/// Filter historical offences out and only allow those from the bonding period.
pub struct FilterHistoricalOffences<T, R> {
	_inner: sp_std::marker::PhantomData<(T, R)>,
}

impl<T, Reporter, Offender, R, O> ReportOffence<Reporter, Offender, O>
	for FilterHistoricalOffences<Module<T>, R> where
	T: Trait,
	R: ReportOffence<Reporter, Offender, O>,
	O: Offence<Offender>,
{
	fn report_offence(reporters: Vec<Reporter>, offence: O) -> Result<(), OffenceError> {
		<Module<T>>::ensure_storage_upgraded();

		// disallow any slashing from before the current bonding period.
		let offence_session = offence.session_index();
		let bonded_eras = BondedEras::get();

		if bonded_eras.first().filter(|(_, start)| offence_session >= *start).is_some() {
			R::report_offence(reporters, offence)
		} else {
			<Module<T>>::deposit_event(
				RawEvent::OldSlashingReportDiscarded(offence_session)
			);
			Ok(())
		}
	}
}

impl<T: Trait> sp_runtime::BoundToRuntimeAppPublic for Module<T> {
	type Public = T::KeyType;
}

impl<T: Trait> pallet_session::OneSessionHandler<T::AccountId> for Module<T> {
	type Key = T::KeyType;

	fn on_genesis_session<'a, I: 'a>(validators: I)
		where I: Iterator<Item=(&'a T::AccountId, T::KeyType)>
	{
		assert!(Self::keys().is_empty(), "Keys are already initialized!");
		<Keys<T>>::put(validators.map(|x| x.1).collect::<Vec<_>>());
	}

	fn on_new_session<'a, I: 'a>(_changed: bool, validators: I, _queued_validators: I)
		where I: Iterator<Item=(&'a T::AccountId, T::KeyType)>
	{
		// Update they keys
		<Keys<T>>::put(validators.map(|x| x.1).collect::<Vec<_>>());
	}

	fn on_before_session_ending() {}

	fn on_disabled(_i: usize) {}
}


/// Disallows any transactions that change the election result to be submitted after the election
/// window is open.
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct LockStakingStatus<T>(sp_std::marker::PhantomData<T>);

impl<T: Trait + Send + Sync> sp_std::fmt::Debug for LockStakingStatus<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "LockStakingStatus<{:?}>", self.0)
	}
	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl<T> LockStakingStatus<T> {
	/// Create new `LockStakingStatus`.
	pub fn new() -> Self {
		Self(sp_std::marker::PhantomData)
	}
}

impl<T> Default for LockStakingStatus<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T: Trait + Send + Sync> SignedExtension for LockStakingStatus<T> {
	const IDENTIFIER: &'static str = "LockStakingStatus";
	type AccountId = T::AccountId;
	type Call = <T as Trait>::Call;
	type AdditionalSigned = ();
	type DispatchInfo = frame_support::weights::DispatchInfo;
	type Pre = ();

	fn additional_signed(&self) -> Result<(), TransactionValidityError> { Ok(()) }

	fn validate(
		&self,
		_who: &Self::AccountId,
		call: &Self::Call,
		_info: Self::DispatchInfo,
		_len: usize,
	) -> TransactionValidity {
		if let Some(inner_call) = call.is_sub_type() {
			if let ElectionStatus::Open(_) = <Module<T>>::era_election_status() {
				match inner_call {
					Call::<T>::set_payee(..) |
					Call::<T>::set_controller(..) |
					Call::<T>::set_validator_count(..) |
					Call::<T>::force_no_eras(..) |
					Call::<T>::force_new_era(..) |
					Call::<T>::set_invulnerables(..) |
					Call::<T>::force_unstake(..) |
					Call::<T>::force_new_era_always(..) |
					Call::<T>::cancel_deferred_slash(..) |
					Call::<T>::set_history_depth(..) |
					Call::<T>::reap_stash(..) |
					Call::<T>::submit_election_solution(..) |
					Call::<T>::submit_election_solution_unsigned(..) => {
						// These calls are allowed. Nothing.
					}
					_ => {
						return Err(InvalidTransaction::Stale.into());
					}
				}
			}
		}

		Ok(Default::default())
	}
}

#[allow(deprecated)]
impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
	type Call = Call<T>;
	fn validate_unsigned(call: &Self::Call) -> TransactionValidity {
		if let Call::submit_election_solution_unsigned(
			winners,
			compact,
			score,
			validator_index,
			signature,
		) = call {
			use offchain_election::SignaturePayload;

			// discard early solution
			if Self::era_election_status().is_closed() {
				debug::native::debug!(
					target: "staking",
					"rejecting unsigned transaction because it is too early."
				);
				return InvalidTransaction::Future.into();
			}

			// discard weak solution
			if let Some(queued_score) = Self::queued_score() {
				if !is_score_better(queued_score, *score) {
					debug::native::debug!(
						target: "staking",
						"rejecting unsigned transaction because the claimed score is not good enough."
					);
					return InvalidTransaction::Custom(1u8).into();
				}
			}

			// check signature
			let payload: SignaturePayload = (
				winners,
				compact,
				score,
				validator_index,
			);

			let all_keys = Self::keys();
			let validator_key = all_keys.get(*validator_index as usize)
				// validator index is incorrect -- no key corresponds to it.
				.ok_or(TransactionValidityError::Unknown(UnknownTransaction::CannotLookup.into()))?;

			let signature_valid = payload.using_encoded(|encoded_payload| {
				validator_key.verify(&encoded_payload, &signature)
			});

			if !signature_valid {
				return InvalidTransaction::BadProof.into();
			}

			Ok(ValidTransaction {
				priority: score[0].saturated_into(),
				requires: vec![],
				provides: vec![(Self::current_era(), validator_key).encode()],
				longevity: TryInto::<u64>::try_into(T::ElectionLookahead::get()).unwrap_or(150_u64),
				propagate: true,
			})
		} else {
			InvalidTransaction::Call.into()
		}
	}
}
