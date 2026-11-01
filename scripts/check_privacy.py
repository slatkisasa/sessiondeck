#!/usr/bin/env python3
# Copyright (C) 2026 slatkisasa
# SPDX-License-Identifier: AGPL-3.0-only
"""Scan Git content without printing matched personal data."""
import argparse
import re
import subprocess
from pathlib import PurePosixPath

EMAIL = re.compile(rb"[A-Za-z0-9._%+\-]+@([A-Za-z0-9.-]+\.[A-Za-z]{2,})")
USER_PATH = re.compile(rb"(?:/Users/|/home/|[A-Za-z]:[\\/]Users[\\/])[A-Za-z0-9_.-]+")
ALLOWED_DOMAINS = {b"example.invalid", b"example.com", b"example.org", b"example.net", b"users.noreply.github.com"}


def git(*args):
    return subprocess.check_output(["git", *args])


def findings(path, data):
    name = PurePosixPath(path).name.lower()
    if (name == "accounts.json" or name.startswith("accounts.json.")
            or name == "battle.net.config" or name.startswith("battle.net.config.")
            or "backups" in [p.lower() for p in PurePosixPath(path).parts]):
        yield "private account/configuration file"
    if b"\0" in data:
        return  # Images and other binaries still require manual review.
    for number, line in enumerate(data.splitlines(), 1):
        if any(m.group(1).lower() not in ALLOWED_DOMAINS for m in EMAIL.finditer(line)):
            yield f"line {number}: non-example email address"
        if USER_PATH.search(line):
            yield f"line {number}: personal filesystem path"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--history", action="store_true", help="scan all blobs reachable from HEAD")
    args = parser.parse_args()
    if args.history:
        entries = []
        for row in git("rev-list", "--objects", "HEAD").splitlines():
            oid, _, path = row.partition(b" ")
            if path and git("cat-file", "-t", oid.decode()).strip() == b"blob":
                entries.append((path.decode("utf-8", "surrogateescape"), oid.decode()))
    else:
        # Read the index, not working files: unstaged edits cannot hide staged data.
        entries = []
        for row in git("ls-files", "--stage", "-z").split(b"\0"):
            if not row:
                continue
            meta, path = row.split(b"\t", 1)
            mode, oid, stage = meta.split()
            if stage != b"0":
                raise RuntimeError("Resolve unmerged files before scanning.")
            if mode != b"160000":
                entries.append((path.decode("utf-8", "surrogateescape"), oid.decode()))
    failed = False
    for path, oid in entries:
        for finding in findings(path, git("cat-file", "blob", oid)):
            print(f"{ascii(path)}: {finding}")
            failed = True
    if failed:
        print("Privacy scan failed. Remove private data; use example.invalid for examples.")
    else:
        print("Privacy scan passed.")
    return int(failed)


if __name__ == "__main__":
    raise SystemExit(main())
