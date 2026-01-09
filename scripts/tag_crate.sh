#!/usr/bin/env bash
set -euxo pipefail

crate=$1
version=$2

git config user.name "CI"
git config user.email "ci@example.com"

git tag "${crate}-v${version}"
git push origin "${crate}-v${version}"
