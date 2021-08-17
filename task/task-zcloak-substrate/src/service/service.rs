use lifeline::{Bus, Lifeline, Receiver, Sender, Service, Task};

use server_traits::server::component::ServerComponent;
use server_traits::server::service::ServerService;
use server_traits::server::task::{ServerSand, ServerTask};
use server_traits::server::config::Config;
use std::error;

use crate::bus::ZcloakTaskBus;
use crate::message::ZcloakTaskMessage;
use crate::task::ZcloakTask;
use crate::config::ZcloakNodeConfig;
use primitives::utils::ipfs::config::IpfsConfig;
use support_zcloak_node::client::Zcloak;
use support_zcloak_node::runtime::ZcloakRuntime;
use support_zcloak_node::account::ZcloakAccount;
use components_subxt_client::SubstrateClient;

#[derive(Debug)]
pub struct TaskService {
    _greet: Lifeline
}

impl ServerService for TaskService {}

impl Service for TaskService {
    type Bus = ZcloakTaskBus;
    type Lifeline = anyhow::Result<Self>;

    fn spawn(bus: &Self::Bus) -> Self::Lifeline {
        let mut rx = bus.rx::<ZcloakTaskMessage>()?;

        let zcloak_node_config: ZcloakNodeConfig = Config::restore_with_namespace(ZcloakTask::NAME, "zcloak")?;
        let ipfs_config: IpfsConfig = Config::restore_with_namespace(ZcloakTask::NAME, "ipfs")?;
        let _greet = Self::try_task(
            &format!("{}-service-task", ZcloakTask::NAME),
            async move {
                while let Some(message) = rx.recv().await {
                    match message {
                        ZcloakTaskMessage::TaskEvent => {
                            log::info!("start zCloak server's starks verifier server ");
                            log::info!("zcloak node url is {:?}", zcloak_node_config.url);
                            log::info!("ipfs ip is {:?}", ipfs_config.url_index);
                            let ipfs_url = ipfs_config.url_index.clone();
                            let subxt_client = SubstrateClient::<ZcloakRuntime>::new(zcloak_node_config.url.clone()).await?;
                            let zcloak_account = ZcloakAccount::new(zcloak_node_config.private_key.clone());
                            let zcloak_client = Zcloak::new(subxt_client, zcloak_account);
                            
                            // zcloak_client.subscribe_events(ipfs_config.clone()).await?;
                            tokio::spawn(async move {
                                run_verifer(zcloak_client, ipfs_url)
                                .await
                            });
                            log::info!("zCloak server's starks verifier server is running")
                        }
                        _ => continue,
                    }
                    log::debug!(
                        target: ZcloakTask::NAME,
                        "[{}] recv a new task message: {:?}",
                        ZcloakTask::NAME,
                        message
                    );

                }
                Ok(())
            },
        );
        Ok(Self { _greet })
    }
}

async fn run_transfer(client: Zcloak){
    if let Err(err) = client.subscribe_transfer_events().await{
        log::error!(
            target: ZcloakTask::NAME,
            "subscribe transfer events error {:#?}", err
        );
    }
}

async fn run_verifer(client: Zcloak, config: String, ) {
    if let Err(err) = client.subscribe_events(config).await{
        log::error!(
            target: ZcloakTask::NAME,
            "subscribe verifier events error {:#?}", err
        )
    }
}