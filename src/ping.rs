use std::collections::HashSet;
use anyhow::{anyhow, bail, Result};
use log::{error, warn};
use pnet::ipnetwork::{IpNetwork, Ipv4Network};
use std::net::{IpAddr, Ipv4Addr, SocketAddrV4};

use std::time::Duration;
use pnet::datalink::NetworkInterface;
use rand::random;
use surge_ping::{Client, IcmpPacket, PingIdentifier, PingSequence};

const PAYLOAD: [u8; 56] = [0; 56];

pub async fn ping_scan(
    interface: &NetworkInterface,
    delay: u64,
    targets: Ipv4Network
) -> Result<()> {
    let ipv4_network = match interface.ips.get(0) {
        Some(IpNetwork::V4(ip)) => {
            ip.clone()
        },
        _ => {
            bail!("none ipv4")
        }
    };
    let socket_addr = SocketAddrV4::new(ipv4_network.ip(), 0);
    // let config = surge_ping::Config::builder().bind(socket_addr.into()).build();
    let config = surge_ping::Config::builder().build();
    let client = Client::new(&config).unwrap();
    let mut handlers = Vec::new();
    for ipv4 in targets.iter() {
        // println!("{:?}", ipv4);
        let addr = IpAddr::V4(ipv4);
        let mut pinger = client.pinger(addr, PingIdentifier(random())).await;
        pinger.timeout(Duration::from_secs(delay));
        handlers.push(tokio::spawn(async move {
            pinger.ping(PingSequence(0), &PAYLOAD).await
        }));
    }
    for handler in handlers {
        match handler.await? {
                Ok((IcmpPacket::V4(packet), rtt)) => {
                    println!("{:?} {:0.2?}", packet.get_source(), rtt);
                }
                _ => {},
            }
    }
    Ok(())
}