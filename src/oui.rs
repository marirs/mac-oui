use std::{
    fs::read_to_string,
    path::Path,
    collections::BTreeMap,
};
use serde::{Serialize, Deserialize, Deserializer};
use byteorder::{NetworkEndian, ReadBytesExt};
use eui48::MacAddress;
use csv;

type Start = u64;
type End = u64;
type OuiMap = BTreeMap<(Start, End), Entry>;
type Error = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Entry {
    /// Organization Unique Identifier
    pub oui: String,
    /// flag is set to 'true' and companyName, companyAddress and countryCode are 'private'
    #[serde(deserialize_with = "string_to_bool")]
    pub is_private: bool,
    /// Name of the company which registered the MAC addresses block
    pub company_name: String,
    /// Company's full address
    pub company_address: String,
    /// Company's country code in ISO 3166 format
    pub country_code: String,
    /// 'MA-L' for MAC Address Block Large, or
    /// 'MA-M' for MAC Address Block Medium, or
    /// 'MA-S' for MAC Address Block Small, or
    /// 'IAB' for Individual Address Block
    pub assignment_block_size: String,
    /// Date when the range was allocated, in YYYY-MM-DD format
    pub date_created: String,
    /// Date when the range was last updated, in YYYY-MM-DD format
    pub date_updated: String,
}

fn string_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    match s {
        "1" => Ok(true),
        _ => Ok(false),
    }
}

pub struct Oui {
    db: OuiMap
}

impl Oui {
    pub fn default() -> Result<Oui, Error> {
        //! Loads the default oui csv database
        //!
        //! ## Example
        //! ```rust
        //! use mac_oui::Oui;
        //!
        //! fn main() {
        //!     let db = Oui::default();
        //!     assert!(db.is_ok());
        //! }
        //! ```
        if !Path::new("assets/oui.csv").exists() {
            return Err(
                format!("Internal DB not found.")
            )
        }
        let db_text = include_str!("../assets/oui.csv");

        let oui_entry = read_into_db(db_text);
        match oui_entry {
            Ok(e) => { Ok(Oui { db: e }) }
            Err(e) => Err(format!("Error: {}", e))
        }
    }

    pub fn from_csv_file<P: AsRef<Path>>(oui_csv: P) -> Result<Oui, Error> {
        //! Loads a database from the given path.
        //! The default database is loaded from:
        //! `https://macaddress.io/database-download/csv`
        //!
        //! ## Example
        //! ```rust
        //! use mac_oui::Oui;
        //!
        //! fn main () {
        //!     let db = Oui::from_csv_file("assets/oui.csv");
        //!     assert!(db.is_ok())
        //! }
        //! ```
        let db_text = if let Ok(contents) = read_to_string(oui_csv.as_ref()) {
            contents
        } else {
            return Err(
                format!(
                    "could not open database file - {}",
                    oui_csv.as_ref().to_str().unwrap()
                )
            )
        };
        let oui_entry = read_into_db(&db_text);
        match oui_entry {
            Ok(e) => {
                Ok(Oui {
                    db: e
                })
            }
            Err(e) => Err(format!("Error: {}", e))
        }
    }

    pub fn lookup(&self, mac_addr: &str) -> Result<Option<Entry>, Error> {
        //! Lookup for a mac address in the OUI Database and
        //! return an Entry Result.
        //!
        //! ## Example
        //! ```rust
        //! use mac_oui::Oui;
        //!
        //! fn main() {
        //!     let db = Oui::default().unwrap();
        //!     let res = db.lookup("70:B3:D5:27:4f:81");
        //!     assert!(res.is_ok());
        //!
        //!     println!("{:#?}", res.unwrap());
        //! }
        //! ```
        let mac_addr = match MacAddress::parse_str(&mac_addr) {
            Ok(m) => m,
            Err(e) => return Err(e.to_string())
        };
        let mac_u = mac_addr_to_u64(&mac_addr);
        match mac_u {
            Ok(m) => self.query(&m),
            Err(e) => return Err(e.to_string())
        }
    }

    /// Queries the database using a u64 representation from the wrapper query functions
    fn query(&self, query: &u64) -> Result<Option<Entry>, Error> {
       let mut results = Vec::<((u64, u64), Entry)>::new();

        for ((s, e), value) in &self.db {
            if query >= s && query <= e {
                results.push(((*s, *e), value.clone()));
            }
        }

        if results.len() > 2 {
            return Err(format!(
                "more than two oui matches - possible database error",
            ));
        }
        // Get the last value from the search,
        // and return it
        match results.pop() {
            Some(r) => Ok(Some(r.1)),
            _ => Ok(None),
        }
    }
}

fn mac_addr_to_u64(mac: &MacAddress) -> Result<u64, Error> {
    //! Converts a MAC Address to a u64 value
    let mac_bytes = mac.as_bytes();

    let padded = vec![
        0,
        0,
        mac_bytes[0],
        mac_bytes[1],
        mac_bytes[2],
        mac_bytes[3],
        mac_bytes[4],
        mac_bytes[5],
    ];

    let mut padded_mac = &padded[..8];
    let mac_num = if let Ok(padded) = padded_mac.read_u64::<NetworkEndian>() {
        padded
    } else {
        return Err(
            format!(
                "could not read_u64 from padded MAC byte array: {:?}",
                padded_mac
            )
        )
    };
    Ok(mac_num)
}

fn csv_de(csv_text: &str) -> Result<Vec<Entry>, csv::Error> {
    csv::Reader::from_reader(csv_text.as_bytes())
        .deserialize()
        .collect()
}

fn read_into_db(csv_text: &str) -> Result<OuiMap, Error> {
    //! Reads the OUI CSV File into a Btree Map
    let mut mac_oui_db = OuiMap::new();

    let records = match csv_de(csv_text) {
        Ok(r) => r,
        Err(_e) => return Err(
            String::from("CSV file is not matching OUI CSV, \
            be sure to download here: https://macaddress.io/database-download/csv")
        )
    };

    // loop thru
    for record in records {
        // Get the mask if any
        let mask: u8;
        let oui_mask: Vec<_> = record.oui.split('/').collect();
        match oui_mask.len() {
            1 => mask = 24,
            2 => {
                mask = u8::from_str_radix(&oui_mask[1], 10).unwrap();
                if !(mask >= 8 && mask <= 48) {
                    return Err(format!("incorrect mask value: {}", mask));
                }
            }
            _ => {
                return Err(format!(
                    "invalid number of mask separators: {:?}",
                    oui_mask
                ))
            }
        };

        // remove the separators from
        // the mac address string
        let oui = record.oui
            .to_uppercase()
            .replace(":", "")
            .replace("-", "")
            .replace(".", "");
        let oui_int = if let Ok(oi) = u64::from_str_radix(&oui, 16) {
            oi
        } else {
            return Err(
                format!("could not parse OUI info, is this a oui csv file?")
            )
        };
        // If it's a 24-bit mask, shift over as non-24
        let oui_start: u64;
        if mask == 24 {
            oui_start = oui_int << 24;
        } else {
            oui_start = oui_int
        };
        // Find the end of this OUI entry range
        let oui_end: u64 = oui_start | 0xFFFF_FFFF_FFFF >> mask;

        // Add to the database
        let data = Entry {
            oui: record.oui,
            is_private: record.is_private,
            company_name: record.company_name,
            company_address: record.company_address,
            country_code: record.country_code,
            assignment_block_size: record.assignment_block_size,
            date_created: record.date_created,
            date_updated: record.date_updated
        };
        mac_oui_db.insert((oui_start, oui_end), data);
    }

    Ok(mac_oui_db)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_database() {
        let db = Oui::default();
        assert!(db.is_ok());
    }

    #[test]
    fn test_from_file() {
        let db = Oui::from_csv_file("assets/oui.csv");
        assert!(db.is_ok());
    }

    #[test]
    fn test_lookup() {
        let db = Oui::default().unwrap();

        let res = db.lookup("70:B3:D5:e7:4f:81").unwrap();
        assert_eq!(res.unwrap().company_name, "Ieee Registration Authority")
    }
}