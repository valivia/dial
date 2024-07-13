#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <inttypes.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "freertos/queue.h"
#include "driver/gpio.h"

// Pins
#define BUTTON_SIGNAL_PIN 34
#define BUTTON_SELECT_1_PIN 36
#define BUTTON_SELECT_2_PIN 37
#define BUTTON_SELECT_3_PIN 35

// State
int button_index = 0;

void set_demultiplex_channel(int index)
{
    gpio_set_level(BUTTON_SELECT_1_PIN, (index & 0b001) ? 1 : 0);
    gpio_set_level(BUTTON_SELECT_2_PIN, (index & 0b010) ? 1 : 0);
    gpio_set_level(BUTTON_SELECT_3_PIN, (index & 0b100) ? 1 : 0);
}

void configure_buttons_gpio()
{
    gpio_set_direction(BUTTON_SIGNAL_PIN, GPIO_MODE_INPUT);
    gpio_set_pull_mode(BUTTON_SIGNAL_PIN, GPIO_PULLUP_ONLY);

    gpio_set_direction(BUTTON_SELECT_1_PIN, GPIO_MODE_OUTPUT);
    gpio_set_direction(BUTTON_SELECT_2_PIN, GPIO_MODE_OUTPUT);
    gpio_set_direction(BUTTON_SELECT_3_PIN, GPIO_MODE_OUTPUT);
}

void buttons_task()
{
    for (;;)
    {
        set_demultiplex_channel(button_index);
        vTaskDelay(1);
        int button_state = gpio_get_level(BUTTON_SIGNAL_PIN);

        if (button_state == 0)
        {
            printf("Button %d pressed\n", button_index);
        }

        // increase index, max 7
        button_index = (button_index + 1) % 8;
    }

    vTaskDelete(NULL);
}

void configure_buttons()
{
    configure_buttons_gpio();
    xTaskCreate(buttons_task, "button_task", 4096, NULL, 0, NULL);
}
