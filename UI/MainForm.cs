// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only

using SessionDeck.Models;
using SessionDeck.Services;

namespace SessionDeck.UI;

public sealed class MainForm : Form
{
    private readonly AccountStore _accountStore;
    private readonly BattleNetService _battleNet;
    private readonly DataGridView _grid = new();
    private readonly Label _status = new();
    private readonly Button _switchButton = new();
    private readonly Button _captureButton = new();
    private readonly Button _newLoginButton = new();
    private readonly Button _renameButton = new();
    private readonly Button _removeButton = new();
    private readonly ToolTip _tooltips = new() { AutoPopDelay = 8000, InitialDelay = 350, ReshowDelay = 100 };
    private readonly CancellationTokenSource _shutdown = new();
    private List<AccountRecord> _accounts = [];

    public MainForm(AccountStore accountStore, BattleNetService battleNet)
    {
        _accountStore = accountStore;
        _battleNet = battleNet;
        SetStyle(
            ControlStyles.AllPaintingInWmPaint |
            ControlStyles.OptimizedDoubleBuffer |
            ControlStyles.ResizeRedraw,
            true);
        BuildInterface();
        LoadAccounts();
    }

    protected override CreateParams CreateParams
    {
        get
        {
            const int wsExComposited = 0x02000000;
            var parameters = base.CreateParams;
            parameters.ExStyle |= wsExComposited;
            return parameters;
        }
    }

    private void BuildInterface()
    {
        Text = "SessionDeck";
        ClientSize = new Size(700, 430);
        MinimumSize = new Size(660, 420);
        StartPosition = FormStartPosition.CenterScreen;
        Font = new Font("Segoe UI", 9F);
        AutoScaleMode = AutoScaleMode.Dpi;

        var layout = new TableLayoutPanel
        {
            Dock = DockStyle.Fill,
            ColumnCount = 1,
            RowCount = 5,
            Padding = new Padding(20, 16, 20, 14)
        };
        layout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100F));
        layout.RowStyles.Add(new RowStyle(SizeType.AutoSize));
        layout.RowStyles.Add(new RowStyle(SizeType.AutoSize));
        layout.RowStyles.Add(new RowStyle(SizeType.Percent, 100F));
        layout.RowStyles.Add(new RowStyle(SizeType.AutoSize));
        layout.RowStyles.Add(new RowStyle(SizeType.AutoSize));

        var title = new Label
        {
            Text = "Battle.net accounts",
            Font = new Font("Segoe UI Semibold", 16F),
            AutoSize = true,
            Margin = new Padding(0)
        };
        var subtitle = new Label
        {
            Text = "Battle.net keeps the login session. This app only saves which account to open.",
            ForeColor = SystemColors.GrayText,
            AutoSize = true,
            Margin = new Padding(0, 0, 0, 10)
        };

        _grid.Dock = DockStyle.Fill;
        _grid.Margin = new Padding(0, 0, 0, 10);
        _grid.AllowUserToAddRows = false;
        _grid.AllowUserToDeleteRows = false;
        _grid.AllowUserToResizeRows = false;
        _grid.AutoGenerateColumns = false;
        _grid.AutoSizeColumnsMode = DataGridViewAutoSizeColumnsMode.Fill;
        _grid.BackgroundColor = SystemColors.Window;
        _grid.BorderStyle = BorderStyle.FixedSingle;
        _grid.MultiSelect = false;
        _grid.ReadOnly = true;
        _grid.RowHeadersVisible = false;
        _grid.SelectionMode = DataGridViewSelectionMode.FullRowSelect;
        _grid.Columns.Add(new DataGridViewTextBoxColumn
        {
            DataPropertyName = nameof(AccountRecord.DisplayName),
            HeaderText = "Name",
            FillWeight = 35
        });
        _grid.Columns.Add(new DataGridViewTextBoxColumn
        {
            DataPropertyName = nameof(AccountRecord.AccountIdentifier),
            HeaderText = "Battle.net account",
            FillWeight = 48
        });
        _grid.Columns.Add(new DataGridViewTextBoxColumn
        {
            Name = "LastUsed",
            HeaderText = "Last used",
            FillWeight = 22
        });
        _grid.SelectionChanged += (_, _) => UpdateButtons();
        _grid.CellDoubleClick += async (_, eventArgs) =>
        {
            if (eventArgs.RowIndex >= 0)
            {
                await SwitchSelectedAsync();
            }
        };

        ConfigureButton(_captureButton, "Save current");
        ConfigureButton(_newLoginButton, "Add account...");
        ConfigureButton(_renameButton, "Rename");
        ConfigureButton(_removeButton, "Remove");
        ConfigureButton(_switchButton, "Switch");
        _switchButton.MinimumSize = new Size(110, 34);

        _captureButton.Click += (_, _) => CaptureCurrent();
        _newLoginButton.Click += async (_, _) => await StartNewSignInAsync();
        _renameButton.Click += (_, _) => RenameSelected();
        _removeButton.Click += (_, _) => RemoveSelected();
        _switchButton.Click += async (_, _) => await SwitchSelectedAsync();

        _tooltips.SetToolTip(_captureButton, "Save the account currently selected in Battle.net.");
        _tooltips.SetToolTip(_newLoginButton, "Open Battle.net's sign-in screen so you can add another account.");
        _tooltips.SetToolTip(_renameButton, "Change the selected account's display name in this app.");
        _tooltips.SetToolTip(_removeButton, "Remove the selected account from this app only.");
        _tooltips.SetToolTip(_switchButton, "Restart Battle.net using the selected saved session.");

        var secondaryButtons = new FlowLayoutPanel
        {
            AutoSize = true,
            AutoSizeMode = AutoSizeMode.GrowAndShrink,
            Dock = DockStyle.Fill,
            FlowDirection = FlowDirection.LeftToRight,
            Margin = new Padding(0),
            WrapContents = true
        };
        secondaryButtons.Controls.AddRange([_captureButton, _newLoginButton, _renameButton, _removeButton]);

        var toolbar = new TableLayoutPanel
        {
            AutoSize = true,
            AutoSizeMode = AutoSizeMode.GrowAndShrink,
            ColumnCount = 2,
            Dock = DockStyle.Fill,
            Margin = new Padding(0, 0, 0, 10)
        };
        toolbar.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100F));
        toolbar.ColumnStyles.Add(new ColumnStyle(SizeType.AutoSize));
        toolbar.Controls.Add(secondaryButtons, 0, 0);
        toolbar.Controls.Add(_switchButton, 1, 0);

        _status.Dock = DockStyle.Fill;
        _status.ForeColor = SystemColors.GrayText;
        _status.AutoEllipsis = true;
        _status.AutoSize = true;
        _status.Margin = new Padding(0);

        layout.Controls.Add(title, 0, 0);
        layout.Controls.Add(subtitle, 0, 1);
        layout.Controls.Add(_grid, 0, 2);
        layout.Controls.Add(toolbar, 0, 3);
        layout.Controls.Add(_status, 0, 4);
        Controls.Add(layout);
        FormClosing += (_, _) => _shutdown.Cancel();
    }

    private static void ConfigureButton(Button button, string text)
    {
        button.Text = text;
        button.AutoSize = true;
        button.AutoSizeMode = AutoSizeMode.GrowAndShrink;
        button.MinimumSize = new Size(86, 34);
        button.Padding = new Padding(8, 0, 8, 0);
        button.Margin = new Padding(0, 0, 6, 0);
    }

    private void LoadAccounts()
    {
        _accounts = _accountStore.Load().ToList();
        var changed = false;
        foreach (var remembered in _battleNet.GetRememberedAccounts())
        {
            if (_accounts.Any(account =>
                    account.AccountIdentifier.Equals(remembered, StringComparison.OrdinalIgnoreCase)))
            {
                continue;
            }

            _accounts.Add(new AccountRecord
            {
                AccountIdentifier = remembered,
                DisplayName = SuggestName(remembered)
            });
            changed = true;
        }

        if (changed)
        {
            _accountStore.Save(_accounts);
        }

        RefreshGrid();
        _status.Text = !_battleNet.ConfigExists
            ? $"Battle.net config not found: {_battleNet.ConfigPath}"
            : !_battleNet.LauncherExists
                ? $"Battle.net launcher not found: {_battleNet.LauncherPath}"
                : "Ready.";
    }

    private void RefreshGrid(Guid? selectId = null)
    {
        var rows = _accounts
            .OrderByDescending(account => account.LastUsedAt)
            .ThenBy(account => account.DisplayName, StringComparer.CurrentCultureIgnoreCase)
            .ToList();
        _grid.Rows.Clear();
        foreach (var account in rows)
        {
            var lastUsed = account.LastUsedAt?.LocalDateTime.ToString("g") ?? "Never";
            var rowIndex = _grid.Rows.Add(account.DisplayName, account.AccountIdentifier, lastUsed);
            _grid.Rows[rowIndex].Tag = account;
            if (selectId == account.Id)
            {
                _grid.Rows[rowIndex].Selected = true;
                _grid.CurrentCell = _grid.Rows[rowIndex].Cells[0];
            }
        }

        if (_grid.Rows.Count > 0 && _grid.SelectedRows.Count == 0)
        {
            _grid.Rows[0].Selected = true;
        }

        UpdateButtons();
    }

    private AccountRecord? SelectedAccount =>
        _grid.SelectedRows.Count == 1 ? _grid.SelectedRows[0].Tag as AccountRecord : null;

    private void UpdateButtons()
    {
        var hasSelection = SelectedAccount is not null;
        _switchButton.Enabled = hasSelection && _battleNet.ConfigExists && _battleNet.LauncherExists;
        _renameButton.Enabled = hasSelection;
        _removeButton.Enabled = hasSelection;
        _captureButton.Enabled = _battleNet.ConfigExists;
        _newLoginButton.Enabled = _battleNet.ConfigExists && _battleNet.LauncherExists;
    }

    private void CaptureCurrent()
    {
        try
        {
            var identifier = _battleNet.GetCurrentAccount();
            if (string.IsNullOrWhiteSpace(identifier))
            {
                MessageBox.Show(
                    this,
                    "No current account is recorded yet. Finish signing into Battle.net, close it, then try Capture current again.",
                    "Capture current account",
                    MessageBoxButtons.OK,
                    MessageBoxIcon.Information);
                return;
            }

            var existing = _accounts.FirstOrDefault(account =>
                account.AccountIdentifier.Equals(identifier, StringComparison.OrdinalIgnoreCase));
            using var dialog = new NameDialog(
                "Capture current account",
                existing?.DisplayName ?? SuggestName(identifier),
                identifier);
            if (dialog.ShowDialog(this) != DialogResult.OK)
            {
                return;
            }

            if (existing is null)
            {
                existing = new AccountRecord { AccountIdentifier = identifier };
                _accounts.Add(existing);
            }

            existing.DisplayName = dialog.DisplayName;
            _accountStore.Save(_accounts);
            RefreshGrid(existing.Id);
            _status.Text = $"Captured {existing.DisplayName}.";
        }
        catch (Exception exception)
        {
            ShowError("Could not capture the current account", exception);
        }
    }

    private async Task StartNewSignInAsync()
    {
        var answer = MessageBox.Show(
            this,
            "Battle.net will close and reopen at its sign-in screen. Sign into the other account with Stay logged in enabled, then return here and click Capture current.\n\nContinue?",
            "Sign in to another account",
            MessageBoxButtons.OKCancel,
            MessageBoxIcon.Information);
        if (answer != DialogResult.OK)
        {
            return;
        }

        await RunBusyAsync("Opening Battle.net sign-in...", async () =>
        {
            await _battleNet.StartNewSignInAsync(_shutdown.Token);
            _status.Text = "Sign in to Battle.net, then click Capture current.";
        });
    }

    private async Task SwitchSelectedAsync()
    {
        var account = SelectedAccount;
        if (account is null)
        {
            return;
        }

        await RunBusyAsync($"Switching to {account.DisplayName}...", async () =>
        {
            await _battleNet.SwitchAsync(account, _shutdown.Token);
            account.LastUsedAt = DateTimeOffset.UtcNow;
            _accountStore.Save(_accounts);
            RefreshGrid(account.Id);
            _status.Text = $"Battle.net started as {account.DisplayName}.";
        });
    }

    private void RenameSelected()
    {
        var account = SelectedAccount;
        if (account is null)
        {
            return;
        }

        using var dialog = new NameDialog("Rename account", account.DisplayName, account.AccountIdentifier);
        if (dialog.ShowDialog(this) != DialogResult.OK)
        {
            return;
        }

        account.DisplayName = dialog.DisplayName;
        _accountStore.Save(_accounts);
        RefreshGrid(account.Id);
    }

    private void RemoveSelected()
    {
        var account = SelectedAccount;
        if (account is null)
        {
            return;
        }

        var answer = MessageBox.Show(
            this,
            $"Remove {account.DisplayName} from this switcher?\n\nThis does not change the Battle.net account or its login session.",
            "Remove account",
            MessageBoxButtons.YesNo,
            MessageBoxIcon.Question);
        if (answer != DialogResult.Yes)
        {
            return;
        }

        _accounts.Remove(account);
        _accountStore.Save(_accounts);
        RefreshGrid();
    }

    private async Task RunBusyAsync(string message, Func<Task> action)
    {
        SetBusy(true);
        _status.Text = message;
        try
        {
            await action();
        }
        catch (OperationCanceledException) when (_shutdown.IsCancellationRequested) { }
        catch (Exception exception)
        {
            ShowError(message.TrimEnd('.'), exception);
        }
        finally
        {
            SetBusy(false);
        }
    }

    private void SetBusy(bool busy)
    {
        _grid.Enabled = !busy;
        _captureButton.Enabled = !busy;
        _newLoginButton.Enabled = !busy;
        _renameButton.Enabled = !busy;
        _removeButton.Enabled = !busy;
        _switchButton.Enabled = !busy;
        UseWaitCursor = busy;
        if (!busy)
        {
            UpdateButtons();
        }
    }

    private void ShowError(string action, Exception exception)
    {
        _status.Text = $"{action}: {exception.Message}";
        MessageBox.Show(
            this,
            $"{action}.\n\n{exception.Message}",
            "SessionDeck",
            MessageBoxButtons.OK,
            MessageBoxIcon.Error);
    }

    private static string SuggestName(string identifier)
    {
        var separator = identifier.IndexOf('@');
        return separator > 0 ? identifier[..separator] : identifier;
    }
}
