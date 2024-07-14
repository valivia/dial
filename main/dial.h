#ifndef DIAL_H
#define DIAL_H

// Pins
#define DIAL_DATA_PIN 5
#define DIAL_MODE_PIN 4


// Functions
void dial_interrupt_handler(void *arg);
void dial_configure_gpio();
void dial_configure();

#endif // DIAL_H