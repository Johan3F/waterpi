import spidev
import time
import os

# Open SPI bus
spi = spidev.SpiDev()
spi.open(0, 0)
spi.max_speed_hz = 1000000

# Function to read SPI data from MCP3008 chip
# Channel must be an integer 0-7
def ReadChannel(channel):
    adc = spi.xfer2([1, (8 + channel) << 4, 0])
    data = ((adc[1] & 3) << 8) + adc[2]
    return data


# Function to convert data to voltage level,
# rounded to specified number of decimal places.
def ConvertVolts(data, places):
    volts = (data * 3.3) / float(1023)
    volts = round(volts, places)
    return volts


# Define sensor channels
moisture_channel = 7

# Define delay between readings
delay = 1

while True:

    # Read the light sensor data
    moisture_level = ReadChannel(moisture_channel)
    moisture_volts = ConvertVolts(moisture_level, 2)

    # Print out results
    print("--------------------------------------------")
    print("Moisture: {} ({}V)".format(moisture_level, moisture_volts))

    # Wait before repeating loop
    time.sleep(delay)
