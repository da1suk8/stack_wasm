use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    from_slice, to_binary, to_vec, Binary, Deps, DepsMut, Env, MessageInfo, Order,
    QueryResponse, Response, StdResult, Storage,
};

use crate::msg::{InstantiateMsg};

// we store one entry for each item in the stack
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Item {
    pub value: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // Push will add some value to the end of list
    Push { value: i32 },
    // Pop will remove value from end of the list
    Pop {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // how many items are in the stack
    Count {},
    // total of all values in the stack
    Sum {},

    List {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SumResponse {
    pub sum: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ListResponse {
    /// List an empty range, both bounded
    pub empty: Vec<u32>,
    /// List all IDs lower than 0x20
    pub early: Vec<u32>,
    /// List all IDs starting from 0x20
    pub late: Vec<u32>,
}

// A no-op, just empty data
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    println!("-- Instantiate --");
    Ok(Response::default())
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Push { value } => handle_push(deps, value),
        ExecuteMsg::Pop {} => handle_pop(deps),
    }
}

const FIRST_KEY: u8 = 0;

fn handle_push(deps: DepsMut, value: i32) -> StdResult<Response> {
    println!("Push value {}", value);
    push(deps.storage, value)?;
    Ok(Response::default())
}

fn push(storage: &mut dyn Storage, value: i32) -> StdResult<()> {
    // find the last element in the queue and extract key
    let last_item = storage.range(None, None, Order::Ascending).next();

    let new_key = match last_item {
        None => FIRST_KEY,
        Some((key, _)) => {
            key[0] + 1 // all keys are one byte
        }
    };
    let new_value = to_vec(&Item { value })?;

    storage.set(&[new_key], &new_value);
    Ok(())
}

// #[allow(clippy::unnecessary_wraps)]
fn handle_pop(deps: DepsMut) -> StdResult<Response> {
    // find the first element in the queue and extract value
    let first = deps.storage.range(None, None, Order::Descending).next();

    let mut res = Response::default();
    if let Some((key, value)) = first {
        // remove from storage and return old value
        deps.storage.remove(&key);
        res.data = Some(Binary(value));
        Ok(res)
    } else {
        Ok(res)
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Count {} => to_binary(&stack_count(deps)),
        QueryMsg::Sum {} => to_binary(&stack_sum(deps)?),
        QueryMsg::List {} => to_binary(&stack_list(deps)),
    }
}

fn stack_count(deps: Deps) -> CountResponse {
    let count = deps.storage.range(None, None, Order::Ascending).count() as u32;
    CountResponse { count }
}

fn stack_sum(deps: Deps) -> StdResult<SumResponse> {
    let values: StdResult<Vec<Item>> = deps
        .storage
        .range(None, None, Order::Ascending)
        .map(|(_, v)| from_slice(&v))
        .collect();
    let sum = values?.iter().fold(0, |s, v| s + v.value);
    Ok(SumResponse { sum })
}

/// Does a range query with both bounds set. Not really useful but to debug an issue
/// between VM and Wasm: https://github.com/CosmWasm/cosmwasm/issues/508
fn stack_list(deps: Deps) -> ListResponse {
    let empty: Vec<u32> = deps
        .storage
        .range(Some(b"large"), Some(b"larger"), Order::Ascending)
        .map(|(k, _)| k[0] as u32)
        .collect();
    let early: Vec<u32> = deps
        .storage
        .range(None, Some(b"\x20"), Order::Ascending)
        .map(|(k, _)| k[0] as u32)
        .collect();
    let late: Vec<u32> = deps
        .storage
        .range(Some(b"\x20"), None, Order::Ascending)
        .map(|(k, _)| k[0] as u32)
        .collect();
    ListResponse { empty, early, late }
}
