// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The operational pallet for the Relay Chain, designed to manage and facilitate the migration of
//! subsystems such as Governance, Staking, Balances from the Relay Chain to the Asset Hub. This
//! pallet works alongside its counterpart, `pallet_ah_migrator`, which handles migration
//! processes on the Asset Hub side.
//!
//! This pallet is responsible for controlling the initiation, progression, and completion of the
//! migration process, including managing its various stages and transferring the necessary data.
//! The pallet directly accesses the storage of other pallets for read/write operations while
//! maintaining compatibility with their existing APIs.
//!
//! To simplify development and avoid the need to edit the original pallets, this pallet may
//! duplicate private items such as storage entries from the original pallets. This ensures that the
//! migration logic can be implemented without altering the original implementations.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_balances::AccountData;
use sp_runtime::AccountId32;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// Super config trait for all pallets that the migration depends on, providing convenient
	/// access to their items.
	#[pallet::config]
	pub trait Config:
		frame_system::Config<AccountData = AccountData<u128>, AccountId = AccountId32, Nonce = u32>
	{
		/// The overall runtime origin type.
		type RuntimeOrigin: Into<Result<pallet_xcm::Origin, <Self as Config>::RuntimeOrigin>>
			+ IsType<<Self as frame_system::Config>::RuntimeOrigin>
			+ From<frame_system::RawOrigin<Self::AccountId>>;
	}

	/// The block number when the migration started.
	#[pallet::storage]
	pub type MigrationStartBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, OptionQuery>;

	/// The block number when the migration ended.
	#[pallet::storage]
	pub type MigrationEndBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, OptionQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);
}

pub mod runtime_api {
	sp_api::decl_runtime_apis! {
		/// API to query information about the Asset Hub migration process.
		pub trait AssetHubMigrationApi<BlockNumber> where BlockNumber: sp_runtime::traits::BlockNumber {
			/// Returns the block number when the migration started.
			fn migration_start_block() -> BlockNumber;

			/// Returns the block number when the migration ended.
			fn migration_end_block() -> BlockNumber;
		}
	}
}
