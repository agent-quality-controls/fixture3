# fixture3 install stub

This package reserves the `fixture3` Cargo package name and points users to the binary release channel.

`fixture3` is distributed as a prebuilt CLI through `cargo-binstall`. The package published here is intentionally small and does not contain the real application logic.

Install the command with:

```bash
cargo binstall fixture3
```

After installation, run:

```bash
fixture3 --help
```

The help output is the agent guide for the tool. It explains the fixture manifest, suite workflow, approval flow, status output, diff refresh behavior, and exit codes.
