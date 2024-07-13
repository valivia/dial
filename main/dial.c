#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <inttypes.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "freertos/queue.h"
#include "driver/gpio.h"
#include "esp_timer.h"

// Pins
#define DIAL_DATA_PIN 5
#define DIAL_MODE_PIN 4

// State
unsigned long last_interrupt_time = 0;
int dial_counter = 0;

void IRAM_ATTR dial_interrupt_handler(void *arg)
{
    unsigned long interrupt_time = esp_timer_get_time() / 1000;

    if (interrupt_time - last_interrupt_time > 30)
    {
        dial_counter++;
    }

    last_interrupt_time = interrupt_time;
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

    // Interrupts
    gpio_install_isr_service(0);
    gpio_isr_handler_add(DIAL_DATA_PIN, dial_interrupt_handler, (void *)DIAL_DATA_PIN);
    // gpio_isr_handler_add(DIAL_MODE_PIN, dial_interrupt_handler, (void *)DIAL_MODE_PIN);
}

void dial_task()
{
    configure_dial_gpio();
    esp_intr_dump(NULL);

    while (1)
    {
        vTaskDelay(200 / portTICK_PERIOD_MS);

        if (dial_counter == 0)
            continue;

        // printf("Dial counter: %d ", dial_counter);
        // printf("Dial mode: %d ", gpio_get_level(DIAL_MODE_PIN));
        // printf("last interrupt time: %lu\n", last_interrupt_time);

        unsigned long current_time = esp_timer_get_time() / 1000;

        if (current_time - last_interrupt_time > 300)
        {
            printf("reset counter at: %d\n", dial_counter);
            dial_counter = 0;
        }
    }
}

void configure_dial()
{
    printf("Data pin: %d\n", DIAL_DATA_PIN);
    printf("Mode pin: %d\n", DIAL_MODE_PIN);
    xTaskCreatePinnedToCore(dial_task, "dial_task", 2048, NULL, 0, NULL, 0);
}
