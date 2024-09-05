# ECS session manager helper

Simple program to output the right `aws` cli command to connect to a running ECS task.

## Usage
```shell
jumper -c cluster-name -n container-name -s service-name
```
Output:

```
aws ecs execute-command  \
        --region us-west-2 \
        --cluster cluster-name \
        --task arn:aws:ecs:us-west-2:11111111:task/container-name/d2621311sbf584a02849709b35f057db0 \
        --container container-name \
        --command "/bin/bash" \
        --interactive
```
