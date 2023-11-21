use zilliqa_rs::providers::{Http, Provider};

#[tokio::test]
async fn get_balance_should_return_balance_if_account_exist() {
    let provider = Provider::<Http>::try_from("http://127.0.0.1:5555").unwrap();

    println!(
        "{:?}",
        provider
            .get_balance("0x381f4008505e940ad7681ec3468a719060caf796")
            .await
    );
}
