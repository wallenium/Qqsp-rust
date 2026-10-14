#ifndef QSP_CORE_H
#define QSP_CORE_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct QspEngine QspEngine;
#define QSPH(...) (void *user, QspEngine *e, ##__VA_ARGS__)
typedef struct QspHost {
  void *user;
  void (*debug)(void *, QspEngine *, const char *);
  int (*is_playing_file)(void *, QspEngine *, const char *);
  void (*play_file)(void *, QspEngine *, const char *, int);
  void (*close_file)(void *, QspEngine *, const char *);      /* NULL = all */
  void (*show_image)(void *, QspEngine *, const char *);      /* NULL = hide */
  void (*show_window)(void *, QspEngine *, int win, int show); /* 0 acts 1 objs 2 vars 3 input */
  void (*delete_menu)(void *, QspEngine *);
  void (*add_menu_item)(void *, QspEngine *, const char *name, const char *image);
  int (*show_menu)(void *, QspEngine *);
  void (*show_message)(void *, QspEngine *, const char *);
  void (*refresh_int)(void *, QspEngine *, int redraw);
  void (*set_timer)(void *, QspEngine *, int msecs);
  void (*set_input_str_text)(void *, QspEngine *, const char *);
  void (*system)(void *, QspEngine *, const char *);
  void (*open_game_status)(void *, QspEngine *, const char *); /* NULL = ask */
  void (*save_game_status)(void *, QspEngine *, const char *);
  void (*sleep)(void *, QspEngine *, int msecs);
  int (*ms_count)(void *, QspEngine *);
  char *(*input_box)(void *, QspEngine *, const char *prompt); /* malloc'd result */
} QspHost;
#undef QSPH

QspEngine *qsp_new(const QspHost *host);
void qsp_free(QspEngine *);
void qsp_string_free(char *);
char *qsp_version(void);
void qsp_enable_debug(QspEngine *, int on);

int qsp_load_game(QspEngine *, const char *path);
int qsp_load_game_data(QspEngine *, const uint8_t *data, size_t len, const char *name);
int qsp_restart(QspEngine *, int refresh);
int qsp_exec_string(QspEngine *, const char *code, int refresh);
int qsp_exec_location(QspEngine *, const char *name, int refresh);
int qsp_exec_counter(QspEngine *, int refresh);
int qsp_exec_user_input(QspEngine *, int refresh);
int qsp_set_sel_action(QspEngine *, int ind, int refresh);
int qsp_exec_sel_action(QspEngine *, int refresh);
int qsp_set_sel_object(QspEngine *, int ind, int refresh);
int qsp_expr_value(QspEngine *, const char *expr, int *is_str, int *num_val, char **str_val);

int qsp_save_game(QspEngine *, const char *path, int refresh);
int qsp_open_saved_game(QspEngine *, const char *path, int refresh);

void qsp_set_input_text(QspEngine *, const char *);
int qsp_window_visible(QspEngine *, int win);
char *qsp_main_desc(QspEngine *, int *changed);
char *qsp_vars_desc(QspEngine *, int *changed);
int qsp_actions_count(QspEngine *);
int qsp_actions_changed(QspEngine *);
void qsp_action_data(QspEngine *, int i, char **image, char **desc);
int qsp_objects_count(QspEngine *);
int qsp_objects_changed(QspEngine *);
void qsp_object_data(QspEngine *, int i, char **image, char **desc);
int qsp_sel_action(QspEngine *);
int qsp_sel_object(QspEngine *);
char *qsp_cur_loc(QspEngine *);
char *qsp_qst_full_path(QspEngine *);
int qsp_full_refresh_count(QspEngine *);
int qsp_var_values(QspEngine *, const char *name, int ind, int *num_val, char **str_val);
char *qsp_vars_dump(QspEngine *);
int qsp_var_num(QspEngine *, const char *name);
int qsp_last_error_num(QspEngine *);
char *qsp_error_description(QspEngine *);
#ifdef __cplusplus
}
#endif
#endif
