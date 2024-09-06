#![allow(clippy::result_large_err)]

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ecs::{config::BehaviorVersion, config::Region, Error};
use clap::Parser;

const DEFAULT_REGION: &str = "us-west-2";

#[derive(Debug, Parser)]
#[clap(
    name = "ecs-jumper",
    about = "Generate ECS Fargate connection strings",
    version = env!("CARGO_PKG_VERSION")
)]
struct Opt {
    /// The AWS Region.
    #[clap(short, long, default_value = DEFAULT_REGION)]
    region: String,

    /// The ECS cluster name
    #[clap(short, long)]
    cluster: String,

    /// The ECS service name
    #[clap(short, long)]
    service_name: String,

    /// The ECS task container name
    #[clap(short = 'n', long)]
    container_name: String,

    /// Whether to display additional information.
    #[clap(short, long)]
    verbose: bool,
}

fn extract_image_tag(image: &str) -> Option<&str> {
    image.split(':').nth(1)
}

async fn get_fargate_connection_string(
    client: &aws_sdk_ecs::Client,
    cluster: &str,
    container_name: &str,
    service_name: &str,
    region: &str,
) -> Result<(), Error> {
    let tasks = client
        .list_tasks()
        .cluster(cluster)
        .service_name(service_name)
        .send()
        .await?;

    for task_arn in tasks.task_arns() {
        let task_description = client
            .describe_tasks()
            .cluster(cluster)
            .tasks(task_arn)
            .send()
            .await?;

        for task in task_description.tasks() {
            for container in task.containers() {
                if container.name() == Some(container_name) {
                    if let Some(image) = container.image() {
                        println!("---------------"); // Add a blank line for readability
                        println!("Container Image: {}", image);
                        if let Some(tag) = extract_image_tag(image) {
                            println!("Running image Tag: {}", tag);
                            println!(); // Add a blank line for readability
                        } else {
                            println!("Image Tag: Not found or using 'latest'");
                            println!(); // Add a blank line for readability
                        }
                    }
                }
            }
        }

        let command = format!(
            r#"aws ecs execute-command \
                --region {region} \
                --cluster {cluster} \
                --task {task_arn} \
                --container {container_name} \
                --command "/bin/bash" \
                --interactive"#
        );
        println!("\n{}", command);
        println!(); // Add a blank line for readability
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opt = Opt::parse();

    let region_provider = RegionProviderChain::first_try(Region::new(opt.region.clone()))
        .or_default_provider()
        .or_else(Region::new(DEFAULT_REGION));

    if opt.verbose {
        println!();
        println!("ECS client version: {}", env!("CARGO_PKG_VERSION"));
        println!(
            "Region:             {}",
            region_provider.region().await.unwrap().as_ref()
        );
        println!("Cluster:            {}", opt.cluster);
        println!();
    }

    let shared_config = aws_config::defaults(BehaviorVersion::v2024_03_28())
        .region(region_provider)
        .load()
        .await;
    let client = aws_sdk_ecs::Client::new(&shared_config);

    get_fargate_connection_string(
        &client,
        &opt.cluster,
        &opt.container_name,
        &opt.service_name,
        &opt.region,
    )
    .await
}
