#include "touch.h"

#include <stdio.h>
#include <inttypes.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "freertos/queue.h"
#include "esp_log.h"
#include "driver/touch_pad.h"

static const char *TAG = "Touch pad";

// Queue
static QueueHandle_t touch_queue = NULL;
typedef struct touch_msg
{
    touch_pad_intr_mask_t intr_mask;
    uint32_t pad_num;
    uint32_t pad_status;
    uint32_t pad_val;
} touch_event_t;

// Config
#define TOUCH_BUTTON_DENOISE_ENABLE 1
#define TOUCH_CHANGE_CONFIG 0
#define TOUCH_AUTO_CALIBRATION 0

// Parameters
const uint32_t touch_base = 330847;
static const float touch_threshold = 0.2;

static void touchsensor_interrupt_cb(void *arg)
{
    int task_awoken = pdFALSE;
    touch_event_t evt;

    evt.intr_mask = touch_pad_read_intr_status_mask();
    evt.pad_status = touch_pad_get_status();
    evt.pad_num = touch_pad_get_current_meas_channel();

    xQueueSendFromISR(touch_queue, &evt, &task_awoken);
    if (task_awoken == pdTRUE)
    {
        portYIELD_FROM_ISR();
    }
}

static void tp_example_set_thresholds(void)
{
    uint32_t touch_value;
#if TOUCH_AUTO_CALIBRATION
    // read benchmark value
    touch_pad_read_benchmark(touch_pin, &touch_value);
#else
    touch_value = touch_base;
#endif
    // set interrupt threshold.
    touch_pad_set_thresh(TOUCH_PIN, touch_value * touch_threshold);
    ESP_LOGI(TAG, "touch pad [%d] base %" PRIu32 ", thresh %" PRIu32,
             TOUCH_PIN, touch_value, (uint32_t)(touch_value * touch_threshold));
}

static void touchsensor_filter_set(touch_filter_mode_t mode)
{
    /* Filter function */
    touch_filter_config_t filter_info = {
        .mode = mode,      // Test jitter and filter 1/4.
        .debounce_cnt = 1, // 1 time count.
        .noise_thr = 0,    // 50%
        .jitter_step = 4,  // use for jitter mode.
        .smh_lvl = TOUCH_PAD_SMOOTH_IIR_2,
    };
    touch_pad_filter_set_config(&filter_info);
    touch_pad_filter_enable();
    ESP_LOGI(TAG, "touch pad filter init");
}

static void tp_example_read_task(void *pvParameter)
{
    touch_event_t evt = {0};

    /* Wait touch sensor init done */
    vTaskDelay(50 / portTICK_PERIOD_MS);
    tp_example_set_thresholds();

    while (1)
    {
        int ret = xQueueReceive(touch_queue, &evt, (TickType_t)portMAX_DELAY);
        if (ret != pdTRUE)
        {
            continue;
        }
        if (evt.intr_mask & TOUCH_PAD_INTR_MASK_ACTIVE)
        {

            ESP_LOGI(TAG, "TouchSensor [%" PRIu32 "] be activated, status mask 0x%" PRIu32 "", evt.pad_num, evt.pad_status);
        }

        if (evt.intr_mask & TOUCH_PAD_INTR_MASK_INACTIVE)
        {

            ESP_LOGI(TAG, "TouchSensor [%" PRIu32 "] be inactivated, status mask 0x%" PRIu32, evt.pad_num, evt.pad_status);
        }
        if (evt.intr_mask & TOUCH_PAD_INTR_MASK_SCAN_DONE)
        {
            ESP_LOGI(TAG, "The touch sensor group measurement is done [%" PRIu32 "].", evt.pad_num);
        }
        if (evt.intr_mask & TOUCH_PAD_INTR_MASK_TIMEOUT)
        {
            /* Add your exception handling in here. */
            ESP_LOGI(TAG, "Touch sensor channel %" PRIu32 " measure timeout. Skip this exception channel!!", evt.pad_num);
            touch_pad_timeout_resume(); // Point on the next channel to measure.
        }
    }
}

void touch_configure(void)
{
    if (touch_queue == NULL)
    {
        touch_queue = xQueueCreate(1, sizeof(touch_event_t));
    }
    // Initialize touch pad peripheral, it will start a timer to run a filter
    ESP_LOGI(TAG, "Initializing touch pad");
    /* Initialize touch pad peripheral. */
    touch_pad_init();
    touch_pad_config(TOUCH_PIN);

#if TOUCH_CHANGE_CONFIG
    /* If you want change the touch sensor default setting, please write here(after initialize). There are examples: */
    touch_pad_set_measurement_interval(TOUCH_PAD_SLEEP_CYCLE_DEFAULT);
    touch_pad_set_charge_discharge_times(TOUCH_PAD_MEASURE_CYCLE_DEFAULT);
    touch_pad_set_voltage(TOUCH_PAD_HIGH_VOLTAGE_THRESHOLD, TOUCH_PAD_LOW_VOLTAGE_THRESHOLD, TOUCH_PAD_ATTEN_VOLTAGE_THRESHOLD);
    touch_pad_set_idle_channel_connect(TOUCH_PAD_IDLE_CH_CONNECT_DEFAULT);
    for (int i = 0; i < TOUCH_BUTTON_NUM; i++)
    {
        touch_pad_set_cnt_mode(button[i], TOUCH_PAD_SLOPE_DEFAULT, TOUCH_PAD_TIE_OPT_DEFAULT);
    }
#endif

#if TOUCH_BUTTON_DENOISE_ENABLE
    /* Denoise setting at TouchSensor 0. */
    touch_pad_denoise_t denoise = {
        /* The bits to be cancelled are determined according to the noise level. */
        .grade = TOUCH_PAD_DENOISE_BIT4,
        /* By adjusting the parameters, the reading of T0 should be approximated to the reading of the measured channel. */
        .cap_level = TOUCH_PAD_DENOISE_CAP_L4,
    };
    touch_pad_denoise_set_config(&denoise);
    touch_pad_denoise_enable();
    ESP_LOGI(TAG, "Denoise function init");
#endif

    /* Filter setting */
    touchsensor_filter_set(TOUCH_PAD_FILTER_IIR_16);
    touch_pad_timeout_set(true, SOC_TOUCH_PAD_THRESHOLD_MAX);
    /* Register touch interrupt ISR, enable intr type. */
    touch_pad_isr_register(touchsensor_interrupt_cb, NULL, TOUCH_PAD_INTR_MASK_ALL);
    /* If you have other touch algorithm, you can get the measured value after the `TOUCH_PAD_INTR_MASK_SCAN_DONE` interrupt is generated. */
    touch_pad_intr_enable(TOUCH_PAD_INTR_MASK_ACTIVE | TOUCH_PAD_INTR_MASK_INACTIVE | TOUCH_PAD_INTR_MASK_TIMEOUT);

    /* Enable touch sensor clock. Work mode is "timer trigger". */
    touch_pad_set_fsm_mode(TOUCH_FSM_MODE_TIMER);
    touch_pad_fsm_start();

    // Start a task to show what pads have been touched
    xTaskCreate(&tp_example_read_task, "touch_pad_read_task", 4096, NULL, 5, NULL);
}
