// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only
const assert = require('node:assert/strict');
const {classify, detect} = require('./changes.cjs');
const files = (...paths) => paths.map(filename => ({filename}));
const scope = (...paths) => classify(files(...paths));

assert.deepEqual(scope('README.md', 'docs/images/example.png'), {build: false, dependencies: false, languages: []});
assert.deepEqual(scope('release-please-config.json', '.release-please-manifest.json'), {build: false, dependencies: false, languages: []});
assert.deepEqual(scope('Services/storage.rs'), {build: true, dependencies: false, languages: ['rust']});
assert.deepEqual(scope('Cargo.lock'), {build: true, dependencies: true, languages: ['rust']});
assert.equal(scope('.cargo/config.toml').dependencies, true);
assert.equal(scope('rust-toolchain.toml').build, true);
assert.equal(scope('assets/icon.ico').build, true);
assert.equal(scope('build.ps1').build, true);
assert.equal(scope('.github/workflows/ci.yml').build, true);
assert.deepEqual(scope('scripts/check_privacy.py').languages, ['python']);
assert.deepEqual(scope('.github/workflows/security.yml'), {build: false, dependencies: true, languages: ['actions']});
assert.deepEqual(scope('.github/scripts/changes.cjs').languages, ['actions', 'python', 'rust']);
assert.equal(scope('.github/scripts/changes.cjs').build, true);
assert.equal(scope('.github/scripts/changes.cjs').dependencies, true);
assert.equal(classify([{filename: 'docs/old.txt', previous_filename: 'Services/old.rs'}]).build, true);
assert.equal(classify([{filename: 'docs/old.txt', previous_filename: 'Cargo.lock'}]).dependencies, true);
assert.deepEqual(scope('Services/AccountStore.cs').languages, []);

(async () => {
  const context = {eventName: 'pull_request', repo: {}, issue: {number: 1}};
  const github = {rest: {pulls: {}, repos: {}}, paginate: async () => files('README.md')};
  assert.equal((await detect({context, github})).build, false);
  github.paginate = async () => Array(3000).fill({filename: 'README.md'});
  assert.equal((await detect({context, github})).build, true);
  github.paginate = async () => [];
  assert.equal((await detect({context, github})).dependencies, true);
  github.paginate = async () => {throw new Error('API unavailable');};
  await assert.rejects(() => detect({context, github}), /API unavailable/);
  context.eventName = 'push';
  context.payload = {before: 'a'.repeat(40)};
  context.sha = 'b'.repeat(40);
  github.rest.repos.compareCommitsWithBasehead = async () => ({data: {files: files('README.md')}});
  assert.deepEqual((await detect({context, github})).languages, []);
  github.rest.repos.compareCommitsWithBasehead = async () => ({data: {files: Array(300).fill({filename: 'README.md'})}});
  assert.equal((await detect({context, github})).build, true);
  context.payload.before = '0'.repeat(40);
  assert.equal((await detect({context, github})).build, true);
  for (const eventName of ['schedule', 'workflow_dispatch', 'unknown']) {
    assert.deepEqual((await detect({context: {eventName}, github})).languages, ['actions', 'python', 'rust']);
  }
  console.log('Changed-path selection tests passed');
})().catch(error => {console.error(error); process.exitCode = 1;});
