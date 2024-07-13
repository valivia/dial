#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <inttypes.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "freertos/queue.h"
#include "driver/gpio.h"

// Modules
#include "./buttons.h"
#include "./dial.h"

// Speaker
#define SPEAKER_DATA_PIN 8
#define SPEAKER_CLOCK_PIN 9
#define SPEAKER_WS_PIN 7

// Microphone
#define MIC_DATA_PIN 1

// Indicators
#define INDICATOR_1_PIN 21
#define INDICATOR_2_PIN 2

void app_main(void)
{
    configure_buttons();
    configure_dial();
}
