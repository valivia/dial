#ifndef USB_H
#define USB_H

#include <stdint.h>

// Functions
void usb_configure(void);
void usb_send_key(uint8_t keycode[6]);

#endif // USB_H