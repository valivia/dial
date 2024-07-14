#include "indicators.h"

#include "driver/gpio.h"

void indicators_configure_gpio()
{
    gpio_config_t io_conf;
    io_conf.pin_bit_mask = ((1ULL << INDICATOR_1_PIN) | (1ULL << INDICATOR_2_PIN));
    io_conf.mode = GPIO_MODE_OUTPUT;
    io_conf.pull_down_en = 0;
    io_conf.pull_up_en = 0;
    gpio_config(&io_conf);
}

void indicators_set_state(int state)
{
    gpio_set_level(INDICATOR_1_PIN, state);
    gpio_set_level(INDICATOR_2_PIN, state);
}

void indicators_configure()
{
    indicators_configure_gpio();
    gpio_set_level(INDICATOR_1_PIN, 0);
    gpio_set_level(INDICATOR_2_PIN, 0);
}