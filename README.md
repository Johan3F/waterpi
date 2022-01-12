# waterpi
Self watering system for platns using a raspberry pi. 

It uses a MCP3008 chip for reading analog values from a moisture sensor. so it can take up to 8 plants at the same time. It expects one pump per plant, since each plant will be monitored individually.

These are the components that are expected for monitoring one plant:
- 1 Raspberry PI 3 or above with SPI interface ([For more info](https://raspberrypi-aa.github.io/session3/spi.html))
- 1 Humidity sensor. I've installed a capacitive sensor, since I read that resistive ones are more prone to corrosion. Didn't verify this information, but I went with a capacitive sensor nonetheless
- 1 MCP3008 chip. Translation from analog humidity sensor to digital
- 1 Relay. Provides control with logical voltage to higher voltage
- 1 water pump that can run with 5V that the raspberry offers
- Plastic tubing for getting the water from the pump to the plant