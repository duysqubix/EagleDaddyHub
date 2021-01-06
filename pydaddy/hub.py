from typing import *
import time
from pathlib import Path

from digi.xbee.devices import DigiMeshDevice, RemoteDigiMeshDevice, XBeeNetwork
from django.db import models

from django.db.models import Model
from pydaddy.proxy import ProxyDigiMeshDevice
from pydaddy.models import RemoteModule


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

        self.device: DigiMeshDevice = DigiMeshDevice(port="COM4",
                                                     baud_rate=9600)
        self.xnet: XBeeNetwork = self.device.get_network()

    def open(self):
        self.device.open()

    def close(self):
        self.device.close()

    def discover_all_devices(self):
        self.open()
        self.xnet.start_discovery_process()
        count = 0
        init_scan = True
        while self.xnet.is_discovery_running():
            num_devices = len(self.xnet.get_devices())
            if num_devices > count:
                print(f"Found: {len(self.xnet.get_devices())} device(s)")
                count = num_devices
            elif count == 0 and init_scan:
                print("Scanning..")
                init_scan = False

        print(f"Finished. Found {len(self.xnet.get_devices())} device(s).")

        for device in self.xnet.get_devices():
            device_addr = device.get_64bit_addr().address

            try:
                record = RemoteModule.objects.get(address64=device_addr)
            except RemoteModule.DoesNotExist:
                # add it
                new_record = ProxyDigiMeshDevice(device).to_model()
                new_record.save()

        self.close()
        # add to database