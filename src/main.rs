mod arp;
use anyhow::{Result, bail};
use clap::Parser;
use pnet::datalink;
use pnet::datalink::{NetworkInterface};
use pnet::ipnetwork::{IpNetwork};
use crate::arp::arp_scan;

fn main() -> Result<()> {
    custom_utils::logger::logger_stdout_debug();
    let command: Arp = Arp::parse();
    // debug!("{:?}", command);
    println!();
    match command {
        Arp::List => {filter_interface()}
        Arp::Scan { index, mac , delay} => {
            let interface = get_interface(index)?;
            arp_scan(&interface, mac.map(|x| process_target_sub_mac(x)), delay)?;
        }
    }
    println!();
    Ok(())
}

fn process_target_sub_mac(target: String) -> String {
    target.to_uppercase().replace("-", ":")
}

pub fn filter_interface() {
    let interfaces = datalink::interfaces();
    for int in interfaces {
        let ip0 = int.ips.get(0);
        match (ip0, &int.mac) {
            (Some(IpNetwork::V4(ip)), Some(mac)) => {
                println!("\tindex: {:2},\t{}, {:?}, {:?}", int.index, int.description, ip.ip(), mac)
            },
            _ => {
                println!("\tindex: {:2},\t{}, None, None", int.index, int.description)
            }
        }
    }
}

pub fn get_interface(index: u32) -> Result<NetworkInterface> {
    let interfaces = datalink::interfaces();
    for int in interfaces {
        if int.index == index {
            return Ok(int);
        }
    }
    bail!("not this index!");
}

#[derive(Parser, Debug)]
#[command(name="cargo-arp", version="0.1.0", about="a arp tool.")]
pub enum Arp {
    #[command(name="list", about="list these interface of network.")]
    List,
    #[command(name="scan", about="scan index of interface, and filter mac if present."
    , after_help="e.g., cargo-arp scan 8 d2:b5   \n\t cargo-arp scan 8\n\t cargo-arp scan 8 d2:b5 -d 3000.")]
    Scan {
        #[arg(help="index of interface.")]
        index: u32,
        #[arg(help="mac to filter, don't want to be complete.")]
        mac: Option<String>,
        #[arg(short, default_value="3000", help="Wait time for arp response, unit: ms.")]
        delay: u64,
    },
}