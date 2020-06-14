//mod modules;

use rustbee::{
    api::{self, TransmitApiFrame},
    device::DigiMeshDevice,
};

use downcast_rs::DowncastSync;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut device = DigiMeshDevice::new()?;
    //    device
    //        .send_frame(api::AtCommandFrame("AP", Some(b"1")))?
    //        .summary();

    let packet = api::TransmitRequestFrame {
        dest_addr: api::BROADCAST_ADDR,
        broadcast_radius: 0,
        options: None,
        payload: b"\x00\x1a\x2b",
    };
    let response = device.send_frame(packet)?;
    println!("{:#x?}", response);
    let remote_atcmd = api::RemoteAtCommandFrame {
        dest_addr: api::BROADCAST_ADDR,
        options: &api::RemoteCommandOptions {
            apply_changes: true,
        },
        atcmd: "ID",
        cmd_param: None,
    };

    let response = device.send_frame(remote_atcmd)?;

    if let Some(r) = response.downcast_ref::<api::RemoteAtCommandResponse>() {
        println!("{:?}", r.command_data);
    }

    // get id of device
    //    let at_id = api::AtCommandFrame("ID", None);
    //  let reponse = device.send_frame(at_id)?;
    //
    //   let at_api = api::AtCommandFrame("AP", Some(b"1"));
    //  let response = device.send_frame(at_api)?;

    // reponse.summary();
    //response.summary();
    //    // set api to 1
    //    device.command_mode(true)?;
    //    println!("{:x?}", device.rx_buf);
    //    device.atcmd(&api::AtCommands::AtCmd(("AP", Some(b"1"))).create())?;
    //    println!("{:x?}", device.rx_buf);
    //    device.command_mode(false)?;
    //    println!("{:x?}", device.rx_buf);
    //    println!("Attempting to send something");
    //
    //    let packet = api::TransmitRequestFrame {
    //        dest_addr: api::BROADCAST_ADDR,
    //        broadcast_radius: 0,
    //        options: Some(&api::TransmitRequestOptions {
    //            disable_ack: false,
    //            disable_route_discovery: false,
    //            enable_unicast_nack: false,
    //            enable_unicast_trace_route: false,
    //            mode: api::MessagingMode::DigiMesh,
    //        }),
    //        payload: b"hi",
    //    };
    //
    //    let status = device.send_frame(packet)?;
    //    status.summary();
    Ok(())
}