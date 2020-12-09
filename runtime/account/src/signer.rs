// Copyright 2019-2020 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! The Ethereum Signature implementation.
//!
//! It includes the Verify and IdentifyAccount traits for the AccountId20

use sp_core::{H160, H256, ecdsa, ed25519, sr25519, RuntimeDebug};
use codec::{Decode, Encode};
use sha3::{Digest, Keccak256};

#[cfg(feature = "std")]
pub use serde::{Serialize, Deserialize, de::DeserializeOwned};

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, sp_core::RuntimeDebug)]
pub struct EthereumSignature(ecdsa::Signature);

impl From<ecdsa::Signature> for EthereumSignature {
	fn from(x: ecdsa::Signature) -> Self {
		EthereumSignature(x)
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, RuntimeDebug)]
pub enum MultiSignature {
	Ed25519(ed25519::Signature),
	Sr25519(sr25519::Signature),
	Ecdsa(EthereumSignature),
}

impl From<EthereumSignature> for MultiSignature {
	fn from(x: EthereumSignature) -> Self {
		MultiSignature::Ecdsa(x)
	}
}

impl sp_runtime::traits::Verify for MultiSignature {
	type Signer = EthereumSigner;
	fn verify<L: sp_runtime::traits::Lazy<[u8]>>(
		&self,
		mut msg: L,
		signer: &H160
	) -> bool {
		match (self, signer) {
			(MultiSignature::Ecdsa(ref sig), who) => {
				let mut m = [0u8; 32];
				m.copy_from_slice(Keccak256::digest(msg.get()).as_slice());
				match sp_io::crypto::secp256k1_ecdsa_recover(sig.0.as_ref(), &m) {
					Ok(pubkey) => {
						H160::from(H256::from_slice(Keccak256::digest(&pubkey).as_slice())) ==
						*who
					},
					Err(sp_io::EcdsaVerifyError::BadRS) => {
						log::error!(target: "evm", "Error recovering: Incorrect value of R or S");
						false
					},
					Err(sp_io::EcdsaVerifyError::BadV) => {
						log::error!(target: "evm", "Error recovering: Incorrect value of V");
						false
					},
					Err(sp_io::EcdsaVerifyError::BadSignature) => {
						log::error!(target: "evm", "Error recovering: Invalid signature");
						false
					}
				}
			},
			_ => false
		}
	}
}


/// Public key for any known crypto algorithm.
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Encode, Decode, sp_core::RuntimeDebug)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct EthereumSigner ([u8; 20]);

impl sp_runtime::traits::IdentifyAccount for EthereumSigner {
	type AccountId = H160;
	fn into_account(self) -> H160 {
		self.0.into()
	}
}

impl From<[u8; 20]> for EthereumSigner {
	fn from(x: [u8; 20]) -> Self {
		EthereumSigner(x)
	}
}


impl From<ecdsa::Public> for EthereumSigner {
	fn from(x: ecdsa::Public) -> Self {
		let mut m = [0u8; 20];
		m.copy_from_slice(&x.as_ref()[13..33]);
		EthereumSigner(m)
	}
}

#[cfg(feature = "std")]
impl std::fmt::Display for EthereumSigner {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(fmt, "ethereum signature: {:?}", H160::from_slice(&self.0))
	}
}
