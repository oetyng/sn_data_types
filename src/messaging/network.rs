// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::{
    AccountId, Address, Blob, BlobAddress, DebitAgreementProof, Error, Result, RewardCounter,
    Signature, SignedTransfer, TransferId, TransferValidated, XorName,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

// -------------- Network Cmds --------------

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NetworkCmd {
    ///
    Data(NetworkDataCmd),
    ///
    Rewards(NetworkRewardCmd),
    ///
    Transfers(NetworkTransferCmd),
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NetworkRewardCmd {
    /// Sent by the new section to the
    /// old section after node relocation.
    ClaimRewardCounter {
        /// The id of the node
        /// in the old section.
        old_node_id: XorName,
        /// The id of the node
        /// in the new section.
        new_node_id: XorName,
    },
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NetworkTransferCmd {
    ///
    PropagateTransfer(DebitAgreementProof),
    ///
    InitiateRewardPayout(SignedTransfer),
    ///
    FinaliseRewardPayout(DebitAgreementProof),
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NetworkDataCmd {
    ///
    DuplicateChunk {
        ///
        new_holder: XorName,
        ///
        address: BlobAddress,
        ///
        fetch_from_holders: BTreeSet<XorName>,
    },
}

// -------------- Network Events --------------

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NetworkEvent {
    /// Wrapper for a duplicate completion response, from a node to elders.
    DuplicationComplete {
        ///
        chunk: BlobAddress,
        /// The Elder's accumulated signature
        /// over the chunk address. This is sent back
        /// to them so that any uninformed Elder knows
        /// that this is all good.
        proof: Signature,
    },
    ///
    RewardPayoutValidated(TransferValidated),
    /// Raised by the old section to the
    /// old section after node relocation.
    RewardCounterClaimed {
        /// The id of the node
        /// in the new section.
        new_node_id: XorName,
        /// The account for which
        /// the rewards are accumulated.
        account_id: AccountId,
        /// Accumulated work & reward
        counter: RewardCounter,
    },
}

///
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NetworkQuery {
    /// Elder to Adult Get.
    GetChunk {
        /// The holder id.
        holder: XorName,
        /// The chunk address.
        address: BlobAddress,
    },
    /// Adult to Adult Get
    GetChunks {
        /// The holder id.
        holder: XorName,
        /// The chunk addresses.
        addresses: BTreeSet<BlobAddress>,
    },
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NetworkQueryResponse {
    /// Elder to Adult Get.
    GetChunk(Result<Blob>),
    /// Adult to Adult Get
    GetChunks(Result<Vec<Blob>>),
}

///
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NetworkCmdError {
    ///
    Data(NetworkDataError),
    ///
    Rewards(NetworkRewardError),
    ///
    Transfers(NetworkTransferError),
}

///
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NetworkDataError {
    ///
    ChunkDuplication {
        ///
        address: BlobAddress,
        ///
        error: Error,
    },
}

///
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NetworkTransferError {
    /// The error of propagation of TransferRegistered event.
    TransferPropagation(Error),
}

///
#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum NetworkRewardError {
    ///
    RewardClaiming {
        ///
        account_id: AccountId,
        ///
        error: Error,
    },
    ///
    RewardPayoutInitiation {
        ///
        id: TransferId,
        ///
        account: AccountId,
        ///
        error: Error,
    },
    ///
    RewardPayoutFinalisation {
        ///
        id: TransferId,
        ///
        account: AccountId,
        ///
        error: Error,
    },
}

impl NetworkCmd {
    /// Returns the address of the destination for `request`.
    pub fn dst_address(&self) -> Address {
        use Address::*;
        use NetworkCmd::*;
        use NetworkDataCmd::*;
        use NetworkRewardCmd::*;
        use NetworkTransferCmd::*;
        match self {
            Data(DuplicateChunk { new_holder, .. }) => Node(*new_holder),
            Rewards(ClaimRewardCounter { old_node_id, .. }) => Section(*old_node_id),
            Transfers(cmd) => match cmd {
                InitiateRewardPayout(signed_transfer) => Section(signed_transfer.from().into()),
                FinaliseRewardPayout(debit_agreement) => Section(debit_agreement.from().into()),
                PropagateTransfer(debit_agreement) => Section(debit_agreement.to().into()),
            },
        }
    }
}

impl NetworkEvent {
    /// Returns the address of the destination for `request`.
    pub fn dst_address(&self) -> Address {
        use Address::*;
        use NetworkEvent::*;
        match self {
            DuplicationComplete { chunk, .. } => Section(*chunk.name()),
            RewardCounterClaimed { new_node_id, .. } => Section(*new_node_id),
            RewardPayoutValidated(event) => Section(event.from().into()),
        }
    }
}

impl NetworkQuery {
    /// Returns the address of the destination for the query.
    pub fn dst_address(&self) -> Address {
        use Address::*;
        use NetworkQuery::*;
        match self {
            GetChunk { holder, .. } | GetChunks { holder, .. } => Node(*holder),
        }
    }
}