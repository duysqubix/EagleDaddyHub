from typing import *
import time
from pathlib import Path
import colorama
from digi.xbee import serial

from digi.xbee.models.address import XBee64BitAddress
from digi.xbee.devices import DigiMeshDevice, RemoteDigiMeshDevice, DigiMeshNetwork
from digi.xbee.models.status import TransmitStatus
from digi.xbee.models.message import XBeeMessage
from digi.xbee.exception import TransmitException

from django.db import models
from django.db.models import Model

import pydaddy.comms as comms
from pydaddy.proxy import ProxyDigiMeshDevice
from pydaddy.models import RemoteModule
from pydaddy.utils import make_iter
from pydaddy.errors import ModuleNotFoundError
from pydaddy.comms import ResponseMessage, _ModuleCmds


class HubNet(DigiMeshNetwork):
    """
    wrapper around XbeeNetwork with more functionalities
    """
    def add_remotes(self, remotes):
        remotes = make_iter(remotes)
        super().add_remotes(remotes)

    @property
    def modules(self):
        return self.get_devices()

    @property
    def node_ids(self):
        return [x.get_node_id() for x in self.modules]


class RemoteModuleDevice(RemoteDigiMeshDevice):
    @property
    def address(self):
        return self.get_64bit_addr().address

    @property
    def node_id(self):
        return self.get_node_id()

    def __repr__(self) -> str:
        return f"<MODULE: {self.node_id}>"

    def __str__(self):
        return f"<MODULE: {self.address.hex()}"


class HubModule(DigiMeshDevice):
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

    def __init__(self, *args, **kwargs) -> None:
        super().__init__(port="COM4", baud_rate=9600, *args, **kwargs)
        if self.is_open():
            self.close()
        self.open()

        # add stored remote modules to internal network
        remotes = [
            RemoteModuleDevice(self,
                               x64bit_addr=XBee64BitAddress.from_hex_string(
                                   module.address64.hex()),
                               node_id=module.node_id)
            for module in RemoteModule.objects.all()
        ]
        if remotes:
            self.net.add_remotes(remotes)
            # check to see if they are still online

    def _init_network(self):
        return HubNet(self)

    @property
    def net(self):
        return self.get_network()

    def check_remote_connectivity(self):
        results = []
        for remote in self.net.modules:
            try:
                status, resp = self.send_data_to_module(
                    remote, _ModuleCmds.RQST_PING)

                results.append((remote, True))
            except TransmitException:
                results.append((remote, False))
        return results

    def discover_all_devices(self):
        self.net.start_discovery_process()
        count = 0
        init_scan = True
        print("Scanning..")
        while self.net.is_discovery_running():
            num_devices = len(self.net.modules)
            if num_devices > count:
                found_device_addr = self.net.modules[-1].get_64bit_addr(
                ).address

                try:
                    record = RemoteModule.objects.get(
                        address64=found_device_addr)
                    in_db = colorama.Fore.GREEN + u"\N{check mark}" + f": {record.node_id}, {record.address64.hex()[-4:]}" + colorama.Fore.RESET

                except RemoteModule.DoesNotExist:
                    in_db = colorama.Fore.RED + u"\N{ballot x}" + f": {found_device.node_id}, {found_device_addr.hex()[-4:]}" + colorama.Fore.RESET
                print(f"Found: {len(self.net.modules)} device(s) ({in_db})")
                count = num_devices

        print(f"Finished. Found {len(self.net.modules)} device(s).")

        for device in self.net.modules:
            device_addr = device.get_64bit_addr().address

            try:
                record = RemoteModule.objects.get(address64=device_addr)
            except RemoteModule.DoesNotExist:
                # add it
                new_record = ProxyDigiMeshDevice(device).to_model()
                new_record.save()

    def send_data_to_module(self, remote: Union[RemoteModuleDevice, str],
                            data: bytes) -> Tuple[TransmitStatus, XBeeMessage]:
        if isinstance(remote, str):
            remote_device = self.net.get_device_by_node_id(remote)

        elif isinstance(remote, RemoteModuleDevice):
            remote_device = remote

        if not remote_device:
            raise ModuleNotFoundError(
                "no matching remote device with node_id.")

        transmit_status: TransmitStatus = self.send_data(remote_device, data)
        response: ResponseMessage = ResponseMessage(self.read_data(1))

        return (transmit_status, response)
