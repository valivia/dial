#include <stdlib.h>
#include "esp_log.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "tinyusb.h"
#include "class/hid/hid_device.h"
#include "driver/gpio.h"
#include "dial.h"
#include "indicators.h"

static const char *TAG = "usb";
static bool usb_enabled = false;

#define TUSB_DESC_TOTAL_LEN (TUD_CONFIG_DESC_LEN + CFG_TUD_HID * TUD_HID_DESC_LEN)

const uint8_t hid_report_descriptor[] = {
    TUD_HID_REPORT_DESC_KEYBOARD(HID_REPORT_ID(HID_ITF_PROTOCOL_KEYBOARD)),
};

const char *hid_string_descriptor[5] = {
    (char[]){0x09, 0x04},     // 0: is supported language is English (0x0409)
    "Owl Corp",               // 1: Manufacturer
    "Dial",                   // 2: Product
    "000000",                 // 3: Serials, should use chip ID
    "Rotary phone interface", // 4: HID
};

static const uint8_t hid_configuration_descriptor[] = {
    TUD_CONFIG_DESCRIPTOR(1, 1, 0, TUSB_DESC_TOTAL_LEN, TUSB_DESC_CONFIG_ATT_REMOTE_WAKEUP, 500),

    TUD_HID_DESCRIPTOR(0, 4, false, sizeof(hid_report_descriptor), 0x81, 16, 10),
};


uint8_t const *tud_hid_descriptor_report_cb(uint8_t instance)
{
    // We use only one interface and one HID report descriptor, so we can ignore parameter 'instance'
    return hid_report_descriptor;
}


uint16_t tud_hid_get_report_cb(uint8_t instance, uint8_t report_id, hid_report_type_t report_type, uint8_t *buffer, uint16_t reqlen)
{
    (void)instance;
    (void)report_id;
    (void)report_type;
    (void)buffer;
    (void)reqlen;

    return 0;
}


void tud_hid_set_report_cb(uint8_t instance, uint8_t report_id, hid_report_type_t report_type, uint8_t const *buffer, uint16_t bufsize)
{
}

/********* Application ***************/

#define MAX_KEYS 6

static uint8_t current_keys[MAX_KEYS] = {0};

void usb_send_keys()
{
    if (usb_enabled == false || !tud_mounted())
    {
        ESP_LOGW(TAG, "USB not mounted");
        return;
    }

    tud_hid_keyboard_report(HID_ITF_PROTOCOL_KEYBOARD, 0, current_keys);
}

void usb_release_key(uint8_t keycode)
{
    bool found = false;
    for (int i = 0; i < MAX_KEYS; i++)
    {
        if (current_keys[i] == keycode)
        {
            found = true;
            current_keys[i] = 0;
        }
    }

    if (!found)
    {
        ESP_LOGW(TAG, "Failed to release, Key not pressed: %d", keycode);
        return;
    }

    ESP_LOGI(TAG, "Key released: %d", keycode);
    usb_send_keys();
}

void usb_press_key(uint8_t keycode)
{
    for (int i = 0; i < MAX_KEYS; i++)
    {
        if (current_keys[i] == keycode)
        {
            ESP_LOGW(TAG, "Failed to press, Key already pressed: %d", keycode);
            return;
        }
    }

    for (int i = 0; i < MAX_KEYS; i++)
    {
        if (current_keys[i] == 0)
        {
            current_keys[i] = keycode;
            break;
        }
    }

    ESP_LOGI(TAG, "Key pressed: %d", keycode);
    usb_send_keys();
}

void usb_configure(void)
{
    if (!CONFIG_ESP_USB_ACTIVE || gpio_get_level(DIAL_MODE_PIN) == 0)
    {
        indicators_set_state(1);
        ESP_LOGW(TAG, "USB not enabled");
        return;
    }

    ESP_LOGI(TAG, "USB initialization");
    const tinyusb_config_t tusb_cfg = {
        .device_descriptor = NULL,
        .string_descriptor = hid_string_descriptor,
        .string_descriptor_count = sizeof(hid_string_descriptor) / sizeof(hid_string_descriptor[0]),
        .external_phy = false,
        .configuration_descriptor = hid_configuration_descriptor,
    };

    ESP_ERROR_CHECK(tinyusb_driver_install(&tusb_cfg));
    usb_enabled = true;
    ESP_LOGI(TAG, "USB initialization DONE");
}
