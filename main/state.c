#include "state.h"

#include <stdint.h>
#include "freertos/FreeRTOS.h"
#include "class/hid/hid_device.h"
#include "esp_log.h"
#include "esp_timer.h"

#include "usb.h"

int state_current_page_index = 0;
static const char *TAG = "state";

typedef struct
{
    int page_index;
    int action_index;
    bool state;
} button_queue_item;

QueueHandle_t button_queue;

page_t pages[NUM_PAGES];

const uint8_t button_keycodes[5] = {
    HID_KEY_F13,
    HID_KEY_F14,
    HID_KEY_F15,
    HID_KEY_F16,
    HID_KEY_F17};

void change_page_index(int change)
{
    state_current_page_index += change;
    if (state_current_page_index < 0)
    {
        state_current_page_index = NUM_PAGES - 1;
    }
    else if (state_current_page_index >= NUM_PAGES)
    {
        state_current_page_index = 0;
    }

    ESP_LOGI(TAG, "Page index: %d (%d)", state_current_page_index, change);
}

void receive_button_event(int action_index, bool state)
{
    button_queue_item item;
    item.page_index = state_current_page_index;
    item.action_index = action_index;
    item.state = state;
    xQueueSend(button_queue, &item, 0);
}

void keyboard_action(action_t *action, int button_state)
{
    if (button_state == 1)
    {
        // Release key
        if (action->trigger_type == ACTION_HOLD && action->data.last_state == 0)
        {
            action->data.last_use = esp_timer_get_time() / 1000;
            usb_release_key(action->data.keyboard->keycode);
        }
    }
    else
    {
        if (action->data.last_state == 0 && !action->repeat_while_held)
        {
            return;
        }

        // Press key for 50ms
        if (action->trigger_type == ACTION_PRESS)
        {
            usb_press_key(action->data.keyboard->keycode);
            vTaskDelay(50 / portTICK_PERIOD_MS);
            usb_release_key(action->data.keyboard->keycode);
            action->data.last_use = esp_timer_get_time() / 1000;
        }

        // Press key and dont release.
        else if (action->trigger_type == ACTION_HOLD)
        {
            usb_press_key(action->data.keyboard->keycode);
        }

        // Toggle key
        else if (action->trigger_type == ACTION_TOGGLE)
        {
            action->data.last_use = esp_timer_get_time() / 1000;
            if (action->data.active)
            {
                action->data.active = false;
                usb_release_key(action->data.keyboard->keycode);
            }
            else
            {
                action->data.active = true;
                usb_press_key(action->data.keyboard->keycode);
            }
        }
    }

    action->data.last_state = button_state;
}

void state_handle_action(action_t *action, int button_state)
{
    switch (action->type)
    {
    case ACTION_TYPE_KEYBOARD:
        keyboard_action(action, button_state);
        break;
    default:
        printf("Unknown action type.\n");
        break;
    }
}

void state_task()
{
    button_queue = xQueueCreate(10, sizeof(button_queue_item));
    while (1)
    {
        button_queue_item item;
        if (xQueueReceive(button_queue, &item, portMAX_DELAY) == pdTRUE)
        {
            action_t *action = &pages[item.page_index].actions[item.action_index];
            state_handle_action(action, item.state);
        }
    }
}

void state_configure()
{
    xTaskCreatePinnedToCore(state_task, "state_task", 4096, NULL, 0, NULL, 1);
    for (int i = 0; i < NUM_PAGES; i++)
    {

        for (int j = 0; j < ACTIONS_PER_PAGE; j++)
        {
            pages[i].actions[j].type = ACTION_TYPE_KEYBOARD;
            pages[i].actions[j].data.keyboard = malloc(sizeof(keyboard_action_t));
            if (j == 0)
            {
                pages[i].actions[j].trigger_type = ACTION_TOGGLE;
            }
            else if (j == 1)
            {
                pages[i].actions[j].trigger_type = ACTION_HOLD;
            }
            else
            {
                pages[i].actions[j].trigger_type = ACTION_PRESS;
            }
            pages[i].actions[j].timeout = 0;
            pages[i].actions[j].repeat_while_held = false;
            pages[i].actions[j].data.last_use = 0;
            pages[i].actions[j].data.last_state = 1;
            pages[i].actions[j].data.active = false;
            pages[i].actions[j].data.keyboard->keycode = button_keycodes[j];
        }
    }
}