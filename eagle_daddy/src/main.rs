//mod modules;

use rustbee::{api, device::DigiMeshDevice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut device = DigiMeshDevice::new("COM1", 9600)?;
    //    device
    //        .send_frame(api::AtCommandFrame("AP", Some(b"1")))?
    //        .summary();
    println!("{:#?}", device);
    device.discover_nodes(None)?;
    println!("{:#x?}", device.nodes);
    // let atni = api::AtCommandFrame("NI", None);
    // let node_id = device.send_frame(atni)?;

    // if let Some(node_id) = node_id.downcast_ref::<api::AtCommandResponse>() {
    //     println!("{:#?}", &node_id.command_data.as_ref().unwrap());
    // }
    // let packet = api::TransmitRequestFrame {
    //     dest_addr: api::BROADCAST_ADDR,
    //     broadcast_radius: 0,
    //     options: None,
    //     payload: b"\x00\x1a\x2b",
    // };
    // device.send_frame(packet)?.summary();
    // let remote_atcmd = api::RemoteAtCommandFrame {
    //     dest_addr: api::BROADCAST_ADDR,
    //     options: &api::RemoteCommandOptions {
    //         apply_changes: true,
    //     },
    //     atcmd: "ID",
    //     cmd_param: None,
    // };

    // let response = device.send_frame(remote_atcmd)?;

    // if let Some(r) = response.downcast_ref::<api::RemoteAtCommandResponse>() {
    //     println!("{:?}", r.command_data);
    // }

    Ok(())
}
