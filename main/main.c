#include "freertos/FreeRTOS.h"
#include "esp_log.h"

// Modules
#include "indicators.h"
#include "buttons.h"
#include "touch.h"
#include "state.h"
#include "dial.h"
#include "wifi.h"
#include "mqtt.h"
#include "usb.h"

// Speaker
#define SPEAKER_DATA_PIN 8
#define SPEAKER_CLOCK_PIN 9
#define SPEAKER_WS_PIN 7

// Microphone
#define MIC_DATA_PIN 1

static const char *TAG = "core";

void app_main(void)
{
    ESP_LOGI(TAG, "[APP] Free memory: %" PRIu32 " bytes", esp_get_free_heap_size());
    ESP_LOGI(TAG, "[APP] IDF version: %s", esp_get_idf_version());

    state_configure();
    indicators_configure();
    buttons_configure();
    dial_configure();
    usb_configure();
    touch_configure();
    wifi_configure();
    mqtt_configure();
}
