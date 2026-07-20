namespace BNetSwitcher.UI;

public sealed class NameDialog : Form
{
    private readonly TextBox _nameBox = new();

    public NameDialog(string title, string initialName, string accountIdentifier)
    {
        Text = title;
        ClientSize = new Size(420, 166);
        FormBorderStyle = FormBorderStyle.FixedDialog;
        MaximizeBox = false;
        MinimizeBox = false;
        StartPosition = FormStartPosition.CenterParent;
        Font = new Font("Segoe UI", 9F);

        var accountLabel = new Label
        {
            Text = accountIdentifier,
            Location = new Point(16, 16),
            Size = new Size(388, 22),
            AutoEllipsis = true
        };
        var nameLabel = new Label
        {
            Text = "Display name",
            Location = new Point(16, 50),
            AutoSize = true
        };
        _nameBox.Location = new Point(16, 73);
        _nameBox.Size = new Size(388, 23);
        _nameBox.Text = initialName;

        var saveButton = new Button
        {
            Text = "Save",
            DialogResult = DialogResult.OK,
            Location = new Point(248, 116),
            Size = new Size(75, 30)
        };
        var cancelButton = new Button
        {
            Text = "Cancel",
            DialogResult = DialogResult.Cancel,
            Location = new Point(329, 116),
            Size = new Size(75, 30)
        };

        Controls.AddRange([accountLabel, nameLabel, _nameBox, saveButton, cancelButton]);
        AcceptButton = saveButton;
        CancelButton = cancelButton;
        Shown += (_, _) =>
        {
            _nameBox.Focus();
            _nameBox.SelectAll();
        };
    }

    public string DisplayName => _nameBox.Text.Trim();

    protected override void OnFormClosing(FormClosingEventArgs e)
    {
        if (DialogResult == DialogResult.OK && string.IsNullOrWhiteSpace(DisplayName))
        {
            MessageBox.Show(this, "Enter a display name.", Text, MessageBoxButtons.OK, MessageBoxIcon.Information);
            e.Cancel = true;
        }

        base.OnFormClosing(e);
    }
}
