#include <stdint.h>
#include "freertos/FreeRTOS.h"

#ifndef STATE_H
#define STATE_H

// Config
#define NUM_PAGES 3
#define ACTIONS_PER_PAGE 5

// Types

// Keyboard
typedef enum
{
    ACTION_TOGGLE,
    ACTION_PRESS,
    ACTION_HOLD,
} action_trigger_type_t;

typedef struct
{
    uint8_t keycode;
} keyboard_action_t;

typedef struct
{
    int min;
    int max;
    bool optional_dial;
    char topic[32];
} mqtt_action_t;

//
typedef enum
{
    ACTION_TYPE_KEYBOARD,
    ACTION_TYPE_MQTT,
} action_type_t;

typedef struct
{
    uint32_t last_use;
    bool last_state;
    bool active;

    keyboard_action_t *keyboard;
    mqtt_action_t *mqtt;
} action_data_t;

typedef struct
{
    action_type_t type;
    action_data_t data;

    action_trigger_type_t trigger_type;
    uint32_t timeout;
    bool repeat_while_held;
} action_t;

typedef struct
{
    action_t actions[ACTIONS_PER_PAGE];
} page_t;

// Functions
void change_page_index(int change);
void receive_button_event(int action_index, bool state);
void state_handle_action(action_t *action, int button_state);
void state_configure();

#endif // STATE_H