/* Simple C example using ratatui-relu via FFI. */

#include <stdio.h>
#include "include/ratatui_relu.h"

int main(void) {
    /* Smooth a single value from 0.0 to 1.0. */
    RatatuiReluTransition *t = ratatui_relu_transition_new(0.0, 4.0);
    ratatui_relu_transition_set_target(t, 1.0);

    printf("Transition 0.0 -> 1.0\n");
    for (int i = 0; i < 10; i++) {
        ratatui_relu_transition_update(t, 100);
        printf("  step %d: current=%.4f target=%.4f\n",
               i,
               ratatui_relu_transition_current(t),
               ratatui_relu_transition_target(t));
    }
    ratatui_relu_transition_free(t);

    /* Drive a multi-panel layout from C. */
    RatatuiReluMultiPanel *panel = ratatui_relu_multi_panel_new(4.0);
    printf("\nMulti-panel layout\n");

    ratatui_relu_multi_panel_toggle_header(panel);
    ratatui_relu_multi_panel_toggle_left(panel);

    for (int i = 0; i < 8; i++) {
        ratatui_relu_multi_panel_update(panel, 100);
        printf("  step %d: header=%.3f left=%.3f right=%.3f footer=%.3f\n",
               i,
               ratatui_relu_multi_panel_header(panel),
               ratatui_relu_multi_panel_left(panel),
               ratatui_relu_multi_panel_right(panel),
               ratatui_relu_multi_panel_footer(panel));
    }

    ratatui_relu_multi_panel_free(panel);
    return 0;
}
