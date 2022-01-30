MAC Address lookup tool
=========================
![Crates.io](https://img.shields.io/crates/v/mac_oui)
[![Documentation](https://docs.rs/mac_oui/badge.svg)](https://docs.rs/mac_oui)
[![Build Status](https://travis-ci.com/marirs/mac-oui.svg?branch=master)](https://travis-ci.com/marirs/mac-oui)

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
mac_oui = "0.4.0"
```

If you want to use it with the inbuilt oui db; then:
```toml
[dependencies]
mac_oui = { version = "0.4.0", features = ["with-db"] }
```

and then

```rust
use mac_oui::Oui;

fn main () {
    let oui_db = Oui::default();
    assert!(oui_db.is_ok());
}
```

## Running the Example
You can run the default example that is included in the following manner.
- `cargo run --example mac_lookup <mac address>` eg:
```bash
cargo run --example mac_lookup --features="with-db" 70:B3:D5:e7:4f:81
   Compiling mac_oui v0.1.0 (/Users/sgp/Documents/DEV/Repos/macaddr-rs)
    Finished dev [optimized + debuginfo] target(s) in 1.54s
     Running `target/debug/examples/mac_lookup '70:B3:D5:e7:4f:81'`
Entry {
    oui: "70:B3:D5",
    is_private: false,
    company_name: "Ieee Registration Authority",
    company_address: "445 Hoes Lane Piscataway NJ 08554 US",
    country_code: "US",
    assignment_block_size: "MA-L",
    date_created: "2014-01-12",
    date_updated: "2016-04-27",
}
```
- Example of lookup by Manufacturer
```bash
$ cargo run --example manufacturer_lookup --features="with-db" "Apple, Inc"
    Finished dev [optimized + debuginfo] target(s) in 0.02s
     Running `target/debug/examples/manufacturer_lookup 'Apple, Inc'`
[
    Entry {
        oui: "...",
        is_private: false,
        company_name: "Apple, Inc",
        company_address: "1 Infinite Loop Cupertino CA 95014 US",
        country_code: "US",
        assignment_block_size: "MA-L",
        date_created: "2017-02-21",
        date_updated: "2017-02-21",
    },
    <clip>....
]
```

- Example Getting a list of Manufacturers
```bash
$ cargo run --example --features="with-db" db_stats
    Finished dev [optimized + debuginfo] target(s) in 0.06s
     Running `target/debug/examples/db_stats`
Total Records= 41917
Total Manufacturers= 27379
Total MAC Addrs= 41917

====Manufacturers====
[
    "\"Azimut\" Production Association Jsc",
    "\"Continent\" Co Ltd",
    "\"Meta-chrom\" Co Ltd",
    "\"Rpc \"Energoautomatika\" Ltd",
    "(UN)Manned",
    "+plugg srl",
    "01db-Metravib",
    "1.A Connect GmbH",
    "1000eyes GmbH",
    "100fio networks Tech Llc",
    "10net Communications/Dca",
    "11wave Technonlogy Co, Ltd",
    "12Sided Tech, Llc",
    "1394 Printer Working Group",
    "1394 Trade Association",
    "16063",
    "1Net Corp",
    "1Verge Internet Tech (Beijing) Co, Ltd",
    "1more",
    "2 France Marine",
]
...
[
    "杭州德澜科技有限公司（HangZhou Delan Tech Co, Ltd）",
    "\u{200b}Asung Techno Co, Ltd",
    "éolane",
    "Östling Marking Systems GmbH",
    "Öresundskraft AB",
    "Åmic AB",
    "µTech Tecnologia Ltda",
    "«Intellect module» Llc",
    "zte Corp",
    "zhejiang yuanwang communication technolgy Co, Ltd",
    "zhejiang ebang communication Co, Ltd",
    "z-max mediasolution",
    "yLez Tech Pte Ltd",
    "xxter b.v.",
    "xvtec Ltd",
    "xn systems",
    "xmi systems",
    "xm",
    "xiamenshi c-chip Tech Co, Ltd",
    "xTom GmbH",
]
```
---
