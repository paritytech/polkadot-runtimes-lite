// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

pub use bp_bridge_hub_cumulus::*;
use sp_runtime::{FixedPointNumber, FixedU128, Saturating};

/// Identifier of BridgeHubPolkadot in the Polkadot relay chain.
pub const BRIDGE_HUB_POLKADOT_PARACHAIN_ID: u32 = 1002;

/// Pallet index of `BridgeKusamaMessages: pallet_bridge_messages::<Instance1>`.
pub const WITH_BRIDGE_POLKADOT_TO_KUSAMA_MESSAGES_PALLET_INDEX: u8 = 53;

frame_support::parameter_types! {
	/// The XCM fee that is paid for executing XCM program
	/// (with `ExportMessage` instruction) at the Polkadot
	/// BridgeHub.
	pub const BridgeHubPolkadotBaseXcmFeeInDots: Balance =
		5_638_281;

	/// Transaction fee that is paid at the Polkadot BridgeHub
	/// for delivering single outbound message confirmation.
	pub const BridgeHubPolkadotBaseConfirmationFeeInDots: Balance =
		27_027_016;
}

/// Compute the total estimated fee that needs to be paid in
/// DOTs by the sender when sending message from Polkadot
/// Bridge Hub to Kusama Bridge Hub.
pub fn estimate_polkadot_to_kusama_message_fee(
	bridge_hub_kusama_base_delivery_fee_in_uksms: Balance,
) -> Balance {
	BridgeHubPolkadotBaseXcmFeeInDots::get()
		.saturating_add(convert_from_uksm_to_udot(bridge_hub_kusama_base_delivery_fee_in_uksms))
		.saturating_add(BridgeHubPolkadotBaseConfirmationFeeInDots::get())
}

/// Compute the per-byte fee that needs to be paid in DOTs by
/// the sender when sending message from Polkadot Bridge Hub
/// to Kusama Bridge Hub.
pub fn estimate_polkadot_to_kusama_byte_fee() -> Balance {
	convert_from_uksm_to_udot(system_parachains_constants::kusama::fee::TRANSACTION_BYTE_FEE)
}

/// Convert from uKSMs to uDOTs.
fn convert_from_uksm_to_udot(price_in_uksm: Balance) -> Balance {
	// assuming exchange rate is 5 DOTs for 1 KSM
	let dot_to_ksm_economic_rate = FixedU128::from_rational(5, 1);
	// tokens have different nominals and we need to take that
	// into account
	let nominal_ratio = FixedU128::from_rational(
		polkadot_runtime_constants::currency::UNITS,
		kusama_runtime_constants::currency::UNITS,
	);

	dot_to_ksm_economic_rate
		.saturating_mul(nominal_ratio)
		.saturating_mul(FixedU128::saturating_from_integer(price_in_uksm))
		.into_inner() /
		FixedU128::DIV
}

pub mod snowbridge {
	use frame_support::parameter_types;
	use xcm::latest::{Location, NetworkId};

	parameter_types! {
		/// The pallet index of the Ethereum inbound queue
		/// pallet in the BridgeHub runtime.
		pub const InboundQueuePalletInstance: u8 = 80;
		/// The pallet index of the Ethereum inbound queue v2
		/// pallet in the BridgeHub runtime.
		pub const InboundQueueV2PalletInstance: u8 = 91;
		/// Network and location for the Ethereum chain. On
		/// Polkadot, the Ethereum chain bridged to is the
		/// Ethereum Main network, with chain ID 1.
		pub EthereumNetwork: NetworkId =
			NetworkId::Ethereum { chain_id: 1 };
		pub EthereumLocation: Location =
			Location::new(2, EthereumNetwork::get());
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn convert_from_uksm_to_udot_works() {
		let price_in_uksm = 77 * kusama_runtime_constants::currency::UNITS;
		let same_price_in_udot = convert_from_uksm_to_udot(price_in_uksm);

		let price_in_ksm =
			FixedU128::from_rational(price_in_uksm, kusama_runtime_constants::currency::UNITS);
		let price_in_dot = FixedU128::from_rational(
			same_price_in_udot,
			polkadot_runtime_constants::currency::UNITS,
		);
		assert_eq!(price_in_dot / FixedU128::saturating_from_integer(5), price_in_ksm);
	}
}
