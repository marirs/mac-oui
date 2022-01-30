use mac_oui::Oui;
use std::process::exit;

fn main() {
    let oui_db = match Oui::default() {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e);
            exit(1)
        }
    };
    let total_records = oui_db.get_total_records();
    let mut manufacturers = oui_db.get_unique_manufacturers().unwrap();
    let ouis = oui_db.get_unique_ouis().unwrap();

    manufacturers.sort();
    println!("Total Records= {}", total_records);
    println!("Total Manufacturers= {}", manufacturers.len());
    println!("Total MAC Addrs= {}", ouis.len());
    println!();
    println!("====Manufacturers====");
    println!("{:#?}", manufacturers.iter().take(20).collect::<Vec<_>>());
    println!("...");
    println!(
        "{:#?}",
        manufacturers.iter().rev().take(20).collect::<Vec<_>>()
    );
}
