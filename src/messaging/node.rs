// // Copyright 2019 MaidSafe.net limited.
// //
// // This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// // https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// // https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// // modified, or distributed except according to those terms. Please review the Licences for the
// // specific language governing permissions and limitations relating to use of the SAFE Network
// // Software.

// pub use super::transfer::{TransferCmd, TransferQuery};
// use super::{query::DataQuery, system::SystemOp, cmd::DataCmd, Cmd, AuthorisationKind, QueryResponse};
// use crate::{Error, XorName};
// use serde::{Deserialize, Serialize};
// use std::{borrow::Cow, fmt};

// /// Node internal requests
// #[allow(clippy::large_enum_variant)]
// #[derive(Hash, Eq, PartialEq, PartialOrd, Clone, Serialize, Deserialize)]
// pub enum NodeRequest {
//     /// Read
//     Read(Read),
//     /// Write
//     Write(Write),
//     /// System requests
//     /// originating at Client (Owner)
//     System(SystemOp),
// }

// impl NodeRequest {
//     /// Get the `Type` of this `Request`.
//     pub fn get_type(&self) -> Type {
//         use NodeRequest::*;
//         match self {
//             Read(req) => req.get_type(),
//             Write(req) => req.get_type(),
//             System(req) => req.get_type(),
//         }
//     }

//     /// Creates a Response containing an error, with the Response variant corresponding to the
//     /// Request variant.
//     pub fn error_response(&self, error: Error) -> Response {
//         use NodeRequest::*;
//         match self {
//             Read(req) => req.error_response(error),
//             Write(req) => req.error_response(error),
//             System(req) => req.error_response(error),
//         }
//     }

//     /// Returns the type of authorisation needed for the request.
//     pub fn authorisation_kind(&self) -> AuthorisationKind {
//         use NodeRequest::*;
//         match self {
//             Read(req) => req.authorisation_kind(),
//             Write(req) => req.authorisation_kind(),
//             System(req) => req.authorisation_kind(),
//         }
//     }

//     /// Returns the address of the destination for `request`.
//     pub fn dst_address(&self) -> XorName {
//         use NodeRequest::*;
//         match self {
//             Read(req) => req.dst_address(),
//             Write(req) => req.dst_address(),
//             System(req) => req.dst_address(),
//         }
//     }
// }

// impl fmt::Debug for NodeRequest {
//     fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         use NodeRequest::*;
//         match self {
//             Read(req) => write!(formatter, "{:?}", req),
//             Write(req) => write!(formatter, "{:?}", req),
//             System(req) => write!(formatter, "{:?}", req),
//         }
//     }
// }