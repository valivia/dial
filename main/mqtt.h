#ifndef MQTT_H
#define MQTT_H

// Functions
void mqtt_configure(void);
void mqtt_publish(const char *topic, const char *data);

#endif // MQTT_H