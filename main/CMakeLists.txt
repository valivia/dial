set(srcs
    "usb.c" "dial.c" "buttons.c" "main.c" "indicators.c" "touch.c" "state.c" "wifi.c" "mqtt.c"
)

set(reqs
    "freertos" "esp_timer" "esp_wifi" "tinyusb" "esp_event" "nvs_flash" "mqtt"
)

idf_component_register(
    SRCS ${srcs}
    INCLUDE_DIRS "."
    REQUIRES ${reqs}
    PRIV_REQUIRES driver
)