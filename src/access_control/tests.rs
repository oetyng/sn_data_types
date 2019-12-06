// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

#[cfg(test)]
mod tests {
    use crate::access_control::{
        CmdType, HardErasureCmd, MapCmd, MapQuery, MapWrite, ModifyableMapPermissions,
        ModifyableSequencePermissions, PrivatePermissionSet, PrivatePermissions,
        PublicPermissionSet, PublicPermissions, QueryType, Request, SequenceCmd, SequenceQuery,
        SequenceWrite,
    };
    use crate::map::*;
    use crate::sequence::*;
    use crate::shared_data::{Index, Owner, User};
    use crate::{Error, PublicKey, XorName};
    use std::collections::BTreeMap;
    use threshold_crypto::SecretKey;
    use unwrap::unwrap;

    fn gen_public_key() -> PublicKey {
        PublicKey::Bls(SecretKey::random().public_key())
    }

    // ------------------------------------------------------------------------------------------
    // -----------------------------------  Sequence  -------------------------------------------
    // ------------------------------------------------------------------------------------------

    #[test]
    fn set_sequence_permissions() {
        let mut data = PrivateSentriedSequence::new(XorName([1; 32]), 10000);

        // Set the first permissions with correct ExpectedIndices - should pass.
        let res = data.set_permissions(
            PrivatePermissions {
                permissions: BTreeMap::new(),
                expected_data_index: 0,
                expected_owners_index: 0,
            },
            0,
        );

        match res {
            Ok(()) => (),
            Err(x) => panic!("Unexpected error: {:?}", x),
        }

        // Verify that the permissions are part of the history.
        assert_eq!(
            unwrap!(data.permission_history_range(Index::FromStart(0), Index::FromEnd(0),)).len(),
            1
        );

        // Set permissions with incorrect ExpectedIndices - should fail.
        let res = data.set_permissions(
            PrivatePermissions {
                permissions: BTreeMap::new(),
                expected_data_index: 64,
                expected_owners_index: 0,
            },
            1,
        );

        match res {
            Err(_) => (),
            Ok(()) => panic!("Unexpected Ok(()) result"),
        }

        // Verify that the history of permissions remains unchanged.
        assert_eq!(
            unwrap!(data.permission_history_range(Index::FromStart(0), Index::FromEnd(0),)).len(),
            1
        );
    }

    #[test]
    fn set_sequence_owners() {
        let owner_pk = gen_public_key();

        let mut data = PrivateSentriedSequence::new(XorName([1; 32]), 10000);

        // Set the first owner with correct ExpectedIndices - should pass.
        let res = data.set_owner(
            Owner {
                public_key: owner_pk,
                expected_data_index: 0,
                expected_permissions_index: 0,
            },
            0,
        );

        match res {
            Ok(()) => (),
            Err(x) => panic!("Unexpected error: {:?}", x),
        }

        // Verify that the owner is part of the history.
        assert_eq!(
            unwrap!(data.owner_history_range(Index::FromStart(0), Index::FromEnd(0),)).len(),
            1
        );

        // Set owner with incorrect ExpectedIndices - should fail.
        let res = data.set_owner(
            Owner {
                public_key: owner_pk,
                expected_data_index: 64,
                expected_permissions_index: 0,
            },
            1,
        );

        match res {
            Err(_) => (),
            Ok(()) => panic!("Unexpected Ok(()) result"),
        }

        // Verify that the history of owners remains unchanged.
        assert_eq!(
            unwrap!(data.owner_history_range(Index::FromStart(0), Index::FromEnd(0),)).len(),
            1
        );
    }

    #[test]
    fn gets_sequence_shell() {
        let owner_pk = gen_public_key();
        let owner_pk1 = gen_public_key();

        let mut data = PrivateSentriedSequence::new(XorName([1; 32]), 10000);

        let _ = data.set_owner(
            Owner {
                public_key: owner_pk,
                expected_data_index: 0,
                expected_permissions_index: 0,
            },
            0,
        );

        let _ = data.set_owner(
            Owner {
                public_key: owner_pk1,
                expected_data_index: 0,
                expected_permissions_index: 0,
            },
            1,
        );

        assert_eq!(
            data.expected_owners_index(),
            unwrap!(data.shell(0)).expected_owners_index()
        );
    }

    #[test]
    fn can_retrieve_sequence_permissions() {
        let public_key = gen_public_key();
        let invalid_public_key = gen_public_key();

        let mut pub_permissions = PublicPermissions {
            permissions: BTreeMap::new(),
            expected_data_index: 0,
            expected_owners_index: 0,
        };
        let _ = pub_permissions.permissions.insert(
            User::Specific(public_key),
            PublicPermissionSet::new(BTreeMap::new()),
        );

        let mut private_permissions = PrivatePermissions {
            permissions: BTreeMap::new(),
            expected_data_index: 0,
            expected_owners_index: 0,
        };
        let _ = private_permissions
            .permissions
            .insert(public_key, PrivatePermissionSet::new(BTreeMap::new()));

        // pub, unseq
        let mut data = PublicSequence::new(rand::random(), 20);
        unwrap!(data.set_permissions(pub_permissions.clone(), 0));
        let data = SequenceData::from(data);

        assert_eq!(data.public_permissions_at(0), Ok(&pub_permissions));
        assert_eq!(data.private_permissions_at(0), Err(Error::NoSuchData));

        assert_eq!(
            data.public_user_permissions_at(User::Specific(public_key), 0),
            Ok(PublicPermissionSet::new(BTreeMap::new()))
        );
        assert_eq!(
            data.private_user_permissions_at(public_key, 0),
            Err(Error::NoSuchData)
        );
        assert_eq!(
            data.public_user_permissions_at(User::Specific(invalid_public_key), 0),
            Err(Error::NoSuchEntry)
        );

        // pub, seq
        let mut data = PublicSentriedSequence::new(rand::random(), 20);
        unwrap!(data.set_permissions(pub_permissions.clone(), 0));
        let data = SequenceData::from(data);

        assert_eq!(data.public_permissions_at(0), Ok(&pub_permissions));
        assert_eq!(data.private_permissions_at(0), Err(Error::NoSuchData));

        assert_eq!(
            data.public_user_permissions_at(User::Specific(public_key), 0),
            Ok(PublicPermissionSet::new(BTreeMap::new()))
        );
        assert_eq!(
            data.private_user_permissions_at(public_key, 0),
            Err(Error::NoSuchData)
        );
        assert_eq!(
            data.public_user_permissions_at(User::Specific(invalid_public_key), 0),
            Err(Error::NoSuchEntry)
        );

        // Private, unseq
        let mut data = PrivateSequence::new(rand::random(), 20);
        unwrap!(data.set_permissions(private_permissions.clone(), 0));
        let data = SequenceData::from(data);

        assert_eq!(data.private_permissions_at(0), Ok(&private_permissions));
        assert_eq!(data.public_permissions_at(0), Err(Error::NoSuchData));

        assert_eq!(
            data.private_user_permissions_at(public_key, 0),
            Ok(PrivatePermissionSet::new(BTreeMap::new()))
        );
        assert_eq!(
            data.public_user_permissions_at(User::Specific(public_key), 0),
            Err(Error::NoSuchData)
        );
        assert_eq!(
            data.private_user_permissions_at(invalid_public_key, 0),
            Err(Error::NoSuchEntry)
        );

        // Private, seq
        let mut data = PrivateSentriedSequence::new(rand::random(), 20);
        unwrap!(data.set_permissions(private_permissions.clone(), 0));
        let data = SequenceData::from(data);

        assert_eq!(data.private_permissions_at(0), Ok(&private_permissions));
        assert_eq!(data.public_permissions_at(0), Err(Error::NoSuchData));

        assert_eq!(
            data.private_user_permissions_at(public_key, 0),
            Ok(PrivatePermissionSet::new(BTreeMap::new()))
        );
        assert_eq!(
            data.public_user_permissions_at(User::Specific(public_key), 0),
            Err(Error::NoSuchData)
        );
        assert_eq!(
            data.private_user_permissions_at(invalid_public_key, 0),
            Err(Error::NoSuchEntry)
        );
    }

    #[test]
    fn validates_public_sequence_permissions() {
        let public_key_0 = gen_public_key();
        let public_key_1 = gen_public_key();
        let public_key_2 = gen_public_key();
        let mut sequence = PublicSentriedSequence::new(XorName([1; 32]), 100);

        // no owner
        let data = SequenceData::from(sequence.clone());
        assert_eq!(data.is_permitted(get_append_cmd(), public_key_0), false);
        // data is Public - read always allowed
        assert_sequence_read_permitted(&data, public_key_0, true);

        // no permissions
        unwrap!(sequence.set_owner(
            Owner {
                public_key: public_key_0,
                expected_data_index: 0,
                expected_permissions_index: 0,
            },
            0,
        ));
        let data = SequenceData::from(sequence.clone());

        assert_eq!(data.is_permitted(get_append_cmd(), public_key_0), true);
        assert_eq!(data.is_permitted(get_append_cmd(), public_key_1), false);
        // data is Public - read always allowed
        assert_sequence_read_permitted(&data, public_key_0, true);
        assert_sequence_read_permitted(&data, public_key_1, true);

        // with permissions
        let mut permissions = PublicPermissions {
            permissions: BTreeMap::new(),
            expected_data_index: 0,
            expected_owners_index: 1,
        };
        let mut set = BTreeMap::new();
        let _ = set.insert(get_append_cmd(), true);
        let _ = permissions
            .permissions
            .insert(User::Anyone, PublicPermissionSet::new(set));
        let mut set = BTreeMap::new();
        for cmd in get_full_modify_sequence_permissions() {
            let _ = set.insert(cmd, true);
        }
        let _ = permissions
            .permissions
            .insert(User::Specific(public_key_1), PublicPermissionSet::new(set));
        unwrap!(sequence.set_permissions(permissions, 0));
        let data = SequenceData::from(sequence);

        // existing key fallback
        assert_eq!(data.is_permitted(get_append_cmd(), public_key_1), true);
        // existing key override
        assert_modify_sequence_permissions_permitted(&data, public_key_1, true);
        // non-existing keys are handled by `Anyone`
        assert_eq!(data.is_permitted(get_append_cmd(), public_key_2), true);
        assert_modify_sequence_permissions_permitted(&data, public_key_2, false);
        // data is Public - read always allowed
        assert_sequence_read_permitted(&data, public_key_0, true);
        assert_sequence_read_permitted(&data, public_key_1, true);
        assert_sequence_read_permitted(&data, public_key_2, true);
    }

    #[test]
    fn validates_private_sequence_permissions() {
        let public_key_0 = gen_public_key();
        let public_key_1 = gen_public_key();
        let public_key_2 = gen_public_key();
        let mut sequence = PrivateSentriedSequence::new(XorName([1; 32]), 100);

        // no owner
        let data = SequenceData::from(sequence.clone());
        assert_sequence_read_permitted(&data, public_key_0, false);

        // no permissions
        unwrap!(sequence.set_owner(
            Owner {
                public_key: public_key_0,
                expected_data_index: 0,
                expected_permissions_index: 0,
            },
            0,
        ));
        let data = SequenceData::from(sequence.clone());

        assert_sequence_read_permitted(&data, public_key_0, true);
        assert_sequence_read_permitted(&data, public_key_1, false);

        // with permissions
        let mut permissions = PrivatePermissions {
            permissions: BTreeMap::new(),
            expected_data_index: 0,
            expected_owners_index: 1,
        };
        let mut set = BTreeMap::new();
        let _ = set.insert(get_append_cmd(), true);
        for query in get_full_sequence_read_permissions() {
            let _ = set.insert(query, true);
        }
        for cmd in get_full_modify_sequence_permissions() {
            let _ = set.insert(cmd, false);
        }
        let _ = permissions
            .permissions
            .insert(public_key_1, PrivatePermissionSet::new(set));
        unwrap!(sequence.set_permissions(permissions, 0));
        let data = SequenceData::from(sequence);

        // existing key
        assert_sequence_read_permitted(&data, public_key_1, true);
        assert_eq!(data.is_permitted(get_append_cmd(), public_key_1), true);
        assert_modify_sequence_permissions_permitted(&data, public_key_1, false);

        // non-existing key
        assert_sequence_read_permitted(&data, public_key_2, false);
        assert_eq!(data.is_permitted(get_append_cmd(), public_key_2), false);
        assert_modify_sequence_permissions_permitted(&data, public_key_2, false);
    }

    fn get_append_cmd() -> Request {
        Request::Cmd(CmdType::Sequence(SequenceCmd::Append))
    }

    fn get_sequence_read_query(query: SequenceQuery) -> Request {
        Request::Query(QueryType::Sequence(query))
    }

    fn get_full_sequence_read_permissions() -> Vec<Request> {
        vec![
            Request::Query(QueryType::Sequence(SequenceQuery::ReadData)),
            Request::Query(QueryType::Sequence(SequenceQuery::ReadOwners)),
            Request::Query(QueryType::Sequence(SequenceQuery::ReadPermissions)),
        ]
    }

    fn get_modify_sequence_permissions(permission: ModifyableSequencePermissions) -> Request {
        Request::Cmd(CmdType::Sequence(SequenceCmd::ModifyPermissions(
            permission,
        )))
    }

    fn get_full_modify_sequence_permissions() -> Vec<Request> {
        vec![
            Request::Cmd(CmdType::Sequence(SequenceCmd::ModifyPermissions(
                ModifyableSequencePermissions::ReadData,
            ))),
            Request::Cmd(CmdType::Sequence(SequenceCmd::ModifyPermissions(
                ModifyableSequencePermissions::ReadOwners,
            ))),
            Request::Cmd(CmdType::Sequence(SequenceCmd::ModifyPermissions(
                ModifyableSequencePermissions::ReadPermissions,
            ))),
            Request::Cmd(CmdType::Sequence(SequenceCmd::ModifyPermissions(
                ModifyableSequencePermissions::Write(SequenceWrite::Append),
            ))),
            Request::Cmd(CmdType::Sequence(SequenceCmd::ModifyPermissions(
                ModifyableSequencePermissions::Write(SequenceWrite::ModifyPermissions),
            ))),
            Request::Cmd(CmdType::Sequence(SequenceCmd::ModifyPermissions(
                ModifyableSequencePermissions::Write(SequenceWrite::HardErasure(
                    HardErasureCmd::HardDelete,
                )),
            ))),
            Request::Cmd(CmdType::Sequence(SequenceCmd::ModifyPermissions(
                ModifyableSequencePermissions::Write(SequenceWrite::HardErasure(
                    HardErasureCmd::HardUpdate,
                )),
            ))),
        ]
    }

    fn assert_sequence_read_permitted(data: &SequenceData, public_key: PublicKey, permitted: bool) {
        assert_eq!(
            data.is_permitted(get_sequence_read_query(SequenceQuery::ReadData), public_key),
            permitted
        );
        assert_eq!(
            data.is_permitted(
                get_sequence_read_query(SequenceQuery::ReadOwners),
                public_key
            ),
            permitted
        );
        assert_eq!(
            data.is_permitted(
                get_sequence_read_query(SequenceQuery::ReadPermissions),
                public_key
            ),
            permitted
        );
    }

    fn assert_modify_sequence_permissions_permitted(
        data: &SequenceData,
        public_key: PublicKey,
        permitted: bool,
    ) {
        assert_eq!(
            data.is_permitted(
                get_modify_sequence_permissions(ModifyableSequencePermissions::ReadData),
                public_key
            ),
            permitted
        );
        assert_eq!(
            data.is_permitted(
                get_modify_sequence_permissions(ModifyableSequencePermissions::ReadOwners),
                public_key
            ),
            permitted
        );
        assert_eq!(
            data.is_permitted(
                get_modify_sequence_permissions(ModifyableSequencePermissions::ReadPermissions),
                public_key
            ),
            permitted
        );
        assert_eq!(
            data.is_permitted(
                get_modify_sequence_permissions(ModifyableSequencePermissions::Write(
                    SequenceWrite::Append
                )),
                public_key
            ),
            permitted
        );
        assert_eq!(
            data.is_permitted(
                get_modify_sequence_permissions(ModifyableSequencePermissions::Write(
                    SequenceWrite::ModifyPermissions
                )),
                public_key
            ),
            permitted
        );
        assert_eq!(
            data.is_permitted(
                get_modify_sequence_permissions(ModifyableSequencePermissions::Write(
                    SequenceWrite::HardErasure(HardErasureCmd::HardDelete)
                )),
                public_key
            ),
            permitted
        );
        assert_eq!(
            data.is_permitted(
                get_modify_sequence_permissions(ModifyableSequencePermissions::Write(
                    SequenceWrite::HardErasure(HardErasureCmd::HardUpdate)
                )),
                public_key
            ),
            permitted
        );
    }

    // ------------------------------------------------------------------------------------------
    // -----------------------------------  MAP  ------------------------------------------------
    // ------------------------------------------------------------------------------------------

    #[test]
    fn set_map_permissions() {
        let mut data = PrivateSentriedMap::new(XorName([1; 32]), 10000);

        // Set the first permission set with correct ExpectedIndices - should pass.
        let res = data.set_permissions(
            PrivatePermissions {
                permissions: BTreeMap::new(),
                expected_data_index: 0,
                expected_owners_index: 0,
            },
            0,
        );

        match res {
            Ok(()) => (),
            Err(x) => panic!("Unexpected error: {:?}", x),
        }

        // Verify that the permissions are part of the history.
        assert_eq!(
            unwrap!(data.permission_history_range(Index::FromStart(0), Index::FromEnd(0),)).len(),
            1
        );

        // Set permissions with incorrect ExpectedIndices - should fail.
        let res = data.set_permissions(
            PrivatePermissions {
                permissions: BTreeMap::new(),
                expected_data_index: 64,
                expected_owners_index: 0,
            },
            1,
        );

        match res {
            Err(_) => (),
            Ok(()) => panic!("Unexpected Ok(()) result"),
        }

        // Verify that the history of permissions remains unchanged.
        assert_eq!(
            unwrap!(data.permission_history_range(Index::FromStart(0), Index::FromEnd(0),)).len(),
            1
        );
    }

    #[test]
    fn set_map_owner() {
        let owner_pk = gen_public_key();

        let mut data = PrivateSentriedMap::new(XorName([1; 32]), 10000);

        // Set the first owner with correct ExpectedIndices - should pass.
        let res = data.set_owner(
            Owner {
                public_key: owner_pk,
                expected_data_index: 0,
                expected_permissions_index: 0,
            },
            0,
        );

        match res {
            Ok(()) => (),
            Err(x) => panic!("Unexpected error: {:?}", x),
        }

        // Verify that the owner is part of history.
        assert_eq!(
            unwrap!(data.owner_history_range(Index::FromStart(0), Index::FromEnd(0),)).len(),
            1
        );

        // Set new owner with incorrect ExpectedIndices - should fail.
        let res = data.set_owner(
            Owner {
                public_key: owner_pk,
                expected_data_index: 64,
                expected_permissions_index: 0,
            },
            1,
        );

        match res {
            Err(_) => (),
            Ok(()) => panic!("Unexpected Ok(()) result"),
        }

        // Verify that the history of owners remains unchanged.
        assert_eq!(
            unwrap!(data.owner_history_range(Index::FromStart(0), Index::FromEnd(0),)).len(),
            1
        );
    }

    #[test]
    fn gets_map_shell() {
        let owner_pk = gen_public_key();
        let owner_pk1 = gen_public_key();

        let mut data = PrivateSentriedMap::new(XorName([1; 32]), 10000);

        let _ = data.set_owner(
            Owner {
                public_key: owner_pk,
                expected_data_index: 0,
                expected_permissions_index: 0,
            },
            0,
        );

        let _ = data.set_owner(
            Owner {
                public_key: owner_pk1,
                expected_data_index: 0,
                expected_permissions_index: 0,
            },
            1,
        );

        assert_eq!(
            data.expected_owners_index(),
            unwrap!(data.shell(0)).expected_owners_index()
        );
    }

    #[test]
    fn can_retrieve_map_permissions() {
        let public_key = gen_public_key();
        let invalid_public_key = gen_public_key();

        let mut pub_permissions = PublicPermissions {
            permissions: BTreeMap::new(),
            expected_data_index: 0,
            expected_owners_index: 0,
        };
        let _ = pub_permissions.permissions.insert(
            User::Specific(public_key),
            PublicPermissionSet::new(BTreeMap::new()),
        );

        let mut private_permissions = PrivatePermissions {
            permissions: BTreeMap::new(),
            expected_data_index: 0,
            expected_owners_index: 0,
        };
        let _ = private_permissions
            .permissions
            .insert(public_key, PrivatePermissionSet::new(BTreeMap::new()));

        // pub, unseq
        let mut data = PublicMap::new(rand::random(), 20);
        unwrap!(data.set_permissions(pub_permissions.clone(), 0));
        let data = MapData::from(data);

        assert_eq!(data.public_permissions_at(0), Ok(&pub_permissions));
        assert_eq!(data.private_permissions_at(0), Err(Error::NoSuchData));

        assert_eq!(
            data.public_user_permissions_at(User::Specific(public_key), 0),
            Ok(PublicPermissionSet::new(BTreeMap::new()))
        );
        assert_eq!(
            data.private_user_permissions_at(public_key, 0),
            Err(Error::NoSuchData)
        );
        assert_eq!(
            data.public_user_permissions_at(User::Specific(invalid_public_key), 0),
            Err(Error::NoSuchEntry)
        );

        // pub, seq
        let mut data = PublicSentriedMap::new(rand::random(), 20);
        unwrap!(data.set_permissions(pub_permissions.clone(), 0));
        let data = MapData::from(data);

        assert_eq!(data.public_permissions_at(0), Ok(&pub_permissions));
        assert_eq!(data.private_permissions_at(0), Err(Error::NoSuchData));

        assert_eq!(
            data.public_user_permissions_at(User::Specific(public_key), 0),
            Ok(PublicPermissionSet::new(BTreeMap::new()))
        );
        assert_eq!(
            data.private_user_permissions_at(public_key, 0),
            Err(Error::NoSuchData)
        );
        assert_eq!(
            data.public_user_permissions_at(User::Specific(invalid_public_key), 0),
            Err(Error::NoSuchEntry)
        );

        // Private, unseq
        let mut data = PrivateMap::new(rand::random(), 20);
        unwrap!(data.set_permissions(private_permissions.clone(), 0));
        let data = MapData::from(data);

        assert_eq!(data.private_permissions_at(0), Ok(&private_permissions));
        assert_eq!(data.public_permissions_at(0), Err(Error::NoSuchData));

        assert_eq!(
            data.private_user_permissions_at(public_key, 0),
            Ok(PrivatePermissionSet::new(BTreeMap::new()))
        );
        assert_eq!(
            data.public_user_permissions_at(User::Specific(public_key), 0),
            Err(Error::NoSuchData)
        );
        assert_eq!(
            data.private_user_permissions_at(invalid_public_key, 0),
            Err(Error::NoSuchEntry)
        );

        // Private, sentried
        let mut data = PrivateSentriedMap::new(rand::random(), 20);
        unwrap!(data.set_permissions(private_permissions.clone(), 0));
        let data = MapData::from(data);

        assert_eq!(data.private_permissions_at(0), Ok(&private_permissions));
        assert_eq!(data.public_permissions_at(0), Err(Error::NoSuchData));

        assert_eq!(
            data.private_user_permissions_at(public_key, 0),
            Ok(PrivatePermissionSet::new(BTreeMap::new()))
        );
        assert_eq!(
            data.public_user_permissions_at(User::Specific(public_key), 0),
            Err(Error::NoSuchData)
        );
        assert_eq!(
            data.private_user_permissions_at(invalid_public_key, 0),
            Err(Error::NoSuchEntry)
        );
    }

    #[test]
    fn validates_public_map_permissions() {
        let public_key_0 = gen_public_key();
        let public_key_1 = gen_public_key();
        let public_key_2 = gen_public_key();
        let mut map = PublicSentriedMap::new(XorName([1; 32]), 100);

        // no owner
        let data = MapData::from(map.clone());
        assert_eq!(data.is_permitted(get_insert_cmd(), public_key_0), false);
        // data is Public - read always allowed
        assert_map_read_permitted(&data, public_key_0, true);

        // no permissions
        unwrap!(map.set_owner(
            Owner {
                public_key: public_key_0,
                expected_data_index: 0,
                expected_permissions_index: 0,
            },
            0,
        ));
        let data = MapData::from(map.clone());

        assert_eq!(data.is_permitted(get_insert_cmd(), public_key_0), true);
        assert_eq!(data.is_permitted(get_insert_cmd(), public_key_1), false);
        // data is Public - read always allowed
        assert_map_read_permitted(&data, public_key_0, true);
        assert_map_read_permitted(&data, public_key_1, true);

        // with permissions
        let mut permissions = PublicPermissions {
            permissions: BTreeMap::new(),
            expected_data_index: 0,
            expected_owners_index: 1,
        };
        let mut set = BTreeMap::new();
        let _ = set.insert(get_insert_cmd(), true);
        let _ = permissions
            .permissions
            .insert(User::Anyone, PublicPermissionSet::new(set));
        let mut set = BTreeMap::new();
        for cmd in get_full_modify_map_permissions() {
            let _ = set.insert(cmd, true);
        }
        let _ = permissions
            .permissions
            .insert(User::Specific(public_key_1), PublicPermissionSet::new(set));
        unwrap!(map.set_permissions(permissions, 0));
        let data = MapData::from(map);

        // existing key fallback
        assert_eq!(data.is_permitted(get_insert_cmd(), public_key_1), true);
        // existing key override
        assert_modify_map_permissions_permitted(&data, public_key_1, true);
        // non-existing keys are handled by `Anyone`
        assert_eq!(data.is_permitted(get_insert_cmd(), public_key_2), true);
        assert_modify_map_permissions_permitted(&data, public_key_2, false);
        // data is Public - read always allowed
        assert_map_read_permitted(&data, public_key_0, true);
        assert_map_read_permitted(&data, public_key_1, true);
        assert_map_read_permitted(&data, public_key_2, true);
    }

    #[test]
    fn validates_private_map_permissions() {
        let public_key_0 = gen_public_key();
        let public_key_1 = gen_public_key();
        let public_key_2 = gen_public_key();
        let mut map = PrivateSentriedMap::new(XorName([1; 32]), 100);

        // no owner
        let data = MapData::from(map.clone());
        assert_map_read_permitted(&data, public_key_0, false);

        // no permissions
        unwrap!(map.set_owner(
            Owner {
                public_key: public_key_0,
                expected_data_index: 0,
                expected_permissions_index: 0,
            },
            0,
        ));
        let data = MapData::from(map.clone());

        assert_map_read_permitted(&data, public_key_0, true);
        assert_map_read_permitted(&data, public_key_1, false);

        // with permissions
        let mut permissions = PrivatePermissions {
            permissions: BTreeMap::new(),
            expected_data_index: 0,
            expected_owners_index: 1,
        };
        let mut set = BTreeMap::new();
        let _ = set.insert(get_insert_cmd(), true);
        for query in get_full_map_read_permissions() {
            let _ = set.insert(query, true);
        }
        for cmd in get_full_modify_map_permissions() {
            let _ = set.insert(cmd, false);
        }
        let _ = permissions
            .permissions
            .insert(public_key_1, PrivatePermissionSet::new(set));
        unwrap!(map.set_permissions(permissions, 0));
        let data = MapData::from(map);

        // existing key
        assert_map_read_permitted(&data, public_key_1, true);
        assert_eq!(data.is_permitted(get_insert_cmd(), public_key_1), true);
        assert_modify_map_permissions_permitted(&data, public_key_1, false);

        // non-existing key
        assert_map_read_permitted(&data, public_key_2, false);
        assert_eq!(data.is_permitted(get_insert_cmd(), public_key_2), false);
        assert_modify_map_permissions_permitted(&data, public_key_2, false);
    }

    fn get_insert_cmd() -> Request {
        Request::Cmd(CmdType::Map(MapCmd::Insert))
    }

    fn get_map_read_query(query: MapQuery) -> Request {
        Request::Query(QueryType::Map(query))
    }

    fn get_full_map_read_permissions() -> Vec<Request> {
        vec![
            Request::Query(QueryType::Map(MapQuery::ReadData)),
            Request::Query(QueryType::Map(MapQuery::ReadOwners)),
            Request::Query(QueryType::Map(MapQuery::ReadPermissions)),
        ]
    }

    fn get_modify_map_permissions(permission: ModifyableMapPermissions) -> Request {
        Request::Cmd(CmdType::Map(MapCmd::ModifyPermissions(permission)))
    }

    fn get_full_modify_map_permissions() -> Vec<Request> {
        vec![
            Request::Cmd(CmdType::Map(MapCmd::ModifyPermissions(
                ModifyableMapPermissions::ReadData,
            ))),
            Request::Cmd(CmdType::Map(MapCmd::ModifyPermissions(
                ModifyableMapPermissions::ReadOwners,
            ))),
            Request::Cmd(CmdType::Map(MapCmd::ModifyPermissions(
                ModifyableMapPermissions::ReadPermissions,
            ))),
            Request::Cmd(CmdType::Map(MapCmd::ModifyPermissions(
                ModifyableMapPermissions::Write(MapWrite::Insert),
            ))),
            Request::Cmd(CmdType::Map(MapCmd::ModifyPermissions(
                ModifyableMapPermissions::Write(MapWrite::Update),
            ))),
            Request::Cmd(CmdType::Map(MapCmd::ModifyPermissions(
                ModifyableMapPermissions::Write(MapWrite::Delete),
            ))),
            Request::Cmd(CmdType::Map(MapCmd::ModifyPermissions(
                ModifyableMapPermissions::Write(MapWrite::ModifyPermissions),
            ))),
            Request::Cmd(CmdType::Map(MapCmd::ModifyPermissions(
                ModifyableMapPermissions::Write(MapWrite::HardErasure(HardErasureCmd::HardDelete)),
            ))),
            Request::Cmd(CmdType::Map(MapCmd::ModifyPermissions(
                ModifyableMapPermissions::Write(MapWrite::HardErasure(HardErasureCmd::HardUpdate)),
            ))),
        ]
    }

    fn assert_map_read_permitted(data: &MapData, public_key: PublicKey, permitted: bool) {
        assert_eq!(
            data.is_permitted(get_map_read_query(MapQuery::ReadData), public_key),
            permitted
        );
        assert_eq!(
            data.is_permitted(get_map_read_query(MapQuery::ReadOwners), public_key),
            permitted
        );
        assert_eq!(
            data.is_permitted(get_map_read_query(MapQuery::ReadPermissions), public_key),
            permitted
        );
    }

    fn assert_modify_map_permissions_permitted(
        data: &MapData,
        public_key: PublicKey,
        permitted: bool,
    ) {
        assert_eq!(
            data.is_permitted(
                get_modify_map_permissions(ModifyableMapPermissions::ReadData),
                public_key
            ),
            permitted
        );
        assert_eq!(
            data.is_permitted(
                get_modify_map_permissions(ModifyableMapPermissions::ReadOwners),
                public_key
            ),
            permitted
        );
        assert_eq!(
            data.is_permitted(
                get_modify_map_permissions(ModifyableMapPermissions::ReadPermissions),
                public_key
            ),
            permitted
        );
        assert_eq!(
            data.is_permitted(
                get_modify_map_permissions(ModifyableMapPermissions::Write(MapWrite::Insert)),
                public_key
            ),
            permitted
        );
        assert_eq!(
            data.is_permitted(
                get_modify_map_permissions(ModifyableMapPermissions::Write(MapWrite::Update)),
                public_key
            ),
            permitted
        );
        assert_eq!(
            data.is_permitted(
                get_modify_map_permissions(ModifyableMapPermissions::Write(MapWrite::Delete)),
                public_key
            ),
            permitted
        );
        assert_eq!(
            data.is_permitted(
                get_modify_map_permissions(ModifyableMapPermissions::Write(
                    MapWrite::ModifyPermissions
                )),
                public_key
            ),
            permitted
        );
        assert_eq!(
            data.is_permitted(
                get_modify_map_permissions(ModifyableMapPermissions::Write(MapWrite::HardErasure(
                    HardErasureCmd::HardDelete
                ))),
                public_key
            ),
            permitted
        );
        assert_eq!(
            data.is_permitted(
                get_modify_map_permissions(ModifyableMapPermissions::Write(MapWrite::HardErasure(
                    HardErasureCmd::HardUpdate
                ))),
                public_key
            ),
            permitted
        );
    }
}