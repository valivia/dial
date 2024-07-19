#ifndef USB_H
#define USB_H

#include <stdint.h>

// Functions
void usb_configure(void);
void usb_press_key(uint8_t keycode);
void usb_release_key(uint8_t keycode);
void usb_send_keys();

#endif // USB_H