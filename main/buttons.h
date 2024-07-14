#ifndef BUTTONS_H
#define BUTTONS_H

// Pins
#define BUTTON_SIGNAL_PIN 34
#define BUTTON_SELECT_1_PIN 36
#define BUTTON_SELECT_2_PIN 37
#define BUTTON_SELECT_3_PIN 35

// Functions
void set_demultiplex_channel(int index);
void buttons_configure_gpio();
void buttons_configure();
void buttons_task();


#endif // BUTTONS_H