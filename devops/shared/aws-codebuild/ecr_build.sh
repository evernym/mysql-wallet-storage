#!/bin/bash -ex

# Expected environment variableis
#   - AWS_REGION - AWS region
#   - AWS_ACCOUNT_ID - AWS account id
#   - AWS_ECR_REPO_NAME - AWS ECR repository name

usage() {
    echo "$0 <aws-ecr-image-tags> <local-image-name> <shell-command>"
}

for envv in AWS_REGION AWS_ACCOUNT_ID AWS_ECR_REPO_NAME; do
    if [[ -z "${!envv}" ]]; then
        echo "$envv env variable is not defined" >&2 
        exit 1
    fi
done

if [[ $# -ne 3 ]]; then
    echo "$0: Incorrect number of required arguments specified (expected 6, provided $#)." >&2
    echo "Try '$0 --help' for more information ..."
    exit 1
fi

aws_ecr_image_tags=$1
local_image=$2
build_cmd=$3

aws_ecr_host="$AWS_ACCOUNT_ID.dkr.ecr.$AWS_REGION.amazonaws.com"
aws_ecr_image_name="$aws_ecr_host/$AWS_ECR_REPO_NAME"

echo "Building docker image '$local_image' ..."
eval "$build_cmd"
docker images "$local_image"

for tag in $aws_ecr_image_tags; do
    echo "Tagging the image as '$aws_ecr_image_name:$tag' ..."
    docker tag "$local_image" "$aws_ecr_image_name:$tag"
done
docker images "$aws_ecr_image_name"

for tag in $aws_ecr_image_tags; do
    echo "Pushing an image '$aws_ecr_image_name:$tag' ..."
    docker push "$aws_ecr_image_name:$tag"
done
