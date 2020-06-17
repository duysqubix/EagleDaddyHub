#include "globals.h"
#include <Arduino.h>
#include <string.h>

RecieveFrame g_RxFrame;

void setup()
{
    Serial.begin(115200);
    Serial.setTimeout(100);
    while (!Serial)
        ;
}

void reverse(uint8_t arr[], uint8_t n)
{
    for (uint8_t low = 0, high = n - 1; low < high; low++, high--) {
        uint8_t temp = arr[low];
        arr[low] = arr[high];
        arr[high] = temp;
    }
}

void parse_rx_packet()
{
    uint8_t size = sizeof(uint64_t);
    uint8_t addr[size], data[size];

    for (int i = 0; i < size; i++) {
        addr[i] = RX_PACKET[i + 1]; // store address values of packet
    }

    for (int i = 0; i < MAX_RF_DATA_LEN; i++) {
        data[i] = RX_PACKET[i + 12]; // store actual rf data of packet
    }

    reverse(addr, size); // reverse as AVR stores integers little-endian, and incoming data is big-endian
    memcpy(&g_RxFrame.source_addr, addr, size); // store source addr to rx frame

    g_RxFrame.recv_opts = RX_PACKET[11]; // store recieve options

    memcpy(&g_RxFrame.rf_data, data, MAX_RF_DATA_LEN); // copy rf data to
}

void transmit_request(uint8_t* data, uint8_t len)
{
    Serial.setTimeout(500);
    uint8_t packet_len = len + 18;
    uint8_t packet[packet_len];

    packet[0] = 0x7e;
    packet[1] = ((packet_len - 4) >> 8) & 0xff;
    packet[2] = (packet_len - 4) & 0xff;
    packet[3] = 0x10;
    packet[4] = 0x01; // frame_id =0, no respnose frame sent to this device
    packet[5] = (g_RxFrame.source_addr >> 56) & 0xff;
    packet[6] = (g_RxFrame.source_addr >> 48) & 0xff;
    packet[7] = (g_RxFrame.source_addr >> 40) & 0xff;
    packet[8] = (g_RxFrame.source_addr >> 32) & 0xff;
    packet[9] = (g_RxFrame.source_addr >> 24) & 0xff;
    packet[10] = (g_RxFrame.source_addr >> 16) & 0xff;
    packet[11] = (g_RxFrame.source_addr >> 8) & 0xff;
    packet[12] = g_RxFrame.source_addr & 0xff;
    packet[13] = 0xff;
    packet[14] = 0xfe;
    packet[15] = 0; // broadcast radius
    packet[16] = 0; // transmit options

    for (uint8_t i = 0; i < len; i++) {
        packet[17 + i] = *data;
        data++;
    }

    uint8_t chksum = 0;
    for (uint8_t i = 3; i < packet_len - 1; i++) {
        chksum = chksum + packet[i];
    }

    packet[packet_len - 1] = 0xff - chksum;

    uint8_t tries = 0;
    const uint8_t max_tries = 3;
    // now attempt to send it, it will try a max of three times
    // if it doesn't recieve a transmit status report, within the alloted timeout and max tries has been attempted,
    // it will silently discard everything, and return to normal mode

    uint8_t status[11];
    bool success = false;
    while (tries < max_tries) {
        for (uint8_t i = 0; i < sizeof(packet); i++) {
            Serial.write(packet[i]);
        }

        Serial.flush();

        if (!Serial.readBytes(status, sizeof(status))) {
            // attempt gone wrong, retry
            tries++;
            continue;
        } else {
            // success full we can do some extra checks here
            if (status[3] == 0x8b)
                break; // looks like a successful transmit
            tries++;
            continue;
        }
    }
}

enum Commands {
    RequestTime = 0x1d,
    RequestTH = 0x2b,
    RequestDist = 0x3c,
    RequestMotor = 0x4a,
    BroadcastId = 0x0aaa,
};

void process_cmd(MasterRequest* request)
{
    uint8_t cmd = request->cmd;

    // Send back temperature and Humditity
    //
    // Response packet in the form of:
    // [mod_id, temp, hum]; where temp in C, hum = % both uint16_t
    if (cmd == RequestTH) {
        uint8_t to_send[] = { 0x00, 0x1a, 0x00, 0x3d, 0x00, 0x45 };
        transmit_request(to_send, sizeof(to_send));
    }

    // Send back real time as described by World Clock
    //
    // Response:
    // [mod_id, u64 value that contains time]
    else if (cmd == RequestTime) {
        uint8_t to_send[] = { 0x00, 0x01, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8 };
        transmit_request(to_send, sizeof(to_send));
    }

    // Sends back running motor time when motor is running, uint16_t in secs
    //
    // Response:
    // [mod_id, time(s)]
    else if (cmd == RequestMotor) {
        uint8_t to_send[] = { 0x00, 0x1a, 0x00, 0x0c };
        transmit_request(to_send, sizeof(to_send));
    }

    // Sends back distance to feed from top of barrel as uint16_t in cm
    //
    //Repsonse:
    // [mod_id, distance(cm)]
    else if (cmd == RequestDist) {
        uint8_t to_send[] = { 0x00, 0x1a, 0x00, 0x19 };
        transmit_request(to_send, sizeof(to_send));
    } else {
        uint8_t to_send[5] = { 0x00, 0x1a, 0xff, 0x2 }; // unknown command
        to_send[4] = cmd;
        transmit_request(to_send, sizeof(to_send));
    }
}

void handle_packets()
{
    switch (RX_PACKET[0]) {
    case 0x90:
        MasterRequest request;
        parse_rx_packet(); // populates g_RxFrame

        // parse out the request from master here
        memcpy(&request, &g_RxFrame.rf_data, MAX_RF_DATA_LEN);
        request.module_id = (g_RxFrame.rf_data[0] << 8) | g_RxFrame.rf_data[1];

        if (request.module_id != MOD_ID) {
            // firs check to see if it isn't broadcast request
            // check for broadcast
            // Internal Broadcast command - sends back its module_id. Must use a random number
            // as this cmd is intended to be used when host uses 'broadcast' mode, we don't want
            // to saturate RF bus with multiple data bits
            if (request.module_id == BroadcastId) {
                uint8_t to_send[] = { 0x00, 0x1a };
                delay(random(0, 2000)); // wait anywhere between 0ms and 2000ms
                transmit_request(to_send, sizeof(to_send));
                break;
            }

            // no? okay, then error out
            uint8_t err[] = { 0x00, 0x1a, 0xff, 0x01 }; // Error, Wrong Module ID
            transmit_request(err, 4);
            break;
        }

        process_cmd(&request);

        break;
    default:
        break;
    }
}

void debug()
{
    uint32_t upper = (g_RxFrame.source_addr >> 24);
    uint32_t lower = g_RxFrame.source_addr & 0xffffffff;
    Serial.print(upper, HEX);
    Serial.print(" ");
    Serial.print(lower, HEX);
    Serial.print(",");
    Serial.print(g_RxFrame.recv_opts, HEX);
    Serial.print(", ");
    for (int i = 0; i < MAX_RF_DATA_LEN; i++) {
        Serial.print(g_RxFrame.rf_data[i], HEX);
        Serial.print(" ");
    }
    Serial.println("");
}

uint8_t buf;

void loop()
{

    // put your main code here, to run repeatedly:
    if (Serial.available() > 0) {
        for (;;) {
            buf = Serial.read();
            if (buf == 0x7e)
                break;
        }
        Serial.readBytes(HEADER, 2);

        //if (HEADER[0] != 0x7e)
        //    return;
        uint16_t buf_len = (HEADER[0] << 8) | HEADER[1];

        Serial.readBytes(RX_PACKET, buf_len + 1);

        handle_packets();
        //Serial.read(Serial.available());
        //        debug();
    }
}
