// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::keys::{BlsKeypairShare, SignatureShare};
use crate::{utils, Error, PublicKey, Signature};
use ed25519_dalek::{Keypair as Ed25519Keypair, PublicKey as Ed25519PublicKey};
use hex_fmt::HexFmt;
use multibase::Decodable;
use rand::{CryptoRng, Rng};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use signature::Signer;
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter},
    hash::{Hash, Hasher},
};
use threshold_crypto::{
    serde_impl::SerdeSecret, PublicKeySet, PublicKeyShare as BlsPublicKeyShare,
    SecretKeyShare as BlsSecretKeyShare,
};
use xor_name::XorName;

/// A struct holding an Ed25519 keypair, an optional BLS keypair share, and the corresponding public
/// ID for a network Node.
#[derive(Serialize, Deserialize)]
pub struct FullId {
    ed25519: Ed25519Keypair,
    bls: Option<BlsKeypairShare>,
    public_id: PublicId,
}

impl FullId {
    /// Constructs a `FullId` with a random Ed25519 keypair and no BLS keys.
    pub fn new<T: CryptoRng + Rng>(rng: &mut T) -> Self {
        let ed25519 = Ed25519Keypair::generate(rng);
        let name = PublicKey::Ed25519(ed25519.public).into();
        let public_id = PublicId {
            name,
            ed25519: ed25519.public,
            bls: None,
        };
        Self {
            ed25519,
            bls: None,
            public_id,
        }
    }

    /// Constructs a `FullId` whose name is in the interval [start, end] (both endpoints inclusive).
    pub fn within_range<T: CryptoRng + Rng>(start: &XorName, end: &XorName, rng: &mut T) -> Self {
        let mut ed25519 = Ed25519Keypair::generate(rng);
        loop {
            let name = PublicKey::Ed25519(ed25519.public).into();
            if name >= *start && name <= *end {
                let public_id = PublicId {
                    name,
                    ed25519: ed25519.public,
                    bls: None,
                };
                return Self {
                    ed25519,
                    bls: None,
                    public_id,
                };
            }
            ed25519 = Ed25519Keypair::generate(rng);
        }
    }

    /// Returns the public ID.
    pub fn public_id(&self) -> &PublicId {
        &self.public_id
    }

    /// Creates a detached Ed25519 signature of `data`.
    pub fn sign_using_ed25519<T: AsRef<[u8]>>(&self, data: T) -> Signature {
        Signature::Ed25519(self.ed25519.sign(data.as_ref()))
    }

    /// Creates a detached BLS signature share of `data` if the `self` holds a BLS keypair share.
    pub fn sign_using_bls<T: AsRef<[u8]>>(&self, data: T) -> Option<Signature> {
        self.bls.as_ref().map(|keys| {
            Signature::BlsShare(SignatureShare {
                index: keys.index,
                share: keys.secret.inner().sign(data),
            })
        })
    }

    /// Sets the `FullId`'s BLS keypair share using the provided BLS secret key share.
    pub fn set_bls_keys(&mut self, secret_share: BlsSecretKeyShare, public_set: PublicKeySet) {
        let public = secret_share.public_key_share();
        let secret = SerdeSecret(secret_share);
        self.public_id.bls = Some(public);
        self.bls = Some(BlsKeypairShare {
            index: 0,
            secret,
            public,
            public_key_set: public_set,
        });
    }

    /// Clears the `FullId`'s BLS keypair share, i.e. sets it to `None`.
    pub fn clear_bls_keys(&mut self) {
        self.public_id.bls = None;
        self.bls = None;
    }
}

/// A struct representing the public identity of a network Node.
///
/// It includes the Ed25519 public key and the optional BLS public key.  This struct also provides
/// the Node's network address, i.e. `name()` derived from the Ed25519 public key.
#[derive(Clone, Eq, PartialEq)]
pub struct PublicId {
    name: XorName,
    ed25519: Ed25519PublicKey,
    bls: Option<BlsPublicKeyShare>,
}

impl PublicId {
    /// Returns the Node's network address.
    pub fn name(&self) -> &XorName {
        &self.name
    }

    /// Returns the Node's Ed25519 public key.
    pub fn ed25519_public_key(&self) -> &Ed25519PublicKey {
        &self.ed25519
    }

    /// Returns the Node's BLS public key share.
    pub fn bls_public_key(&self) -> &Option<BlsPublicKeyShare> {
        &self.bls
    }

    /// Returns the PublicId serialised and encoded in z-base-32.
    pub fn encode_to_zbase32(&self) -> String {
        utils::encode(&self)
    }

    /// Creates from z-base-32 encoded string.
    pub fn decode_from_zbase32<T: Decodable>(encoded: T) -> Result<Self, Error> {
        utils::decode(encoded)
    }
}

impl Serialize for PublicId {
    fn serialize<S: Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
        (&self.ed25519, &self.bls).serialize(serialiser)
    }
}

impl<'de> Deserialize<'de> for PublicId {
    fn deserialize<D: Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        let (ed25519, bls): (Ed25519PublicKey, Option<BlsPublicKeyShare>) =
            Deserialize::deserialize(deserialiser)?;
        let name = PublicKey::Ed25519(ed25519).into();
        Ok(PublicId { name, ed25519, bls })
    }
}

impl Ord for PublicId {
    fn cmp(&self, other: &PublicId) -> Ordering {
        utils::serialise(&self).cmp(&utils::serialise(other))
    }
}

impl PartialOrd for PublicId {
    fn partial_cmp(&self, other: &PublicId) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for PublicId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        utils::serialise(&self).hash(state)
    }
}

impl Debug for PublicId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "Node({:<8})", HexFmt(&self.ed25519.to_bytes()))
    }
}

impl Display for PublicId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Debug::fmt(self, formatter)
    }
}

/// This is used at a network node for holding the
/// obligatory Ed25519 keypair needed as Adult, and
/// then a BLS keypair share when being promoted to Elder.
/// (Also the corresponding public keys).
/// The Ed25519 is kept as Elder, in case it is demoted.
#[derive(Serialize, Deserialize)]
pub struct NodeKeypairs {
    ed25519: Ed25519Keypair,
    bls: Option<BlsKeypairShare>,
    public_id: PublicId,
}

impl NodeKeypairs {
    /// Constructs a `NodeKeypairs` with a random Ed25519 keypair and no BLS keys.
    pub fn new<T: CryptoRng + Rng>(rng: &mut T) -> Self {
        let ed25519 = Ed25519Keypair::generate(rng);
        let name = PublicKey::Ed25519(ed25519.public).into();
        let public_id = PublicId {
            name,
            ed25519: ed25519.public,
            bls: None,
        };
        Self {
            ed25519,
            bls: None,
            public_id,
        }
    }

    /// Constructs a `NodeKeypairs` whose name is in the interval [start, end] (both endpoints inclusive).
    pub fn within_range<T: CryptoRng + Rng>(start: &XorName, end: &XorName, rng: &mut T) -> Self {
        let mut ed25519 = Ed25519Keypair::generate(rng);
        loop {
            let name = PublicKey::Ed25519(ed25519.public).into();
            if name >= *start && name <= *end {
                let public_id = PublicId {
                    name,
                    ed25519: ed25519.public,
                    bls: None,
                };
                return Self {
                    ed25519,
                    bls: None,
                    public_id,
                };
            }
            ed25519 = Ed25519Keypair::generate(rng);
        }
    }

    /// Returns the BLS if any, else the Ed25519.
    pub fn public_key(&self) -> PublicKey {
        if let Some(key) = self.public_id.bls {
            PublicKey::BlsShare(key)
        } else {
            PublicKey::Ed25519(self.public_id.ed25519)
        }
    }

    /// Returns the public keys.
    pub fn public_id(&self) -> &PublicId {
        &self.public_id
    }

    /// Returns the BLS public key set if any.
    pub fn public_key_set(&self) -> Option<&PublicKeySet> {
        self.bls.as_ref().map(|s| &s.public_key_set)
    }

    /// Signs with the BLS if any, else the Ed25519.
    pub fn sign(&self, data: &[u8]) -> Signature {
        if let Some(sig) = self.sign_using_bls(data) {
            sig
        } else {
            self.sign_using_ed25519(data)
        }
    }

    /// Creates a detached Ed25519 signature of `data`.
    pub fn sign_using_ed25519<T: AsRef<[u8]>>(&self, data: T) -> Signature {
        Signature::Ed25519(self.ed25519.sign(data.as_ref()))
    }

    /// Creates a detached BLS signature share of `data` if the `self` holds a BLS keypair share.
    pub fn sign_using_bls<T: AsRef<[u8]>>(&self, data: T) -> Option<Signature> {
        self.bls.as_ref().map(|keys| {
            Signature::BlsShare(SignatureShare {
                index: keys.index,
                share: keys.secret.inner().sign(data),
            })
        })
    }

    /// Sets the `NodeKeypairs`'s BLS keypair share using the provided BLS secret key share.
    pub fn set_bls_keys(
        &mut self,
        index: usize,
        secret_share: BlsSecretKeyShare,
        public_set: PublicKeySet,
    ) {
        let public = secret_share.public_key_share();
        let secret = SerdeSecret(secret_share);
        self.public_id.bls = Some(public);
        self.bls = Some(BlsKeypairShare {
            index,
            secret,
            public,
            public_key_set: public_set,
        });
    }

    /// Clears the `NodeKeypairs`'s BLS keypair share, i.e. sets it to `None`.
    pub fn clear_bls_keys(&mut self) {
        self.public_id.bls = None;
        self.bls = None;
    }
}
