#!/usr/bin/env bash

set -Eeuo pipefail

build_docker_image() {
  local version="${VERSION:-snapshot}"
  run_command "docker build -t rafaelfiume/rustic-sketch:$version $project_dir"
}

exit_if_version_is_undefined() {
  if [ -z "${VERSION:-}" ]; then
    exit_with_error "Fail to tag image: \$VERSION is undefined or empty."
  fi
}

main() {
  local script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
  local tools_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
  local project_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
  local environments_dir="$tools_dir/environments"
  local utils_dir="$tools_dir/utilities"
  source "$utils_dir/logs.sh"
  source "$utils_dir/std.sh"
  source "$utils_dir/env-vars-loader.sh"
  source "$utils_dir/docker.sh"

  info "Building docker image..."
  exit_if_docker_is_not_installed
  build_docker_image
  info "Done building docker image."
  
  info "Tagging docker image..."

  exit_if_version_is_undefined

  docker tag rafaelfiume/rustic-sketch:$VERSION rafaelfiume/rustic-sketch:latest
  if [ "${CIRCLE_BRANCH:-}" = "main" ]; then
    run_command "docker tag rafaelfiume/rustic-sketch:$VERSION rafaelfiume/rustic-sketch:stable"
  else
    info "Skipping tagging 'stable' (branch != 'main')."
  fi

  info "Tagging docker image completed."
}

main
