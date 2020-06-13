typedef unsigned int uint16_t;
typedef unsigned char uint8_t;

uint8_t HEADER[3], API_PACKET[128];
uint8_t cntr = 0;
uint16_t buf_len;

void setup()
{
    // put your setup code here, to run once:
    Serial.begin(9600);
    Serial.setTimeout(100);
    pinMode(LED_BUILTIN, HIGH);
    digitalWrite(LED_BUILTIN, LOW);
}

void handle_packets()
{
    switch
        API_PACKET[0]
        {
        case 0x90:
            buf_len
                // handle rx indicator packet here
                break;
        case 0x91:
            // handle rx explicit ehere
            break;
        }
}

void loop()
{
    // put your main code here, to run repeatedly:
    if (Serial.available() > 0)
    {
        digitalWrite(LED_BUILTIN, HIGH);
        Serial.readBytes(HEADER, 3);

        if (HEADER[0] != 0x7e)
            return;
        uint16_t buf_len = (HEADER[1] << 8) | HEADER[2];

        Serial.readBytes(API_PACKET, buf_len + 1);
        Serial.print("BUF LEN: ");
        Serial.println(buf_len);
        for (int i = 0; i < buf_len; i++)
        {
            Serial.print(API_PACKET[i], HEX);
        }
    }

    handle_packets();
}