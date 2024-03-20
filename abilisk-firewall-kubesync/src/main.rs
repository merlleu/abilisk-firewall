use std::collections::HashMap;

use futures::TryStreamExt;
use k8s_openapi::api::core::v1::Node;
use kube::{
    api::Api,
    runtime::{watcher, WatchStreamExt},
    Client, ResourceExt,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let client = Client::try_default().await?;

    let api: Api<Node> = Api::all(client);

    let stream = watcher(api, watcher::Config::default().any_semantic()).default_backoff();

    futures::pin_mut!(stream);

    let mut ips = HashMap::new();

    while let Some(event) = stream.try_next().await? {
        match event {
            watcher::Event::Applied(node) => {
                extract_public_ip(&node).map(|ip| {
                    ips.insert(node.name_any(), ip.to_string());
                });
            }
            watcher::Event::Restarted(nodes) => {
                ips.clear();
                nodes.into_iter().for_each(|node| {
                    extract_public_ip(&node).map(|ip| {
                        ips.insert(node.name_any(), ip.to_string());
                    });
                });
            }
            watcher::Event::Deleted(node) => {
                extract_public_ip(&node).map(|ip| {
                    ips.remove(&node.name_any());
                });
            }
        }
    }

    Ok(())
}

fn extract_public_ip<'a>(node: &'a Node) -> Option<&'a String> {
    for a in node.status.as_ref()?.addresses.as_ref()?.iter() {
        if a.type_ == "ExternalIP" {
            return Some(&a.address);
        }
    }
    None
}

// fn remove_unused_properties(node: &mut Node) {
//     node.managed_fields_mut().clear();
//     node.annotations_mut().clear();
//     if let Some(status) = node.status.take() {
//         node.status = Some(NodeStatus {
//             addresses: status.addresses,
//             ..Default::default()
//         });
//     }
// }
