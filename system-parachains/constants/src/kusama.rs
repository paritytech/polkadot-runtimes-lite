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

/// Constants relating to KSM.
pub mod currency {
	use polkadot_core_primitives::Balance;

	/// One "KSM" that a UI would show a user.
	pub const UNITS: Balance = 1_000_000_000_000;
	pub const QUID: Balance = UNITS / 30;
	pub const CENTS: Balance = QUID / 100;
	pub const MILLICENTS: Balance = CENTS / 1_000;
}

/// Constants related to Kusama fee payment.
pub mod fee {
	use polkadot_core_primitives::Balance;

	/// Cost of every transaction byte at Kusama system
	/// parachains.
	///
	/// It is the Relay Chain (Kusama) `TransactionByteFee`
	/// / 10.
	pub const TRANSACTION_BYTE_FEE: Balance = super::currency::MILLICENTS;
}
