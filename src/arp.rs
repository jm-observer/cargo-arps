use anyhow::{anyhow, bail, Result};
use log::{error, warn};
use pnet::datalink::{Config, DataLinkReceiver, DataLinkSender, MacAddr, NetworkInterface};
use pnet::ipnetwork::IpNetwork;
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::MutablePacket;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::ErrorKind;
use std::net::Ipv4Addr;

use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EtherTypes, MutableEthernetPacket};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

pub fn arp_scan(
    interface: &NetworkInterface,
    target_sub_mac: Option<String>,
    delay: u64,
    order_by_mac: bool,
) -> Result<()> {
    let (ipv4_network, src) = match (interface.ips.get(0), &interface.mac) {
        (Some(IpNetwork::V4(ip)), Some(mac)) => {
            println!("scan interface:");
            println!(
                "\tindex: {:2},\t{}, {:?}, {:?}",
                interface.index,
                interface.description,
                ip.ip(),
                mac
            );
            println!();
            (ip.clone(), mac.clone())
        }
        _ => {
            println!("none ipv4 or none mac");
            bail!("none ipv4 or none mac")
        }
    };

    let (mut tx, rx) = get_datalink_channel(&interface, delay)?;
    let name = interface.name.clone();
    let (tmp_tx, tmp_rx) = channel::<ArpAck>();
    thread::spawn(move || collect_arp_response(rx, tmp_tx, name));
    let local_ip = ipv4_network.ip();
    println!("start to send arp request……");
    for ipv4 in ipv4_network.iter() {
        // debug!("{}", ipv4);
        let arp_packet = init_arp_packet(src, local_ip, ipv4)?;
        if let Some(Err(e)) = tx.send_to(&arp_packet, Some(interface.clone())) {
            bail!("error: {:?}", e);
        }
    }
    println!("request sended, listening response……");
    thread::sleep(Duration::from_millis(delay));
    let mut targets = HashSet::<ArpAck>::new();
    while let Ok(ack) = tmp_rx.try_recv() {
        targets.insert(ack);
    }
    let mut targets: Vec<ArpAck> = targets.into_iter().collect();
    if order_by_mac {
        targets.sort_by(|x, y| x.mac.cmp(&y.mac));
    } else {
        targets.sort_by(|x, y| x.ip.cmp(&y.ip));
    }
    let mut aim_targets = HashSet::<ArpAck>::new();
    println!("all responses：");
    for target in targets {
        println!("\t{}  {}", target.mac, target.ip);
        if let Some(ref sub_mac) = target_sub_mac {
            if target.mac.to_uppercase().contains(sub_mac) {
                aim_targets.insert(target);
            }
        }
    }

    if target_sub_mac.is_some() {
        println!("filter result：");
        for target in aim_targets {
            println!("\t{}  {}", target.mac, target.ip);
        }
    }

    Ok(())
}

/// 收集识别的回复包
fn collect_arp_response(mut rx: Box<dyn DataLinkReceiver>, sender: Sender<ArpAck>, name: String) {
    loop {
        match rx.next() {
            Ok(packet) => {
                // if timeout_flag.load(Ordering::Relaxed) {
                //     break;
                // }
                if let Some(arp_packet) =
                    ArpPacket::new(&packet[MutableEthernetPacket::minimum_packet_size()..])
                {
                    let operation = arp_packet.get_operation();
                    if operation == ArpOperations::Reply {
                        let sender_ipv4 = arp_packet.get_sender_proto_addr();
                        let sender_mac = arp_packet.get_sender_hw_addr();

                        if let Err(e) = sender.send(ArpAck {
                            mac: sender_mac.to_string(),
                            ip: sender_ipv4,
                        }) {
                            error!("sende error: {:?}", e);
                        }
                    }
                }
            }
            Err(_e) => {
                if _e.kind() == ErrorKind::TimedOut {
                    warn!("{}接收数据链路层数据超时", name);
                } else {
                    error!("{}接收数据链路层数据失败：{:?}", name, _e);
                }
                // error!("{}接收数据链路层数据失败：{:?}", name, _e);
                break;
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Eq, Hash, PartialEq)]
pub struct ArpAck {
    pub(crate) mac: String,
    pub(crate) ip: Ipv4Addr,
}

pub fn init_arp_packet(
    source_mac: MacAddr,
    source_ip: Ipv4Addr,
    target_ip: Ipv4Addr,
) -> anyhow::Result<[u8; 42]> {
    let mut ethernet_buffer = [0u8; 42];
    let mut ethernet_packet =
        MutableEthernetPacket::new(&mut ethernet_buffer).ok_or(anyhow!("初始化arp请求包失败"))?;
    ethernet_packet.set_destination(MacAddr::broadcast());
    ethernet_packet.set_source(source_mac);
    ethernet_packet.set_ethertype(EtherTypes::Arp);

    let mut arp_buffer = [0u8; 28];
    let mut arp_packet =
        MutableArpPacket::new(&mut arp_buffer).ok_or(anyhow!("初始化arp请求包失败"))?;

    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4);
    arp_packet.set_operation(ArpOperations::Request);
    arp_packet.set_sender_hw_addr(source_mac);
    arp_packet.set_sender_proto_addr(source_ip);
    arp_packet.set_target_hw_addr(MacAddr::zero());
    arp_packet.set_target_proto_addr(target_ip);

    ethernet_packet.set_payload(arp_packet.packet_mut());

    Ok(ethernet_buffer)
}

pub fn get_datalink_channel(
    interface: &NetworkInterface,
    delay: u64,
) -> Result<(Box<dyn DataLinkSender>, Box<dyn DataLinkReceiver>)> {
    let mut config = Config::default();
    config.read_timeout = Some(Duration::from_millis(delay));
    match datalink::channel(interface, config) {
        Ok(Ethernet(tx, rx)) => Ok((tx, rx)),
        Ok(_) => bail!("Unhandled channel type"),
        Err(e) => bail!(
            "An error occurred when creating the datalink channel: {}",
            e
        ),
    }
}
