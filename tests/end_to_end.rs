use serde_json::json;

#[tokio::test]
async fn test_end_to_end() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;

    let operator_account = worker.dev_create_account().await?;

    let wasm = near_workspaces::compile_project("./").await?;
    let contract = worker.dev_deploy(&wasm).await?;
    contract
        .call("new")
        .args_json(json!({ "operator_account_id": operator_account.id() }))
        .deposit(near_sdk::ONE_NEAR)
        .transact()
        .await?
        .into_result()?;

    assert_eq!(
        contract
            .call("get_balance")
            .args_json(json!({ "account_id": operator_account.id() }))
            .view()
            .await?
            .result,
        b"\"1000000000000000000000000\""
    );

    let user_account = worker.dev_create_account().await?;

    assert!(user_account
        .call(contract.id(), "near_transfer")
        .args_json(json!({ "receiver_account_id": operator_account.id(), "amount": "0" }))
        .max_gas()
        .transact()
        .await?
        .into_result()
        .is_err());

    assert!(user_account
        .call(contract.id(), "near_deposit")
        .deposit(near_sdk::ONE_NEAR)
        .max_gas()
        .transact()
        .await?
        .into_result()
        .is_err());

    operator_account.call(contract.id(), "near_transfer")
        .args_json(json!({ "receiver_account_id": user_account.id(), "amount": "100000000000000000000000" }))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    assert_eq!(
        contract
            .call("get_balance")
            .args_json(json!({ "account_id": operator_account.id() }))
            .view()
            .await?
            .result,
        b"\"900000000000000000000000\""
    );

    assert_eq!(
        contract
            .call("get_balance")
            .args_json(json!({ "account_id": user_account.id() }))
            .view()
            .await?
            .result,
        b"\"100000000000000000000000\""
    );

    user_account
        .call(contract.id(), "near_deposit")
        .deposit(near_sdk::ONE_NEAR)
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    assert_eq!(
        contract
            .call("get_balance")
            .args_json(json!({ "account_id": user_account.id() }))
            .view()
            .await?
            .result,
        b"\"1100000000000000000000000\""
    );

    let balance_before_withdraw = user_account.view_account().await?.balance;

    user_account
        .call(contract.id(), "near_withdraw")
        .args_json(json!({ "amount": "1100000000000000000000000" }))
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    assert_eq!(
        contract
            .call("get_balance")
            .args_json(json!({ "account_id": user_account.id() }))
            .view()
            .await?
            .result,
        b"\"0\""
    );

    assert!(user_account.view_account().await?.balance > balance_before_withdraw);

    Ok(())
}
