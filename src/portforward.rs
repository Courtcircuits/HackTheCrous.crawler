use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::Api,
    runtime::wait::{await_condition, conditions::is_pod_running},
    Client, ResourceExt,
};
use tokio::{io::copy_bidirectional, net::TcpListener};

pub async fn port_forward() -> tokio::task::JoinHandle<()> {
    let handle = tokio::spawn(async move {
        let client = Client::try_default().await.unwrap();
        let pods: Api<Pod> = Api::namespaced(client, "hackthecrous");
        let _ = await_condition(
            pods.clone(),
            "hackthecrous-services-postgresql-0",
            is_pod_running(),
        );

        let listenner = TcpListener::bind("0.0.0.0:5432").await.unwrap();
        loop {
            let (mut stream, _) = listenner.accept().await.unwrap();
            let pods = pods.clone();
            tokio::spawn(async move {
                let mut pf = pods
                    .portforward("hackthecrous-services-postgresql-0", &[5432])
                    .await
                    .unwrap();
                let mut port = pf.take_stream(5432).unwrap();
                if let Err(e) = copy_bidirectional(&mut stream, &mut port).await {
                    eprintln!("error: {}", e);
                }
            });
        }
    });
    return handle;
}
