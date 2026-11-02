// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set("ProductName", "SessionDeck")
            .set("FileDescription", "Battle.net session switcher")
            .set_manifest_file("app.manifest")
            .compile()
            .expect("compile Windows resources");
    }
}
