#![allow(clippy::result_large_err)]

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ecs::{config::BehaviorVersion, config::Region, meta::PKG_VERSION, Error};
use clap::Parser;

const DEFAULT_REGION: &str = "us-west-2";

#[derive(Debug, Parser)]
struct Opt {
    /// The AWS Region. Defaults to us-west-2
    #[structopt(short, long, default_value = "us-west-2")]
    region: Option<String>,

    /// The ECS cluster name
    #[structopt(short, long, required = true)]
    cluster: String,

    /// The ECS service name
    #[structopt(short, long, required = true)]
    service_name: String,

    /// The ECS task container name
    #[structopt(long, short='n', required = true)]
    container_name: String,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

// returns a connection string suitable for remotely connecting to a Fargate task
async fn get_fargate_connection_string(
    client: &aws_sdk_ecs::Client,
    cluster: String,
    container_name: String,
    service_name: String,
) -> Result<(), aws_sdk_ecs::Error> {

    let tasks = client
        .list_tasks()
        .set_cluster(Option::from(cluster.clone()))
        .set_service_name(service_name.into())
        .send()
        .await?;

    for task_arn in tasks.task_arns() {
        let command = format!(
            r#"aws ecs execute-command  \
        --region {region} \
        --cluster {cluster} \
        --task {task_arn} \
        --container {container_name} \
        --command "/bin/bash" \
        --interactive
    "#,
            region = "us-west-2",
            cluster = cluster,
            container_name = container_name,
            task_arn = task_arn,
        );
        println!("{}", command);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let Opt {
        region,
        verbose,
        cluster,
        container_name,
        service_name,
    } = Opt::parse();

    let region_provider = RegionProviderChain::first_try(region.clone().map(Region::new))
        .or_default_provider()
        .or_else(Region::new(DEFAULT_REGION));

    if verbose {
        println!();
        println!("ECS client version: {}", PKG_VERSION);
        println!(
            "Region:             {}",
            region_provider.region().await.unwrap().as_ref()
        );
        println!("Cluster:            {}", cluster);
        println!();
    }

    let shared_config = aws_config::defaults(BehaviorVersion::v2024_03_28())
        .region(region_provider)
        .load()
        .await;
    let client = aws_sdk_ecs::Client::new(&shared_config);

    // show_clusters(&client).await
    get_fargate_connection_string(&client, cluster, container_name, service_name).await
}
