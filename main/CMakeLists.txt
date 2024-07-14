set(srcs
    "usb.c" "dial.c" "buttons.c" "main.c" "indicators.c"
)

set(reqs
    "tinyusb" "esp_timer" "freertos"
)

idf_component_register(
    SRCS ${srcs}
    INCLUDE_DIRS "."
    REQUIRES ${reqs}
    PRIV_REQUIRES driver
)