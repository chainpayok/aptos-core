// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0
#![allow(clippy::extra_unused_lifetimes)]
use crate::{models::transactions::Transaction, schema::move_modules};
use aptos_api_types::{DeleteModule, MoveModule as APIMoveModule, MoveModuleBytecode, WriteModule};
use field_count::FieldCount;
use serde::Serialize;

#[derive(
    Associations, Clone, Debug, FieldCount, Identifiable, Insertable, Queryable, Serialize,
)]
#[belongs_to(Transaction, foreign_key = "transaction_version")]
#[primary_key(transaction_version, write_set_change_index)]
#[diesel(table_name = "move_modules")]
pub struct MoveModule {
    pub transaction_version: i64,
    pub write_set_change_index: i64,
    pub name: String,
    pub address: String,
    pub bytecode: Option<Vec<u8>>,
    pub exposed_functions: Option<serde_json::Value>,
    pub friends: Option<serde_json::Value>,
    pub structs: Option<serde_json::Value>,
    pub is_deleted: bool,
    // Default time columns
    pub inserted_at: chrono::NaiveDateTime,
}

pub struct MoveModuleByteCodeParsed {
    pub address: String,
    pub name: String,
    pub bytecode: Vec<u8>,
    pub exposed_functions: serde_json::Value,
    pub friends: serde_json::Value,
    pub structs: serde_json::Value,
}

impl MoveModule {
    pub fn from_write_module(
        write_module: &WriteModule,
        write_set_change_index: i64,
        transaction_version: i64,
    ) -> Self {
        let parsed_data = Self::convert_move_module_bytecode(&write_module.data);
        Self {
            transaction_version,
            write_set_change_index,
            name: parsed_data
                .as_ref()
                .map(|d| d.name.clone())
                .unwrap_or_default(),
            address: write_module.address.to_string(),
            bytecode: parsed_data.as_ref().map(|d| d.bytecode.clone()),
            exposed_functions: parsed_data.as_ref().map(|d| d.exposed_functions.clone()),
            friends: parsed_data.as_ref().map(|d| d.friends.clone()),
            structs: parsed_data.as_ref().map(|d| d.structs.clone()),
            is_deleted: false,
            inserted_at: chrono::Utc::now().naive_utc(),
        }
    }

    pub fn from_delete_module(
        delete_module: &DeleteModule,
        write_set_change_index: i64,
        transaction_version: i64,
    ) -> Self {
        Self {
            transaction_version,
            write_set_change_index,
            name: delete_module.module.name.to_string(),
            address: delete_module.address.to_string(),
            bytecode: None,
            exposed_functions: None,
            friends: None,
            structs: None,
            is_deleted: true,
            inserted_at: chrono::Utc::now().naive_utc(),
        }
    }

    pub fn convert_move_module_bytecode(
        mmb: &MoveModuleBytecode,
    ) -> Option<MoveModuleByteCodeParsed> {
        mmb.clone().try_parse_abi().map_or_else(
            |e| {
                aptos_logger::warn!(
                    "[fh-stream] Could not decode MoveModuleBytecode ABI. Error: {:?}",
                    e
                );
                None
            },
            |mmb| {
                mmb.abi.as_ref().map(|move_module| {
                    Self::convert_move_module(move_module, mmb.bytecode.0.clone())
                })
            },
        )
    }

    pub fn convert_move_module(
        move_module: &APIMoveModule,
        bytecode: Vec<u8>,
    ) -> MoveModuleByteCodeParsed {
        MoveModuleByteCodeParsed {
            address: move_module.address.to_string(),
            name: move_module.name.0.to_string(),
            bytecode,
            exposed_functions: move_module
                .exposed_functions
                .iter()
                .map(|move_func| serde_json::to_value(move_func).unwrap())
                .collect(),
            friends: move_module
                .friends
                .iter()
                .map(|move_module_id| serde_json::to_value(move_module_id).unwrap())
                .collect(),
            structs: move_module
                .structs
                .iter()
                .map(|move_struct| serde_json::to_value(move_struct).unwrap())
                .collect(),
        }
    }
}
