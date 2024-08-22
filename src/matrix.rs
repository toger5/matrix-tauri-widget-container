use matrix_sdk::Client;

pub(crate) async fn login(
    homeserver_url: String,
    username: String,
    password: String,
) -> Result<Client, matrix_sdk::Error> {
    // Note that when encryption is enabled, you should use a persistent store to be
    // able to restore the session with a working encryption setup.
    // See the `persist_session` example.
    let client = Client::builder()
        .homeserver_url(homeserver_url)
        .build()
        .await
        .unwrap();
    client
        .matrix_auth()
        .login_username(&username, &password)
        .initial_device_display_name("tauri widget container")
        .await?;

    println!("logged in as {username}");
    Ok(client)
}
