using BNetSwitcher.Models;
using BNetSwitcher.Services;

namespace BNetSwitcher.UI;

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
    private readonly CancellationTokenSource _shutdown = new();
    private List<AccountRecord> _accounts = [];

    public MainForm(AccountStore accountStore, BattleNetService battleNet)
    {
        _accountStore = accountStore;
        _battleNet = battleNet;
        BuildInterface();
        LoadAccounts();
    }

    private void BuildInterface()
    {
        Text = "BNet Switcher";
        ClientSize = new Size(680, 410);
        MinimumSize = new Size(620, 400);
        StartPosition = FormStartPosition.CenterScreen;
        Font = new Font("Segoe UI", 9F);

        var title = new Label
        {
            Text = "Battle.net accounts",
            Font = new Font("Segoe UI Semibold", 16F),
            Location = new Point(18, 15),
            AutoSize = true
        };
        var subtitle = new Label
        {
            Text = "Battle.net keeps the login session. This app only saves which account to open.",
            ForeColor = SystemColors.GrayText,
            Location = new Point(20, 48),
            AutoSize = true
        };

        _grid.Location = new Point(20, 78);
        _grid.Size = new Size(640, 226);
        _grid.Anchor = AnchorStyles.Top | AnchorStyles.Bottom | AnchorStyles.Left | AnchorStyles.Right;
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

        ConfigureButton(_captureButton, "Capture current", new Point(20, 319), 112);
        ConfigureButton(_newLoginButton, "Sign in new...", new Point(138, 319), 104);
        ConfigureButton(_renameButton, "Rename", new Point(248, 319), 82);
        ConfigureButton(_removeButton, "Remove", new Point(336, 319), 82);
        ConfigureButton(_switchButton, "Switch account", new Point(526, 319), 134);
        _switchButton.Anchor = AnchorStyles.Bottom | AnchorStyles.Right;
        _captureButton.Anchor = AnchorStyles.Bottom | AnchorStyles.Left;
        _newLoginButton.Anchor = AnchorStyles.Bottom | AnchorStyles.Left;
        _renameButton.Anchor = AnchorStyles.Bottom | AnchorStyles.Left;
        _removeButton.Anchor = AnchorStyles.Bottom | AnchorStyles.Left;

        _captureButton.Click += (_, _) => CaptureCurrent();
        _newLoginButton.Click += async (_, _) => await StartNewSignInAsync();
        _renameButton.Click += (_, _) => RenameSelected();
        _removeButton.Click += (_, _) => RemoveSelected();
        _switchButton.Click += async (_, _) => await SwitchSelectedAsync();

        _status.Location = new Point(20, 367);
        _status.Size = new Size(640, 24);
        _status.Anchor = AnchorStyles.Bottom | AnchorStyles.Left | AnchorStyles.Right;
        _status.ForeColor = SystemColors.GrayText;
        _status.AutoEllipsis = true;

        Controls.AddRange([
            title, subtitle, _grid, _captureButton, _newLoginButton,
            _renameButton, _removeButton, _switchButton, _status
        ]);
        FormClosing += (_, _) => _shutdown.Cancel();
    }

    private static void ConfigureButton(Button button, string text, Point location, int width)
    {
        button.Text = text;
        button.Location = location;
        button.Size = new Size(width, 32);
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
            "BNet Switcher",
            MessageBoxButtons.OK,
            MessageBoxIcon.Error);
    }

    private static string SuggestName(string identifier)
    {
        var separator = identifier.IndexOf('@');
        return separator > 0 ? identifier[..separator] : identifier;
    }
}
