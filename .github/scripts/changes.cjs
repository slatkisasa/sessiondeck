// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only
const all = () => ({build: true, dependencies: true, languages: ['actions', 'python', 'rust']});

function classify(files) {
  const result = {build: false, dependencies: false, languages: []};
  const languages = new Set();
  for (const file of files) {
    for (const path of [file.filename, file.previous_filename].filter(Boolean)) {
      const detector = path === '.github/workflows/changes.yml' || path.startsWith('.github/scripts/');
      const cargo = /(^|\/)Cargo\.(toml|lock)$/.test(path) || path.startsWith('.cargo/') || /^rust-toolchain(\.toml)?$/.test(path);
      const rust = path.endsWith('.rs') || cargo;
      const packaging = path === 'build.ps1' || path === 'app.manifest' || /^(assets|resources)\//.test(path);
      if (rust || packaging || detector || ['.github/workflows/ci.yml', '.github/workflows/release.yml'].includes(path)) result.build = true;
      if (cargo || detector || path === '.github/workflows/security.yml') result.dependencies = true;
      if (rust || packaging || detector || path === '.github/workflows/codeql.yml') languages.add('rust');
      if (path.endsWith('.py') || /(^|\/)(requirements[^/]*\.txt|pyproject\.toml|poetry\.lock|uv\.lock|Pipfile(\.lock)?)$/.test(path) || detector || path === '.github/workflows/codeql.yml') languages.add('python');
      if (/^\.github\/(workflows|actions)\//.test(path) || detector) languages.add('actions');
    }
  }
  result.languages = [...languages].sort();
  return result;
}

async function detect({github, context}) {
  if (context.eventName === 'pull_request') {
    const files = await github.paginate(github.rest.pulls.listFiles, {
      ...context.repo, pull_number: context.issue.number, per_page: 100,
    });
    // The PR API caps its response at 3,000 files. An empty list is unexpected.
    return files.length === 0 || files.length >= 3000 ? all() : classify(files);
  }
  if (context.eventName === 'push' && context.payload.before && !/^0+$/.test(context.payload.before)) {
    const {data} = await github.rest.repos.compareCommitsWithBasehead({
      ...context.repo, basehead: `${context.payload.before}...${context.sha}`,
    });
    // Compare responses contain at most 300 changed files, even with pagination.
    return !data.files?.length || data.files.length >= 300 ? all() : classify(data.files);
  }
  // Scheduled/manual runs, new branches, and unfamiliar events run full checks.
  return all();
}

module.exports = {classify, detect};
