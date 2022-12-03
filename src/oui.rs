use byteorder::{NetworkEndian, ReadBytesExt};
use macaddr::MacAddr6 as MacAddress;
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashSet, fs::read_to_string, iter::FromIterator, path::Path};

type Start = u64;
type OuiMap = rangemap::RangeInclusiveMap<Start, Entry>;
type OuiMultiMap = multimap::MultiMap<String, Entry>;
type Error = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
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
    db: OuiMap,
    manufacturer_map: OuiMultiMap,
    manufacturers: HashSet<String>,
    ouis: HashSet<String>,
    records: i32,
}

impl Oui {
    #[cfg(feature = "with-db")]
    pub fn default() -> Result<Oui, Error> {
        //! Loads the default oui csv database
        //!
        //! ## Example
        //! ```rust
        //! use mac_oui::Oui;
        //!
        //! let db = Oui::default();
        //! assert!(db.is_ok());
        //! ```
        let db_text = include_str!("../assets/oui.csv");

        let oui_entry = read_into_db(db_text);
        match oui_entry {
            Ok(e) => Ok(Oui {
                db: e.0,
                manufacturer_map: e.1,
                manufacturers: e.2,
                ouis: e.3,
                records: e.4,
            }),
            Err(e) => Err(format!("Error: {}", e)),
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
        //! let db = Oui::from_csv_file("assets/oui.csv");
        //! assert!(db.is_ok())
        //! ```
        let db_text = if let Ok(contents) = read_to_string(oui_csv.as_ref()) {
            contents
        } else {
            return Err(format!(
                "could not open database file - {}",
                oui_csv.as_ref().to_str().unwrap()
            ));
        };
        let oui_entry = read_into_db(&db_text);
        match oui_entry {
            Ok(e) => Ok(Oui {
                db: e.0,
                manufacturer_map: e.1,
                manufacturers: e.2,
                ouis: e.3,
                records: e.4,
            }),
            Err(e) => Err(format!("Error: {}", e)),
        }
    }

    pub fn lookup_by_mac(&self, mac_addr: &str) -> Result<Option<&Entry>, Error> {
        //! Lookup for a Manufacturer Name based upon
        //! the given MAC Address
        let mac_addr: MacAddress = match mac_addr.parse() {
            Ok(m) => m,
            Err(e) => return Err(e.to_string()),
        };
        let mac_u = &mac_addr.to_u64();
        match mac_u {
            Ok(m) => self.query(m),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn lookup_by_manufacturer(
        &self,
        manufacturer_name: &str,
    ) -> Result<Option<&Vec<Entry>>, Error> {
        //! Lookup for the MAC Address Reference based
        //! upon the given Manufacturer Name
        match self
            .manufacturer_map
            .get_vec(&manufacturer_name.to_string())
        {
            Some(r) => Ok(Some(r)),
            _ => Ok(None),
        }
    }

    pub fn get_unique_manufacturers(&self) -> Result<Vec<String>, Error> {
        //! Get a list of Manufacturers present in the database
        Ok(Vec::from_iter(self.manufacturers.clone()))
    }

    pub fn get_unique_ouis(&self) -> Result<Vec<String>, Error> {
        //! Get a list of MAC OUI references present in the database
        Ok(Vec::from_iter(self.ouis.clone()))
    }

    pub fn get_total_records(&self) -> i32 {
        //! Get total records in the database
        self.records
    }

    /// Queries the database using a u64 representation from the wrapper query functions
    fn query(&self, query: &u64) -> Result<Option<&Entry>, Error> {
        match self.db.get(query) {
            Some(r) => Ok(Some(r)),
            _ => Ok(None),
        }
    }
}

trait MacAddrToU64 {
    fn to_u64(&self) -> Result<u64, Error>;
}

impl MacAddrToU64 for MacAddress {
    fn to_u64(&self) -> Result<u64, Error> {
        //! Converts a MAC Address to a u64 value
        let mac_bytes = self.as_bytes();

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
            return Err(format!(
                "could not read_u64 from padded MAC byte array: {:?}",
                padded_mac
            ));
        };
        Ok(mac_num)
    }
}

fn csv_de(csv_text: &str) -> Result<Vec<Entry>, csv::Error> {
    csv::Reader::from_reader(csv_text.as_bytes())
        .deserialize()
        .collect()
}

fn read_into_db(
    csv_text: &str,
) -> Result<(OuiMap, OuiMultiMap, HashSet<String>, HashSet<String>, i32), Error> {
    //! Reads the OUI CSV File into a Btree Map
    let mut oui_db = OuiMap::new();
    let mut manufacturer_map = OuiMultiMap::new();
    let mut unique_manufacturers = HashSet::<String>::new();
    let mut unique_ouis = HashSet::<String>::new();
    let mut nr_records = 0;

    let records = match csv_de(csv_text) {
        Ok(r) => r,
        Err(_e) => {
            return Err(String::from(
                "CSV file is not matching OUI CSV, \
            be sure to download here: https://macaddress.io/database-download/csv",
            ))
        }
    };

    // Loop thru
    for record in records {
        // Get the mask if any
        let mask: u8;
        let oui_mask: Vec<_> = record.oui.split('/').collect();
        match oui_mask.len() {
            1 => mask = 24,
            2 => {
                mask = oui_mask[1].parse::<u8>().unwrap();
                if !(8..=48).contains(&mask) {
                    return Err(format!("incorrect mask value: {}", mask));
                }
            }
            _ => return Err(format!("invalid number of mask separators: {:?}", oui_mask)),
        };

        // remove the separators from
        // the mac address string
        let oui = record
            .oui
            .to_uppercase()
            .replace(":", "")
            .replace("-", "")
            .replace(".", "");
        let oui_int = if let Ok(oi) = u64::from_str_radix(&oui, 16) {
            oi
        } else {
            return Err("could not parse OUI info, is this a oui csv file?".to_string());
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
            oui: record.oui.clone(),
            is_private: record.is_private,
            company_name: record.company_name.clone(),
            company_address: record.company_address,
            country_code: record.country_code,
            assignment_block_size: record.assignment_block_size,
            date_created: record.date_created,
            date_updated: record.date_updated,
        };
        nr_records += 1;
        oui_db.insert(oui_start..=oui_end, data.clone());
        manufacturer_map.insert(record.company_name.clone(), data);
        unique_manufacturers.insert(record.company_name);
        unique_ouis.insert(record.oui);
    }

    Ok((
        oui_db,
        manufacturer_map,
        unique_manufacturers,
        unique_ouis,
        nr_records,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "with-db")]
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
        let db = Oui::from_csv_file("assets/oui.csv").unwrap();

        let res = db.lookup_by_mac("70:B3:D5:e7:4f:81").unwrap();
        assert_eq!(res.unwrap().company_name, "Ieee Registration Authority")
    }

    #[test]
    fn test_get_by_manufacturer() {
        let db = Oui::from_csv_file("assets/oui.csv").unwrap();

        match db.lookup_by_manufacturer("Ieee Registration Authority") {
            Ok(m) => match m {
                Some(entries) => {
                    let ouis: Vec<String> = entries.iter().map(|e| e.oui.clone()).rev().collect();
                    return assert!(ouis.contains(&"70:B3:D5".to_string()));
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        }
    }

    #[test]
    fn test_get_unique_manufacturers() {
        let db = Oui::from_csv_file("assets/oui.csv").unwrap();

        let res = db.get_unique_manufacturers().unwrap();
        assert_eq!(res.len(), 29370)
    }

    #[test]
    fn test_get_unique_ouis() {
        let db = Oui::from_csv_file("assets/oui.csv").unwrap();

        let res = db.get_unique_ouis().unwrap();
        assert_eq!(res.len(), 47486)
    }

    #[test]
    fn test_get_records() {
        let db = Oui::from_csv_file("assets/oui.csv").unwrap();

        let res = db.get_total_records();
        assert_eq!(res, 47486)
    }
}
