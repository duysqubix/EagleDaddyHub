
#ifndef __GLOBALS_H__
#define __GLOBALS_H__

#define MAX_RF_DATA_LEN 8
#define MAX_RX_PACKET_LEN 64

typedef unsigned long long uint64_t;
typedef unsigned long uint32_t;
typedef unsigned int uint16_t;
typedef unsigned char uint8_t;

typedef struct {
    uint64_t source_addr;
    uint8_t recv_opts;
    uint8_t rf_data[MAX_RF_DATA_LEN];
} RecieveFrame;

typedef struct {
    uint16_t module_id;
    uint8_t cmd;
    uint8_t args[5];
} MasterRequest;

const uint16_t MOD_ID = 0x001a;
uint8_t HEADER[3], RX_PACKET[MAX_RX_PACKET_LEN];
uint8_t RX_BUF;
#endif
