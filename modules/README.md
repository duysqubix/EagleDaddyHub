
Prototype Module (0x001a):

Commands:

    RequestTime:            0x1d   // request time from Real world clock.
    RequestTempHumidity :   0x2b // request temperature
        Example:
            Master = [0x00, 0x1a, 0x2b]
            Slave = [0x00, 0x1a, 0x00, 0x04, 0x00, 0x1c]
    
    RequestDistance:        0x3c // request distance
    RequestMotorTime:       0x4a // request motor time


Example:
    Master -
        [0x00, 0x1a,  {cmd}, {data}]

    Slave -
        [0x00, 0x1a,  {data}]


Deer Feeder Module (0x002b)

    Master -
    [0x00, 0x2b]