use super::modules;
use lazy_static::lazy_static;
use rustbee::device;

lazy_static! {
    pub static ref REMOTE_MODULE_1: modules::Module = {
        modules::Module {
            id: 0x1234,
            device: device::RemoteDigiMeshDevice {
                addr_64bit: 1234,
                node_id: format!("DEVICE1"),
                firmware_version: Some(0xabcd),
                hardware_version: Some(0x4321),
            },
        }
    };
    pub static ref REMOTE_MODULE_2: modules::Module = {
        modules::Module {
            id: 0x1234,
            device: device::RemoteDigiMeshDevice {
                addr_64bit: 5678,
                node_id: format!("DEVICE2"),
                firmware_version: Some(0xabcd),
                hardware_version: Some(0x4321),
            },
        }
    };
}

#[test]
fn module_is_not_equal() {
    let ref m1 = *REMOTE_MODULE_1;
    let ref m2 = *REMOTE_MODULE_2;

    assert_eq!(false, m1.eq(m2));
}

#[test]
fn module_is_equal() {
    let ref m1 = *REMOTE_MODULE_1;
    assert_eq!(true, m1.eq(m1));
}
