import argparse
from serial.tools import list_ports
from digi.xbee.devices import DigiMeshDevice

from pydaddy.db import db
from pydaddy.db.models import RemoteModule

parser = argparse.ArgumentParser()
parser.add_argument(
    "-a",
    "--add",
    help=
    "Adds newly connected device and initializes with default configuration settings"
)
parser.add_argument(
    "-l",
    "--list",
    help=
    "list all detected devices and shows whether it has been added to main db",
    action="store_true")

if __name__ == '__main__':
    args = parser.parse_args()
    if args.list:
        for comdevice in list_ports.comports():
            # check to see if comdevice has xbee attached to it.
            device = DigiMeshDevice(comdevice.device, 9600)
            device.open()
            print(device.get_64bit_addr())
            device.close()
