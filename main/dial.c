#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <inttypes.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "freertos/queue.h"
#include "driver/gpio.h"
#include "esp_timer.h"

// cycle : ~60-70ms

// Pins
#define DIAL_DATA_PIN 5
#define DIAL_MODE_PIN 4

int counter = 0;

// Dial
long int last_dial_time = 0;
int previous_dial_state = 0;

void dial_task(void *arg)
{
    printf("Dial task started\n");

    while (1)
    {
        int dial_mode = gpio_get_level(DIAL_MODE_PIN);
        if (counter != 0 && dial_mode == 1)
        {
            printf("====\nDial task finished: %d\n====\n", counter);
            counter = 0;
            previous_dial_state = 0;
            last_dial_time = 0;
            vTaskDelay(300 / portTICK_PERIOD_MS);
            continue;
        }

        if (dial_mode == 1)
            continue;

        long int current_time = esp_timer_get_time() / 1000;

        int dial_state = gpio_get_level(DIAL_DATA_PIN);
        if (previous_dial_state == dial_state)
            continue;

        long int difference = current_time - last_dial_time;

        printf("Dial state changed to: %d, time since last change: %ld", dial_state, current_time - last_dial_time);

        if (dial_state == 1 && previous_dial_state == 0 && (last_dial_time == 0 || difference > 85)) // stricter: (difference > 85 && difference < 135)
        {
            counter++;
            printf(", --Counted-- %d", counter);
            last_dial_time = current_time;
        }

        printf("\n");

        previous_dial_state = dial_state;
    }

    vTaskDelete(NULL);
}

void configure_dial_gpio()
{
    // Configure DIAL_DATA_PIN
    gpio_set_direction(DIAL_DATA_PIN, GPIO_MODE_INPUT);
    gpio_set_pull_mode(DIAL_DATA_PIN, GPIO_PULLUP_ONLY);
    gpio_set_intr_type(DIAL_DATA_PIN, GPIO_INTR_POSEDGE);

    // Configure DIAL_MODE_PIN
    gpio_set_direction(DIAL_MODE_PIN, GPIO_MODE_INPUT);
    gpio_set_pull_mode(DIAL_MODE_PIN, GPIO_PULLUP_ONLY);
}

void configure_dial()
{
    configure_dial_gpio();
    xTaskCreatePinnedToCore(dial_task, "dial_task", 4096, NULL, 0, NULL, 0);
}