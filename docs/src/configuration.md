# Configuration

## Config File Location

The configuration file is stored at `~/.templar/config.toml`. Override with the `$TEMPLAR_CONFIG` environment variable.

## Interactive Setup

```bash
templar config init
```

This creates a default configuration with mainnet and testnet profiles.

## View Configuration

```bash
templar config show
```

## Configuration Options

```toml
active_profile = "mainnet"

[profiles.mainnet]
near_rpc_url = "https://rpc.mainnet.fastnear.com"
backend_url = "https://api.templarfi.org"
relayer_v0_url = "https://relayer.templarfi.org"
relayer_v1_url = "https://relayer.templarfi.org:4001"
near_network_id = "mainnet"
near_chain_id = 397
registry_contract_ids = ["v1.tmplr.near"]
hermes_url = "https://hermes.pyth.network"
bridge_rpc_url = "https://bridge.chaindefuser.com/rpc"
solver_relayer_url = "https://solver-relay.chaindefuser.com/rpc"
hot_bridge_contract = "v2_1.omni.hot.tg"
intents_contract = "intents.near"

[theme]
banner = true
color = "auto"
animations = true
unicode = true
voice = "cypherpunk"
```

## Profiles

Switch between profiles using `--profile`:

```bash
templar --profile testnet markets list
```

## Global Flags

| Flag | Description |
|------|-------------|
| `--quiet`, `-q` | Suppress banner and non-essential output |
| `--color <MODE>` | Color mode: auto, always, never |
| `--output <FORMAT>` | Output format: table, json |
| `--no-banner` | Suppress the startup banner |
| `--no-animation` | Disable animated output |
| `--profile <NAME>` | Configuration profile to use |
| `--rpc-url <URL>` | Override NEAR RPC URL |

## Security

- Config files are created with mode `0o600` (owner read/write only)
- The `~/.templar/` directory is created with mode `0o700`
- Credentials are stored separately in `~/.templar/keys/`
