# Theme & Branding

The Templar CLI features a cypherpunk-themed terminal experience that matches the [templarfi.org](https://www.templarfi.org/) brand identity.

## Color Palette

| Color | Hex | Usage |
|-------|-----|-------|
| Gold | `#D5AA51` | Headers, command names, key values |
| Antique Gold | `#AE8227` | Borders, separators, progress bars |
| Ivory | `#E8E1D3` | Primary body text |
| Warm Grey | `#A59B89` | Labels, timestamps, muted text |
| Cipher Purple | `#963CDC` | Cryptographic operations |
| Success Green | `#50C759` | Confirmations |
| Danger Red | `#E03535` | Errors, liquidation warnings |
| Info Teal | `#1499B6` | Explorer links |

## Voice Modes

The CLI offers two voice modes:

### Cypherpunk (default)

```
✠ Forging your configuration...
✠ Sealing transaction...
⣾ Communing with the bridge oracle...
✓ Configuration forged at ~/.templar/config.toml
```

### Standard

```
Creating configuration...
Submitting transaction...
Requesting deposit address...
Configuration created at ~/.templar/config.toml
```

Switch with:
```toml
[theme]
voice = "standard"
```

## Disabling Colors

```bash
# Via flag
templar --color never markets list

# Via environment variable
NO_COLOR=1 templar markets list
```

## Disabling Animations

```bash
templar --no-animation supply deposit ...
```

Or in config:
```toml
[theme]
animations = false
```

## ASCII Fallback

For terminals without Unicode support:
```toml
[theme]
unicode = false
```

This replaces box-drawing characters (`┏━┓`) with ASCII equivalents (`+-+`) and `✠` with `*`.

## Banner

The startup banner shows the Templar mark ASCII art on wide terminals (≥ 110 cols) or a compact version on narrow terminals.

Disable with:
```bash
templar --no-banner markets list
```

Or permanently:
```toml
[theme]
banner = false
```
