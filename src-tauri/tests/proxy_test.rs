use colima_desktop_lib::proxy::server::ProxyServer;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_proxy_routes_to_backend() {
    // 1. Create proxy with a route: echo-test.local → 3099
    let proxy = ProxyServer::new(17080); // use high port to avoid conflicts
    let routes = proxy.routes();
    let shutdown = proxy.shutdown_handle();

    {
        let mut table = routes.lock().await;
        table.insert("echo-test.local".to_string(), 3099);
    }

    // 2. Start proxy in background
    tokio::spawn(async move {
        let _ = proxy.run().await;
    });

    // Give the proxy a moment to bind
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 3. Send request to proxy with Host header
    let client = reqwest::Client::new();
    let resp = client
        .get("http://127.0.0.1:17080/hello")
        .header("Host", "echo-test.local")
        .send()
        .await
        .expect("request to proxy should succeed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("should be JSON");
    assert_eq!(body["url"], "/hello");
    assert_eq!(body["message"], "Hello from echo server on port 3099!");

    // 4. Test unknown host returns 502
    let resp_bad = client
        .get("http://127.0.0.1:17080/")
        .header("Host", "unknown.local")
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp_bad.status(), 502);

    // 5. Shutdown
    shutdown.notify_one();
}
