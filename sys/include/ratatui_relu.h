#ifndef RATATUI_RELU_H
#define RATATUI_RELU_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque handles */
typedef struct RatatuiReluTransition RatatuiReluTransition;
typedef struct RatatuiReluMultiPanel RatatuiReluMultiPanel;

/* Transition */
RatatuiReluTransition *ratatui_relu_transition_new(double value, double speed);
void ratatui_relu_transition_free(RatatuiReluTransition *t);
void ratatui_relu_transition_set_target(RatatuiReluTransition *t, double target);
void ratatui_relu_transition_update(RatatuiReluTransition *t, uint64_t dt_ms);
double ratatui_relu_transition_current(const RatatuiReluTransition *t);
double ratatui_relu_transition_target(const RatatuiReluTransition *t);

/* MultiPanelState */
RatatuiReluMultiPanel *ratatui_relu_multi_panel_new(double speed);
void ratatui_relu_multi_panel_free(RatatuiReluMultiPanel *s);
void ratatui_relu_multi_panel_update(RatatuiReluMultiPanel *s, uint64_t dt_ms);
void ratatui_relu_multi_panel_toggle_header(RatatuiReluMultiPanel *s);
void ratatui_relu_multi_panel_toggle_left(RatatuiReluMultiPanel *s);
void ratatui_relu_multi_panel_toggle_right(RatatuiReluMultiPanel *s);
void ratatui_relu_multi_panel_toggle_footer(RatatuiReluMultiPanel *s);
void ratatui_relu_multi_panel_toggle_all(RatatuiReluMultiPanel *s);
double ratatui_relu_multi_panel_header(const RatatuiReluMultiPanel *s);
double ratatui_relu_multi_panel_left(const RatatuiReluMultiPanel *s);
double ratatui_relu_multi_panel_right(const RatatuiReluMultiPanel *s);
double ratatui_relu_multi_panel_footer(const RatatuiReluMultiPanel *s);

#ifdef __cplusplus
}
#endif

#endif /* RATATUI_RELU_H */
