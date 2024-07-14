#ifndef INDICATORS_H
#define INDICATORS_H

// Pins
#define INDICATOR_1_PIN 21
#define INDICATOR_2_PIN 2

// Functions
void indicators_configure_gpio();
void indicators_configure();
void indicators_set_state(int state);

#endif //INDICATORS_H