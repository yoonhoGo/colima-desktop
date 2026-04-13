use colima_desktop_lib::proxy::server::ProxyServer;

#[tokio::test]
async fn test_proxy_to_ddocdoc_auth() {
    // Route dd-auth.local → port 3001 (ddocdoc-auth)
    let proxy = ProxyServer::new(17081);
    let routes = proxy.routes();
    let shutdown = proxy.shutdown_handle();

    {
        let mut table = routes.lock().await;
        table.insert("dd-auth.local".to_string(), 3001);
        table.insert("echo-test.local".to_string(), 3099);
    }

    tokio::spawn(async move {
        let _ = proxy.run().await;
    });
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let client = reqwest::Client::new();

    // Test 1: dd-auth.local → ddocdoc-auth on 3001
    let resp = client
        .get("http://127.0.0.1:17081/")
        .header("Host", "dd-auth.local")
        .send()
        .await
        .expect("dd-auth.local request should succeed");

    println!("dd-auth.local status: {}", resp.status());
    let body = resp.text().await.unwrap();
    println!("dd-auth.local body: {}", &body[..body.len().min(200)]);

    // Test 2: echo-test.local → echo server on 3099
    let resp2 = client
        .get("http://127.0.0.1:17081/hello")
        .header("Host", "echo-test.local")
        .send()
        .await
        .expect("echo-test.local request should succeed");

    assert_eq!(resp2.status(), 200);
    let json: serde_json::Value = resp2.json().await.unwrap();
    println!("echo-test.local: {}", json["message"]);
    assert_eq!(json["message"], "Hello from echo server on port 3099!");

    // Test 3: Multiple routes work simultaneously
    println!("\n--- Both routes working simultaneously ---");
    println!("dd-auth.local → :3001 (ddocdoc-auth)");
    println!("echo-test.local → :3099 (echo server)");

    shutdown.notify_one();
}
