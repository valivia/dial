#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <inttypes.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "freertos/queue.h"
#include "driver/gpio.h"

// Modules
#include "indicators.h"
#include "buttons.h"
#include "touch.h"
#include "dial.h"
#include "usb.h"

// Speaker
#define SPEAKER_DATA_PIN 8
#define SPEAKER_CLOCK_PIN 9
#define SPEAKER_WS_PIN 7

// Microphone
#define MIC_DATA_PIN 1

void app_main(void)
{
    indicators_configure();
    buttons_configure();
    dial_configure();
    usb_configure();
    touch_configure();
}
