use mac_oui::Oui;
use std::{
    env,
    process::exit
};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("pass a mac address string for lookup");
        exit(1);
    }
    let mac_addr = args[1].clone();

    let oui_db = match Oui::default() {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e);
            exit(1)
        }
    };
    let res = oui_db.lookup(&mac_addr);
    match res {
        Ok(r) => {
            if let Some(rec) = r {
                println!("{:#?}", &rec)
            } else {
                println!("No entry found for: {}", mac_addr)
            }
        },
        Err(e) => println!("Error: {}", e)
    }
}
