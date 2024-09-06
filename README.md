# ECS session manager helper

Simple program to output the proper `awscli` command to connect to a running ECS task.

## Usage
```shell
ecs-jumper -c cluster-name -n container-name -s service-name
```
Output:

```
Container Image: 123123123.dkr.ecr.us-west-2.amazonaws.com/example:7a80faa
Running image Tag: 7a80faa


aws ecs execute-command \
                --region us-west-2 \
                --cluster example-cluster \
                --task arn:aws:ecs:us-west-2:123123123:task/container-name/edafda3154507abc1dfff8e33e220 \
                --container container-name \
                --command "/bin/bash" \
                --interactive
```
