#!/usr/bin/env bash

set -e

BUILD_DIR=dist/
DEPLOY_BRANCH=gh-pages

set -x

rm -rf $BUILD_DIR
git worktree prune # If the old worktree wasn't deleted, this will clean it up
git worktree add $BUILD_DIR $DEPLOY_BRANCH
# TODO clean up old files before building
npm run build
pushd $BUILD_DIR
git add .
# `git commit` returns exit=1 for no changes, so only commit if there's a diff
git diff --staged --quiet || git commit -m 'Deploy GitHub Pages'
git push
popd
git worktree remove $BUILD_DIR
