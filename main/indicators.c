#include "indicators.h"
#include "driver/gpio.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "freertos/timers.h"

#define NUM_INDICATORS 2
const int indicator_pins[NUM_INDICATORS] = {INDICATOR_1_PIN, INDICATOR_2_PIN};

static TimerHandle_t indicator_timers[NUM_INDICATORS];

void indicator_timer_callback(TimerHandle_t xTimer);

void indicators_configure_gpio()
{
    gpio_config_t io_conf;
    io_conf.pin_bit_mask = (1ULL << INDICATOR_1_PIN) | (1ULL << INDICATOR_2_PIN);
    io_conf.intr_type = GPIO_INTR_DISABLE;
    io_conf.mode = GPIO_MODE_OUTPUT;
    io_conf.pull_down_en = 0;
    io_conf.pull_up_en = 0;
    gpio_config(&io_conf);
}

void indicators_activate(int indicator, long int time_ms)
{
    if (indicator < 0 || indicator >= NUM_INDICATORS)
        return;

    gpio_set_level(indicator_pins[indicator], 1);

    xTimerChangePeriod(indicator_timers[indicator], pdMS_TO_TICKS(time_ms), 0);
    xTimerStart(indicator_timers[indicator], 0);
}

void indicators_deactivate(int indicator)
{
    if (indicator < 0 || indicator >= NUM_INDICATORS)
        return;

    gpio_set_level(indicator_pins[indicator], 0);
    xTimerStop(indicator_timers[indicator], 0);
}

void indicator_timer_callback(TimerHandle_t xTimer)
{
    int timer_id = (int)pvTimerGetTimerID(xTimer);
    gpio_set_level(indicator_pins[timer_id], 0);
}

void indicators_configure()
{
    indicators_configure_gpio();
    gpio_set_level(INDICATOR_1_PIN, 0);
    gpio_set_level(INDICATOR_2_PIN, 0);

    for (int i = 0; i < NUM_INDICATORS; i++)
    {
        indicator_timers[i] = xTimerCreate(
            "IndicatorTimer",
            pdMS_TO_TICKS(1000),
            pdFALSE,
            (void *)i,
            indicator_timer_callback);
    }
}