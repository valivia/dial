#ifndef INDICATORS_H
#define INDICATORS_H

// Pins
#define INDICATOR_1_PIN 21
#define INDICATOR_2_PIN 2

// Functions
void indicators_configure_gpio();
void indicators_configure();
void indicators_activate(int indicator, long int time_ms);
void indicators_deactivate(int indicator);

#endif // INDICATORS_H