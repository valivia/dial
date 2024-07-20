#include "dial.h"

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "driver/gpio.h"
#include "esp_timer.h"
#include "esp_log.h"

static const char *TAG = "dial";

// cycle : ~60-70ms

// Dial
int counter = 0;
long int last_dial_time = 0;
int previous_dial_state = 0;

void dial_data_isr(void *arg)
{
    long int current_time = esp_timer_get_time() / 1000;
    long int difference = current_time - last_dial_time;
    if (last_dial_time == 0 || difference > 85) // stricter: (difference > 85 && difference < 135)
    {
        counter++;
        last_dial_time = current_time;
    }
}

void dial_task(void *arg)
{
    printf("Dial task started\n");

    while (1)
    {
        if (counter == 0)
        {
            vTaskDelay(500 / portTICK_PERIOD_MS);
            continue;
        }

        if (gpio_get_level(DIAL_MODE_PIN) == 1)
        {
            ESP_LOGI(TAG, "Dial task finished: %d", counter);
            counter = 0;
            previous_dial_state = 0;
            last_dial_time = 0;
            // should prob cap the counter to 10 on the output lol

            // dont think this works reliably rn
            vTaskDelay(300 / portTICK_PERIOD_MS);
            continue;
        }
    }

    vTaskDelete(NULL);
}

void dial_configure_gpio()
{
    // Configure DIAL_DATA_PIN
    gpio_set_direction(DIAL_DATA_PIN, GPIO_MODE_INPUT);
    gpio_set_pull_mode(DIAL_DATA_PIN, GPIO_PULLUP_ONLY);
    gpio_set_intr_type(DIAL_DATA_PIN, GPIO_INTR_POSEDGE);
    gpio_install_isr_service(0);
    gpio_isr_handler_add(DIAL_DATA_PIN, dial_data_isr, NULL);

    // Configure DIAL_MODE_PIN
    gpio_set_direction(DIAL_MODE_PIN, GPIO_MODE_INPUT);
    gpio_set_pull_mode(DIAL_MODE_PIN, GPIO_PULLUP_ONLY);
}

void dial_configure()
{
    dial_configure_gpio();
    xTaskCreatePinnedToCore(dial_task, "dial_task", 4096, NULL, 0, NULL, 0);
}