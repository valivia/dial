#include "buttons.h"

#include <stdlib.h>
#include <inttypes.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "driver/gpio.h"
#include "esp_timer.h"
#include "class/hid/hid_device.h"

#include "usb.h"
#include "state.h"
#include "indicators.h"

// State
int button_index = 0;
int next_page_state = 1;
int previous_page_state = 1;

void set_demultiplex_channel(int index)
{
    gpio_set_level(BUTTON_SELECT_1_PIN, (index & 0b001) ? 1 : 0);
    gpio_set_level(BUTTON_SELECT_2_PIN, (index & 0b010) ? 1 : 0);
    gpio_set_level(BUTTON_SELECT_3_PIN, (index & 0b100) ? 1 : 0);
}

void buttons_configure_gpio()
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
        vTaskDelay(1 / portTICK_PERIOD_MS);
        int button_state = gpio_get_level(BUTTON_SIGNAL_PIN);

        // page logic if button 5 or 6
        if (button_index == 5 || button_index == 6)
        {
            int *page_state = (button_index == 5) ? &previous_page_state : &next_page_state;

            if (button_state != *page_state)
            {
                *page_state = button_state;
                if (button_state == 0)
                {
                    int direction = (button_index == 5) ? -1 : 1;
                    int indicator = (button_index == 5) ? 0 : 1;

                    change_page_index(direction);
                    indicators_activate(indicator, 200);
                }
            }
        }

        // Action logic if generic button
        else
        {
            receive_button_event(button_index, button_state);
        }

        // increase index, max 7
        button_index = (button_index + 1) % 8;
    }

    vTaskDelete(NULL);
}

void buttons_configure()
{
    buttons_configure_gpio();
    xTaskCreate(buttons_task, "button_task", 4096, NULL, 0, NULL);
}
