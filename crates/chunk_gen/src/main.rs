use enet;
use flate2::read::GzDecoder;

use std::collections::{HashSet, VecDeque};
use std::io::Read;
use std::net::Ipv4Addr;

use anyhow::Context;
use enet::*;

const CLIENT_INFO: &'static str = "\x1f<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
	<key>alias</key>
	<string>123</string>
	<key>iCloudID</key>
	<string>7ccfbc59df252db72dc30d0feab294e8</string>
	<key>local</key>
	<true/>
	<key>micOrSpeakerOn</key>
	<true/>
	<key>minorVersion</key>
	<integer>3</integer>
	<key>playerID</key>
	<string>202cb962ac59075b964b07152d234b70</string>
	<key>udidNew</key>
	<string>5713cb6857c2a25d9811e4241569f297</string>
	<key>voiceConnected</key>
	<false/>
</dict>
</plist>
";

fn new_client_macro_block_request(macro_index: u32) -> [u8; 9] {
    let mut payload = [0u8; 9]; // Initialize a 9-byte array with all zeros
    payload[0] = 3;

    let value_bytes = macro_index.to_le_bytes();
    payload[1..5].copy_from_slice(&value_bytes);

    payload[5] = 1; // createIfNotCreated

    payload
}

fn main() -> anyhow::Result<()> {
    let enet = Enet::new().context("could not initialize ENet")?;

    let mut host = enet
        .create_host::<()>(
            None,
            10,
            ChannelLimit::Maximum,
            BandwidthLimit::Unlimited,
            BandwidthLimit::Unlimited,
        )
        .context("could not create host")?;

    host.connect(&Address::new(Ipv4Addr::LOCALHOST, 15151), 255, 0)
        .context("connect failed")?;

    let mut world_width_macro = 512;
    let mut chunks_to_generate = VecDeque::new();

    let mut received_chunks = HashSet::new();

    loop {
        let e = host.service(1000).context("service failed")?;

        let mut e = match e {
            Some(ev) => ev,
            _ => continue,
        };

        match e {
            Event::Connect(ref p) => {
                println!("Connected to peer: {:?}", p);
            }
            Event::Disconnect(ref p, r) => {
                println!("connection NOT successful, peer: {:?}, reason: {}", p, r);
                std::process::exit(0);
            }
            Event::Receive {
                ref mut sender,
                channel_id,
                ref packet,
                ..
            } => {
                let data = packet.data();
                match data.first() {
                    Some(b'#') => {
                        // world_id received
                        sender.send_packet(
                            Packet::new(CLIENT_INFO.as_bytes(), PacketMode::ReliableSequenced)?,
                            channel_id,
                        )?;
                    }
                    Some(1) => {
                        // world info received
                        // kickstart chunks to generate!
                        let gzip_data = &data[1..];
                        let mut world_info_plist = String::new();
                        let mut decoder = GzDecoder::new(gzip_data);
                        decoder.read_to_string(&mut world_info_plist).unwrap();

                        let dict =
                            plist::from_bytes::<plist::Dictionary>(world_info_plist.as_bytes())
                                .unwrap();
                        world_width_macro = dict
                            .get("worldWidthMacro")
                            .unwrap()
                            .as_unsigned_integer()
                            .unwrap() as u32;
                        println!("worldWidthMacro = {world_width_macro}");

                        chunks_to_generate
                            .extend((0..world_width_macro).flat_map(move |x| {
                                (0..32).map(move |y| y * world_width_macro + x)
                            }));
                        if let Some(macro_index) = chunks_to_generate.pop_front() {
                            let payload = new_client_macro_block_request(macro_index);
                            sender.send_packet(
                                Packet::new(&payload, PacketMode::ReliableSequenced)?,
                                channel_id,
                            )?;
                        }
                    }
                    Some(4) => {
                        // receiving chunk!
                        let finished_chunk_macro_index =
                            u32::from_le_bytes(data[1..5].try_into().unwrap());
                        let finished_chunk_x = finished_chunk_macro_index % world_width_macro;
                        let finished_chunk_y = finished_chunk_macro_index / world_width_macro;
                        let coord = (finished_chunk_x, finished_chunk_y);
                        if !received_chunks.contains(&coord) {
                            received_chunks.insert(coord);
                            println!(
                                "Received chunk ({}, {})!",
                                finished_chunk_x, finished_chunk_y
                            );
                        }
                        if received_chunks.len() == world_width_macro as usize * 32 {
                            break Ok(());
                        }
                        // Send some other chunk to generate.
                        if let Some(macro_index) = chunks_to_generate.pop_front() {
                            let payload = new_client_macro_block_request(macro_index);
                            sender.send_packet(
                                Packet::new(&payload, PacketMode::ReliableSequenced)?,
                                channel_id,
                            )?;
                        }
                    }
                    _ => {}
                }
            }
        };
    }
}
