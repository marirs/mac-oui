MAC Address lookup tool
=========================

Lookup the MAC Address for a corresponding details (eg: org, created, etc...)

## Requirements

- Rust

## Compile
- Dev
```bash
cargo b
```
- Release
```bash 
cargo b --release
```

## Usage

You can include this in your Cargo.toml file:
```toml
[dependencies]
mac_oui = "0.1.0"
```

## Running the Example
You can run the default example that is included in the following manner.
- `cargo run --example mac_lookup <mac address>` eg:
```bash
cargo run --example mac_lookup 70:B3:D5:e7:4f:81
```

---