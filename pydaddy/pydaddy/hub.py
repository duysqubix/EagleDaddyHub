from pydaddy.db.models import RemoteModule
from typing import *
import yaml
import time
from pathlib import Path

from digi.xbee.devices import DigiMeshDevice, RemoteDigiMeshDevice, XBeeNetwork

from pydaddy.db import DB, CONFIG
from pydaddy.db.proxy import ProxyDigiMeshDevice


class HubModule:
    """
    A singleton instance representing the main
    hub module, the one that interacts with the rest of 
    the network
    """
    __instance = None

    def __new__(cls) -> Any:
        if cls.__instance is None:
            cls.__instance = super(HubModule, cls).__new__(cls)
        return cls.__instance

    def __init__(self) -> None:
        global DB, CONFIG

        baud: int = CONFIG['baud']
        port: str = CONFIG['port']
        self.device: DigiMeshDevice = DigiMeshDevice(port=port, baud_rate=baud)
        self.xnet: XBeeNetwork = self.device.get_network()
        self.db = DB
        self.config = CONFIG

    def open(self):
        self.device.open()

    def close(self):
        self.device.close()

    def discover_all_devices(self):
        self.open()
        self.xnet.start_discovery_process()
        count = 0
        while self.xnet.is_discovery_running():
            num_devices = len(self.xnet.get_devices())
            if num_devices > count:
                print(f"Found: {len(self.xnet.get_devices())} device(s)")
                count = num_devices
            elif count == 0:
                print("Scanning..")
            time.sleep(3)

        print(f"Finished. Found {len(self.xnet.get_devices())} device(s).")

        for device in self.xnet.get_devices():
            record = self.db.query(RemoteModule).filter(
                RemoteModule.address64 ==
                device.get_64bit_addr().address).first()

            if not record:
                # add it
                new_record = ProxyDigiMeshDevice(device).to_model()
                self.db.add(new_record)
                self.db.commit()
        self.close()
        # add to database