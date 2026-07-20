// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only

namespace SessionDeck.Models;

public sealed class AccountRecord
{
    public Guid Id { get; set; } = Guid.NewGuid();
    public string DisplayName { get; set; } = string.Empty;
    public string AccountIdentifier { get; set; } = string.Empty;
    public DateTimeOffset CreatedAt { get; set; } = DateTimeOffset.UtcNow;
    public DateTimeOffset? LastUsedAt { get; set; }
}
