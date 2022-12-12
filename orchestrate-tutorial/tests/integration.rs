use cosmwasm_orchestrate::{
    block,
    cosmwasm_std::{Coin, MessageInfo},
    env, info,
    vm::*,
    Direct, JunoApi, StateBuilder, WasmLoader,
};
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw20::{BalanceResponse, Cw20Coin, Cw20ExecuteMsg, Cw20QueryMsg, Denom};
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
use wasmswap::msg::{ExecuteMsg, InstantiateMsg, TokenSelect};

fn initialize() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        env_logger::init();
    });
}

#[test]
fn works() {
    initialize();
    let code = WasmLoader::new(env!("CARGO_PKG_NAME")).load().unwrap();
    let cw20_code = include_bytes!("../scripts/cw20_base.wasm");
    // Generate a Juno compatible address
    let sender = Account::generate_from_seed::<JunoAddressHandler>("sender").unwrap();
    let swapper = Account::generate_from_seed::<JunoAddressHandler>("swapper").unwrap();
    // Create a VM state by providing the codes that will be executed.
    let mut state = StateBuilder::new()
        .add_code(&code)
        .add_balance(sender.clone(), Coin::new(100_000_000_000_000, "cosm"))
        .add_balance(swapper.clone(), Coin::new(120_000_000, "cosm"))
        .add_code(cw20_code)
        .build();

    // Instantiate the cw20 contract
    let (cw20_address, _) = <JunoApi>::instantiate(
        &mut state,
        2,
        None,
        block(),
        None,
        info(&sender),
        100_000_000_000,
        Cw20InstantiateMsg {
            name: "Picasso".into(),
            symbol: "PICA".into(),
            decimals: 10,
            initial_balances: vec![Cw20Coin {
                address: sender.clone().into(),
                amount: Uint128::new(100_000_000_000_000),
            }],
            mint: None,
            marketing: None,
        },
    )
    .unwrap();

    let (contract_addr, _) = <JunoApi>::instantiate(
        &mut state,
        1,
        None,
        block(),
        None,
        info(&sender),
        100_000_000_000,
        InstantiateMsg {
            token1_denom: Denom::Native("cosm".into()),
            token2_denom: Denom::Cw20(Addr::unchecked(cw20_address.clone())),
            lp_token_code_id: 2,
            owner: None,
            protocol_fee_recipient: sender.clone().into(),
            protocol_fee_percent: Decimal::zero(),
            lp_fee_percent: Decimal::zero(),
        },
    )
    .unwrap();

    let _ = <JunoApi>::execute(
        &mut state,
        env(&cw20_address),
        info(&sender),
        100_000_000_000,
        Cw20ExecuteMsg::IncreaseAllowance {
            spender: contract_addr.clone().into(),
            amount: Uint128::new(100_000_000_000),
            expires: Some(cw0::Expiration::Never {}),
        },
    )
    .unwrap();

    let _ = <JunoApi>::execute(
        &mut state,
        env(&contract_addr),
        MessageInfo {
            sender: sender.clone().into(),
            funds: vec![Coin::new(400_000_000, "cosm")],
        },
        100_000_000_000,
        ExecuteMsg::AddLiquidity {
            token1_amount: 400_000_000u128.into(),
            min_liquidity: 50_000u128.into(),
            max_token2: 300_000_000u128.into(),
            expiration: None,
        },
    )
    .unwrap();

    let _ = <JunoApi>::execute(
        &mut state,
        env(&contract_addr),
        MessageInfo {
            sender: swapper.clone().into(),
            funds: vec![Coin::new(120_000_000, "cosm")],
        },
        100_000_000_000,
        ExecuteMsg::Swap {
            input_token: TokenSelect::Token1,
            input_amount: Uint128::new(120_000_000),
            min_output: Uint128::zero(),
            expiration: None,
        },
    )
    .unwrap();

    let query_res: BalanceResponse = <JunoApi<Direct>>::query(
        &mut state,
        env(&cw20_address),
        Cw20QueryMsg::Balance {
            address: swapper.clone().into(),
        },
    )
    .unwrap();

    assert_eq!(
        query_res.balance,
        Uint128::new((120_000_000 * 300_000_000) / (400_000_000 + 120_000_000))
    );
}
